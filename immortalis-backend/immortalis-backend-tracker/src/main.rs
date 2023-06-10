use std::collections::HashSet;
use std::sync::Arc;

use chrono::Duration;
use diesel::QueryDsl;
use diesel::{insert_into, update, BoolExpressionMethods, ExpressionMethods, OptionalExtension};
use diesel_async::pooled_connection::deadpool::{self, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::env_var_config::EnvVarConfigTracker;
use immortalis_backend_common::schema::{scheduled_archivals, tracked_collections, videos};

use immortalis_backend_common::utilities::UrlType;
use serde_json::Value;
use tracing::{error, info};
use youtube_dl::{Playlist, YoutubeDlOutput};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let env_var_config = Arc::new(envy::from_env::<EnvVarConfigTracker>().unwrap());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &env_var_config.general_config.database_url,
    );
    let application_connection_pool = Pool::builder(config).build().unwrap();

    for _ in 0..env_var_config.tracker_thread_count {
        let worker_connection_pool = application_connection_pool.clone();
        tokio::spawn(async move {
            let task_connection_pool = worker_connection_pool.clone();
            loop {
                if !track(task_connection_pool.clone()).await {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    // if no tracked_collections were processed, wait for 5 sec
                }
            }
        });
    }

    let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(50));
    loop {
        interval_timer.tick().await;
    }
}

/// dequeues a TrackedCollection. The Entry will become available again once the processing_timeout has passed, if it hasn't been deleted by then
async fn dequeue(
    db_connection: &mut deadpool::Object<AsyncPgConnection>,
    processing_timeout_seconds: i64,
) -> Result<Option<TrackedCollection>, diesel::result::Error> {
    db_connection
        .transaction::<Option<TrackedCollection>, diesel::result::Error, _>(|db_connection| {
            async move {
                let not_checked_after = chrono::Utc::now()
                    .naive_utc()
                    .checked_sub_signed(Duration::minutes(10))
                    .unwrap();

                let result = tracked_collections::table
                    .limit(1)
                    .filter(
                        tracked_collections::last_checked
                            .lt(not_checked_after)
                            .or(tracked_collections::last_checked.is_null()),
                    )
                    .for_update()
                    .skip_locked()
                    .first::<TrackedCollection>(db_connection)
                    .await
                    .optional()?;

                if let Some(entry) = result {
                    // set not_before to now + timeout. This prevents other processes from trying to preform it as well and allows retry in case this process crashes
                    update(tracked_collections::table)
                        .set(
                            tracked_collections::last_checked.eq(chrono::Utc::now()
                                .naive_utc()
                                .checked_add_signed(Duration::seconds(processing_timeout_seconds))
                                .unwrap()),
                        )
                        .filter(tracked_collections::id.eq(entry.id))
                        .execute(db_connection)
                        .await?;
                    Ok(Some(entry))
                } else {
                    Ok(None)
                }
            }
            .scope_boxed()
        })
        .await
}

/// returns true if a tracked_collection has been processed, returns false if there were no due tracked_collections or an error occured
async fn track(pool: Pool<AsyncPgConnection>) -> bool {
    // try getting db connection, retry if it fails
    let db_connection = &mut loop {
        match pool.get().await {
            Ok(c) => break c,
            Err(e) => {
                error!("Encountered Database error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    };

    let tracked_collection = match dequeue(db_connection, 600).await {
        Ok(x) => match x {
            Some(scheduled_archival) => {
                info!(
                    scheduled_archival.id = scheduled_archival.id,
                    scheduled_archival.url = scheduled_archival.url,
                    "Dequeued entry: {} with url: {}",
                    scheduled_archival.id,
                    scheduled_archival.url
                );
                scheduled_archival
            }
            None => {
                info!("No due TrackedCollections found");
                return false;
            }
        },
        Err(x) => {
            error!(
                "Failed to Dequeue TrackedCollections, encountered error {}",
                x
            );
            return false;
        }
    };
    info!(
        "Checking collection id: {} url: {}",
        tracked_collection.id, tracked_collection.url
    );

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
        .arg(&tracked_collection.url)
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

    let mut archived_or_scheduled_video_urls = videos::table
        .select(videos::original_url)
        .load::<String>(db_connection)
        .await
        .unwrap();

    let scheduled_video_urls = scheduled_archivals::table
        .select(scheduled_archivals::url)
        .load::<String>(db_connection)
        .await
        .unwrap();

    archived_or_scheduled_video_urls.extend(scheduled_video_urls);

    let archived_or_scheduled_video_urls =
        HashSet::<String>::from_iter(archived_or_scheduled_video_urls);

    if let Some(videos) = tracked_collection.entries {
        for video in videos {
            let url = video.webpage_url.unwrap();
            
            match immortalis_backend_common::utilities::get_url_type(&url) {
                UrlType::Collection | UrlType::VideoOrCollection =>  {
                    insert_into(tracked_collections::table)
                        .values(tracked_collections::url.eq(&url))
                        .on_conflict_do_nothing()
                        .execute(db_connection)
                        .await
                        .unwrap();
                    info!("Inserted {} into TrackedCollections", url);
                    continue;
                },
                _ => ()
            }

            if archived_or_scheduled_video_urls.contains(&url) {
                info!(
                    "{} has already been archived or is scheduled for archival and will not be scheduled again",
                    url
                )
            }

            insert_into(scheduled_archivals::table)
                .values(scheduled_archivals::url.eq(&url))
                .on_conflict_do_nothing()
                .execute(db_connection)
                .await
                .unwrap();
            info!("Scheduled {} for archival", url)
        }
    }
    true
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
