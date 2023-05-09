use async_process::Command;
use chrono::Duration;
use diesel::QueryDsl;
use diesel::{delete, insert_into, update, ExpressionMethods};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncPgConnection, RunQueryDsl, AsyncConnection};
use dotenvy::dotenv;
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::database_models::video::InsertableVideo;
use immortalis_backend_common::database_models::video_status::VideoStatus;
use immortalis_backend_common::env_var_names;
use immortalis_backend_common::schema::{scheduled_archivals, videos};
use tokio::fs;
use youtube_dl::YoutubeDl;

use tracing::{info, warn};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var(env_var_names::DATABASE_URL).unwrap(),
    );
    let application_connection_pool = Pool::builder(config).build().unwrap();

    let file_storage_location = std::env::var(env_var_names::FILE_STORAGE_LOCATION)
    .expect("FILE_STORAGE_LOCATION invalid or missing");
    let skip_download = true;

    // spawn 4 workers
    for _ in 0..std::env::var(env_var_names::TRACKER_THREAD_COUNT)
        .unwrap()
        .parse::<i32>()
        .unwrap()
    {
        let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));

        let worker_connection_pool = application_connection_pool.clone();
        let worker_file_storage_location = file_storage_location.clone();
        tokio::spawn(async move {
            let task_connection_pool = worker_connection_pool.clone();
            loop {
                interval_timer.tick().await;
                archive(task_connection_pool.clone(), &skip_download, &worker_file_storage_location).await;
            }
        });
    }

    let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(50));
    loop {
        interval_timer.tick().await;
    }
}

async fn archive(pool: Pool<AsyncPgConnection>, skip_download: &bool, file_storage_location: &str) {
    let db_connection = &mut pool.get().await.unwrap();
    let _g = db_connection.transaction::<_, diesel::result::Error ,_>(|db_connection| async move {
        let results = scheduled_archivals::table
        .limit(1)
        .filter(scheduled_archivals::not_before.lt(chrono::Utc::now().naive_utc()))
        .for_update()
        .skip_locked()
        .load::<ScheduledArchival>(db_connection)
        .await
        .unwrap();

        if results.is_empty() {
            info!("No ScheduledArchivals found");
            return Ok(());
        }

        let result = &results[0];

        let yt_video_result = YoutubeDl::new(&result.url).run_async().await;
        if let Ok(video) = yt_video_result {
            let yt_dl_video = video.into_single_video().unwrap();

            delete(scheduled_archivals::table)
                .filter(scheduled_archivals::id.eq(result.id))
                .execute(db_connection)
                .await
                .unwrap();
            info!(
                result.id = result.id,
                result.url = result.url,
                "Dequeued entry: {} with url: {}",
                result.id,
                result.url
            );

            let upload_date = yt_dl_video.upload_date.unwrap();
            let video_duration = match yt_dl_video.duration {
                Some(x) => i32::try_from(x.as_i64().unwrap()).unwrap(),
                None => 0,
            };

            let resp = reqwest::get(yt_dl_video.thumbnail.clone().unwrap()).await.unwrap().bytes().await.unwrap();

            let thumbnail_address = yt_dl_video.thumbnail.unwrap();
            let thumbnail_id = uuid::Uuid::new_v4();
            let thumbnail_extension = thumbnail_address.split(".").last().unwrap();
            fs::create_dir_all(file_storage_location.to_string()  + "thumbnails").await.unwrap();
            fs::write(file_storage_location.to_string() + "thumbnails/" + &thumbnail_id.to_string() + "." + thumbnail_extension, resp).await.unwrap();

            let file_id = uuid::Uuid::new_v4();
            let video = InsertableVideo {
                title: yt_dl_video.title,
                channel: yt_dl_video.channel.unwrap(),
                views: yt_dl_video.view_count.unwrap(),
                upload_date: chrono::NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(
                        upload_date[0..=3].parse::<i32>().unwrap(),
                        upload_date[4..=5].parse::<u32>().unwrap(),
                        upload_date[6..=7].parse::<u32>().unwrap(),
                    )
                    .unwrap(),
                    chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap(),
                ),
                archived_date: chrono::Utc::now().naive_utc(),
                duration: video_duration,
                thumbnail_address: thumbnail_address.clone(),
                original_url: result.url.clone(),
                status:
                    immortalis_backend_common::database_models::video_status::VideoStatus::BeingArchived,
                file_id: file_id.clone(),
                file_extension: "mkv".to_string(),
                thumbnail_id: thumbnail_id.clone(),
                thumbnail_extension: thumbnail_extension.to_string()
            };

            insert_into(videos::table)
                .values(video)
                .on_conflict_do_nothing()
                .execute(db_connection)
                .await
                .unwrap();

            // if SKIP_DOWNLOAD is set, we skip the download
            if *skip_download
            {
                let cmd = Command::new("yt-dlp")
                    .arg(&result.url)
                    .arg("-o")
                    .arg(file_storage_location.to_string() + file_id.to_string().as_str() + ".%(ext)s")
                    .arg("--embed-thumbnail")
                    .arg("--embed-metadata")
                    .arg("--embed-chapters")
                    .arg("--embed-info-json")
                    .arg("--embed-subs")
                    .arg("--wait-for-video")
                    .arg("60")
                    .arg("--live-from-start")
                    .arg("--print")
                    .arg(file_storage_location.to_string() + "%(title)s",
                    )
                    .arg("--no-simulate")
                    .output();

                cmd.await.unwrap();
            }

            //tokio::time::sleep(tokio::time::Duration::from_secs(15)).await; //placeholder for actual download
            // if duration is 0 (video), we're done. If it isnt (livestream), we need to reload the metadata and update the duration
            if video_duration != 0 {
                update(videos::table)
                    .set(videos::status.eq(VideoStatus::Archived))
                    .execute(db_connection)
                    .await
                    .unwrap();
                return Ok(())
            }

            let video_duration = YoutubeDl::new(&result.url)
                .run_async()
                .await
                .unwrap()
                .into_single_video()
                .unwrap()
                .duration
                .unwrap()
                .as_i64()
                .unwrap();

            update(videos::table)
                .set((
                    videos::status.eq(VideoStatus::Archived),
                    videos::duration.eq(i32::try_from(video_duration).unwrap()),
                ))
                .execute(db_connection)
                .await
                .unwrap();
        } else {
            // try again in 10 minutes
            update(scheduled_archivals::table)
                .set(
                    scheduled_archivals::not_before.eq(chrono::Utc::now()
                        .naive_utc()
                        .checked_add_signed(Duration::minutes(10))
                        .unwrap()),
                )
                .execute(db_connection)
                .await
                .unwrap();
            warn!(
                "Received error {:#?}. Video {} will be retried in 10 minutes",
                yt_video_result, result.url
            );
        }

        Ok(())
    }.scope_boxed()).await;
}
