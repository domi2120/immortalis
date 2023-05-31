use std::sync::Arc;

use async_process::Command;
use chrono::Duration;
use diesel::QueryDsl;
use diesel::{delete, insert_into, update, ExpressionMethods};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use immortalis_backend_common::database_models::file::File;
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::database_models::video::InsertableVideo;
use immortalis_backend_common::database_models::video_status::VideoStatus;
use immortalis_backend_common::env_var_config::EnvVarConfig;
use immortalis_backend_common::schema::{files, scheduled_archivals, videos};
use tokio::fs;
use youtube_dl::YoutubeDl;

use tracing::{info, warn};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let env_var_config = Arc::new(envy::from_env::<EnvVarConfig>().unwrap());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    fs::create_dir_all(env_var_config.file_storage_location.clone())
        .await
        .expect("could not create file_storage_location");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &env_var_config.database_url,
    );
    let application_connection_pool = Pool::builder(config).build().unwrap();

    // spawn workers equal to archiver_thread_count
    for _ in 0..env_var_config.archiver_thread_count {
        let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));

        let worker_connection_pool = application_connection_pool.clone();
        let worker_env_var_config = env_var_config.clone();

        tokio::spawn(async move {
            let task_env_var_config = worker_env_var_config.clone();
            let task_connection_pool = worker_connection_pool.clone();
            loop {
                interval_timer.tick().await;
                archive(task_connection_pool.clone(), task_env_var_config.clone()).await;
            }
        });
    }

    let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(50));
    loop {
        interval_timer.tick().await;
    }
}

async fn archive(pool: Pool<AsyncPgConnection>, env_var_config: Arc<EnvVarConfig>) {
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
            let mut thumbnail_extension = thumbnail_address.split('.').last().unwrap();
            thumbnail_extension = &thumbnail_extension[0..thumbnail_extension.find('?').unwrap_or(thumbnail_extension.len())]; // trim params that may follow the extension

            fs::write(env_var_config.file_storage_location.clone() + &thumbnail_id.to_string() + "." + thumbnail_extension, &resp).await.unwrap();

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
                file_id,
                thumbnail_id,
            };

            // insert file for thumbnail
            insert_into(files::table)
                .values(File {id: thumbnail_id, file_name: video.title.to_string(), file_extension: thumbnail_extension.to_string(), size: resp.len() as i64})
                .execute(db_connection)
                .await
                .unwrap();

            // get file_size from youtube (exact or if its unknown then aprox). This value may be replaced by the actual size of the file after the download
            let mut file_size = yt_dl_video.filesize.unwrap_or(yt_dl_video.filesize_approx.unwrap_or(0.0) as i64);

            // if SKIP_DOWNLOAD is set, we skip the download
            if !env_var_config.skip_download
            {
                let cmd = Command::new("yt-dlp")
                    .arg(&result.url)
                    .arg("-o")
                    .arg(env_var_config.temp_file_storage_location.to_string() + file_id.to_string().as_str() + ".%(ext)s")
                    .arg("--embed-thumbnail") // webm doesnt support embedded thumbnails, so we should get .mkv files
                    .arg("--embed-metadata")
                    .arg("--embed-chapters")
                    .arg("--embed-info-json")
                    .arg("--embed-subs")
                    .arg("--wait-for-video")
                    .arg("60")
                    .arg("--live-from-start")
                    .arg("--print")
                    .arg(env_var_config.file_storage_location.to_string() + "%(title)s",
                    )
                    .arg("--no-simulate")
                    .output();

                cmd.await.unwrap();

                let temp_file_name = env_var_config.temp_file_storage_location.to_string() + file_id.to_string().as_str() + ".mkv";
                file_size = fs::File::open(&temp_file_name).await.unwrap().metadata().await.unwrap().len() as i64;
                // move it from temp storage to longtime storage. This may later be replaced by for example an external S3 or similar
                fs::rename(&temp_file_name, env_var_config.file_storage_location.to_string() + file_id.to_string().as_str() + ".mkv").await.unwrap();
            }

            // insert file for video
            insert_into(files::table)
                .values(File {id: video.file_id, file_name: video.title.to_string(), file_extension: "mkv".to_string(), size: file_size})
                .execute(db_connection)
                .await
                .unwrap();

            insert_into(videos::table)
                .values(&video)
                .on_conflict_do_nothing()
                .execute(db_connection)
                .await
                .unwrap();

            //tokio::time::sleep(tokio::time::Duration::from_secs(15)).await; //placeholder for actual download
            // if duration is 0 (video), we're done. If it isnt (livestream), we need to reload the metadata and update the duration
            if video_duration != 0 || env_var_config.skip_download {
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
