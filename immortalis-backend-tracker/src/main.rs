use std::{collections::HashMap, fs, path::PathBuf, borrow::Cow};
use chrono::Duration;
use diesel::associations::HasTable;
use diesel::query_dsl::methods::LockingDsl;
use diesel::{GroupedBy, insert_into, ExpressionMethods, delete, update, BoolExpressionMethods};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::database_models::video::{Video, InsertableVideo};
use immortalis_backend_common::database_models::video_status::VideoStatus;
use immortalis_backend_common::schema::{scheduled_archivals, videos, tracked_collections};
use dotenvy::dotenv;
use async_process::Command;

use serde_json::Value;
use youtube_dl::{YoutubeDl, Playlist, YoutubeDlOutput};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var("DATABASE_URL").unwrap(),
    );
    let pool = Pool::builder(config).build().unwrap();

    let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));
    loop {
        interval_timer.tick().await;
        let pool_instance = pool.clone();
        tokio::spawn(async move { track(pool_instance).await; });
    }
    
}

async fn track(pool: Pool<AsyncPgConnection>) {
    let mut db_connection = &mut pool.get().await.unwrap();

    let not_checked_after = chrono::Utc::now().naive_utc().checked_sub_signed(Duration::minutes(10)).unwrap();
    
    let results = tracked_collections::table.limit(1).filter(tracked_collections::last_checked.lt(not_checked_after).or(tracked_collections::last_checked.is_null())).load::<TrackedCollection>(&mut db_connection).await.unwrap();

    if results.len() == 0 {
        return;
    }

    let result = &results[0];

    // update last_checked, in case we get an error later we also prevent getting stuck by doing it this early
    update(tracked_collections::table).filter(tracked_collections::id.eq(result.id)).set(tracked_collections::last_checked.eq(chrono::Utc::now().naive_utc())).execute(&mut db_connection).await.unwrap();

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
    let mut youtube_dl_output: YoutubeDlOutput;
    if is_playlist {
        let playlist: youtube_dl::Playlist = serde_json::from_value(value).unwrap();
        youtube_dl_output= YoutubeDlOutput::Playlist(Box::new(playlist))
    } else {
        let video: youtube_dl::SingleVideo = serde_json::from_value(value).unwrap();
        youtube_dl_output = YoutubeDlOutput::SingleVideo(Box::new(video))
    }
    let tracked_collection = youtube_dl_output.into_playlist().unwrap();

    if let Some(videos) = tracked_collection.entries {
        for video in videos {
            
            let url =video.webpage_url.unwrap();

            if (url.ends_with("videos") || url.ends_with("streams") || url.ends_with("shorts")) {

                insert_into(tracked_collections::table).values(tracked_collections::url.eq(&url)).execute(&mut db_connection).await.unwrap();
                continue;
            }

            println!("Scheduling {}", url);

            insert_into(scheduled_archivals::table).values(scheduled_archivals::url.eq(url)).execute(&mut db_connection).await.unwrap();
        
        }
    }

}