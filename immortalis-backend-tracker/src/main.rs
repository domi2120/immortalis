use chrono::Duration;
use diesel::QueryDsl;
use diesel::{insert_into, update, BoolExpressionMethods, ExpressionMethods};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::env_var_names;
use immortalis_backend_common::schema::{scheduled_archivals, tracked_collections};

use serde_json::Value;
use youtube_dl::{Playlist, YoutubeDlOutput};

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_max_level(tracing::Level::INFO)
    .event_format(tracing_subscriber::fmt::format::json())
    .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var(env_var_names::DATABASE_URL).unwrap(),
    );
    let application_connection_pool = Pool::builder(config).build().unwrap();

    for i in 0..2 {
        let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));
        let worker_connection_pool = application_connection_pool.clone();
        tokio::spawn(async move {
            let task_connection_pool = worker_connection_pool.clone();
            loop {

                interval_timer.tick().await;
                track(task_connection_pool.clone()).await;
            }
        });
    }

    let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(50));
    loop {
        interval_timer.tick().await;
    }
}

async fn track(pool: Pool<AsyncPgConnection>) {
    let mut db_connection = &mut pool.get().await.unwrap();

    let not_checked_after = chrono::Utc::now()
        .naive_utc()
        .checked_sub_signed(Duration::minutes(10))
        .unwrap();

    let results = tracked_collections::table
        .limit(1)
        .filter(
            tracked_collections::last_checked
                .lt(not_checked_after)
                .or(tracked_collections::last_checked.is_null()),
        )
        .load::<TrackedCollection>(&mut db_connection)
        .await
        .unwrap();

    if results.is_empty() {
        return;
    }

    let result = &results[0];

    // update last_checked, in case we get an error later we also prevent getting stuck by doing it this early
    update(tracked_collections::table)
        .filter(tracked_collections::id.eq(result.id))
        .set(tracked_collections::last_checked.eq(chrono::Utc::now().naive_utc()))
        .execute(&mut db_connection)
        .await
        .unwrap();

    // yt-dlp exits with 1 if there are videos scheduled for the future. This causes YoutubeDL to return an error
    /*
        let tracked_collection = YoutubeDl::new(&result.url)
        .extra_arg("-i")
        .run_async()
        .await
        .unwrap()
        .into_playlist()
        .unwrap();
    */
    let cmd = async_process::Command::new("yt-dlp")
        .arg(&result.url)
        .arg("-J")
        .output()
        .await;

    let value: Value = serde_json::from_reader(cmd.unwrap().stdout.as_slice()).unwrap();
    let is_playlist = value["_type"] == serde_json::json!("playlist");

    // @TODO scheduled streams will end up returning "null" in json, causing the thread to panic. This fixes it but seems kinda hacky
    let youtube_dl_output: YoutubeDlOutput = if is_playlist {
        let fixed_playlist: FixedPlaylist = serde_json::from_value(value).unwrap();
        let c = Some(
            fixed_playlist
                .entries
                .unwrap()
                .iter()
                .filter_map(|x| x.clone())
                .collect(),
        );
        let playlist: youtube_dl::Playlist = Playlist {
            entries: c,
            extractor: fixed_playlist.extractor,
            extractor_key: fixed_playlist.extractor_key,
            id: fixed_playlist.id,
            title: fixed_playlist.title,
            uploader: fixed_playlist.uploader,
            uploader_id: fixed_playlist.uploader_id,
            uploader_url: fixed_playlist.uploader_url,
            webpage_url: fixed_playlist.webpage_url,
            webpage_url_basename: fixed_playlist.webpage_url_basename,
            thumbnails: fixed_playlist.thumbnails,
        };

        YoutubeDlOutput::Playlist(Box::new(playlist))
    } else {
        let video: youtube_dl::SingleVideo = serde_json::from_value(value).unwrap();
        YoutubeDlOutput::SingleVideo(Box::new(video))
    };

    let tracked_collection = youtube_dl_output.into_playlist().unwrap();

    if let Some(videos) = tracked_collection.entries {
        for video in videos {
            let url = video.webpage_url.unwrap();

            if url.ends_with("videos")
                || url.ends_with("streams")
                || url.ends_with("shorts")
                || url.ends_with("videos/")
                || url.ends_with("streams/")
                || url.ends_with("shorts/")
            {
                insert_into(tracked_collections::table)
                    .values(tracked_collections::url.eq(&url))
                    .on_conflict_do_nothing()
                    .execute(&mut db_connection)
                    .await
                    .unwrap();
                continue;
            }

            println!("Scheduling {}", url);

            insert_into(scheduled_archivals::table)
                .values(scheduled_archivals::url.eq(url))
                .on_conflict_do_nothing()
                .execute(&mut db_connection)
                .await
                .unwrap();
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FixedPlaylist {
    pub entries: Option<Vec<Option<youtube_dl::SingleVideo>>>,
    pub extractor: Option<String>,
    pub extractor_key: Option<String>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub uploader_url: Option<String>,
    pub webpage_url: Option<String>,
    pub webpage_url_basename: Option<String>,
    pub thumbnails: Option<Vec<youtube_dl::Thumbnail>>,
}
