use std::sync::Arc;

use async_process::Command;
use chrono::Duration;
use diesel::QueryDsl;
use diesel::{delete, insert_into, update, ExpressionMethods, OptionalExtension};
use diesel_async::pooled_connection::deadpool::{Pool, self};
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

use tracing::{info, warn, error};

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

    let scheduled_archival = match dequeue(db_connection, env_var_config.archiver_archiving_timeout_seconds).await {
        Ok(x) =>  {
            match x {
                Some(scheduled_archival) => {
                    info!(
                        scheduled_archival.id = scheduled_archival.id,
                        scheduled_archival.url = scheduled_archival.url,
                        "Dequeued entry: {} with url: {}",
                        scheduled_archival.id,
                        scheduled_archival.url
                    );
                    scheduled_archival
                },
                None => {
                    info!("No ScheduledArchivals found");
                    return;
                }
            }
        },
        Err(x) => {
            error!("Failed to Dequeue ScheduledArchival, encountered error {}", x);
            return;
        },
    };

    let yt_video_result = YoutubeDl::new(&scheduled_archival.url).run_async().await;
    
    // on error, schedule retry and return early;
    if let Err(_e) = &yt_video_result {
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
            yt_video_result, scheduled_archival.url
        );
        return;
    }

    let yt_dl_video = yt_video_result.unwrap().into_single_video().unwrap();

    let (thumbnail_id, thumbnail_extension, thumbnail_size) = download_image(&yt_dl_video.thumbnail.clone().unwrap(), &env_var_config.file_storage_location).await;

    // get file_size from youtube (exact or if its unknown then aprox). This value may be replaced by the actual size of the file after the download
    let mut file_size = yt_dl_video.filesize.unwrap_or(yt_dl_video.filesize_approx.unwrap_or(0.0) as i64);
    
    let file_id = uuid::Uuid::new_v4();
    let video = InsertableVideo::new(yt_dl_video, immortalis_backend_common::database_models::video_status::VideoStatus::BeingArchived, file_id, thumbnail_id);

    // insert file for thumbnail
    insert_into(files::table)
        .values(File {id: thumbnail_id, file_name: video.title.to_string(), file_extension: thumbnail_extension.to_string(), size: thumbnail_size as i64})
        .execute(db_connection)
        .await
        .unwrap();

    // insert file for video
    insert_into(files::table)
        .values(File {id: video.file_id, file_name: video.title.to_string(), file_extension: "mkv".to_string(), size: file_size})
        .execute(db_connection)
        .await
        .unwrap();

    // insert video
    insert_into(videos::table)
        .values(&video)
        .on_conflict_do_nothing()
        .execute(db_connection)
        .await
        .unwrap();

    // if simulate_download is false, we perform the actual download, otherwise we wait for simulated_download_duration_seconds
    if !env_var_config.simulate_download
    {
        let temp_file_name = download_video(&scheduled_archival.url, &env_var_config.temp_file_storage_location, &file_id)
        .await;

        file_size = fs::File::open(&temp_file_name).await.unwrap().metadata().await.unwrap().len() as i64;
        // move it from temp storage to longtime storage. This may later be replaced by for example an external S3 or similar
        fs::rename(&temp_file_name, env_var_config.file_storage_location.to_string() + file_id.to_string().as_str() + ".mkv").await.unwrap();
    } else {
        tokio::time::sleep(tokio::time::Duration::from_secs(env_var_config.simulated_download_duration_seconds)).await;
    }

    // update video file size after download
    update(files::table)
        .set(files::size.eq(file_size))
        .filter(files::id.eq(file_id))
        .execute(db_connection)
        .await
        .unwrap();

    // if duration is 0 (video), we're done. If it isnt (livestream), we need to reload the metadata and update the duration
    if video.duration != 0 || env_var_config.simulate_download {
        update(videos::table)
            .set(videos::status.eq(VideoStatus::Archived))
            .execute(db_connection)
            .await
            .unwrap();
        return
    }
    
    let video_duration = YoutubeDl::new(&scheduled_archival.url)
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
    
    // delete the schedule once archival is completed
    delete(scheduled_archivals::table)
        .filter(scheduled_archivals::id.eq(scheduled_archival.id))
        .execute(db_connection)
        .await
        .unwrap();
}

/// returns the full path of the file
async fn download_video(url: &str, temp_file_storage_location: &str, file_id: &uuid::Uuid) -> String {
    let file_name = temp_file_storage_location.to_string() + file_id.to_string().as_str() + ".mkv";
    let cmd = Command::new("yt-dlp")
                .arg(url)
                .arg("-o")
                .arg(temp_file_storage_location.to_string() + file_id.to_string().as_str() + ".mkv")
                .arg("--embed-thumbnail") // webm doesnt support embedded thumbnails, so we should get .mkv files
                .arg("--embed-metadata")
                .arg("--embed-chapters")
                .arg("--embed-info-json")
                .arg("--embed-subs")
                .arg("--wait-for-video")
                .arg("60")
                .arg("--live-from-start")
                .arg("--no-simulate")
                .output();

    cmd.await.unwrap();
    file_name
}

/// dequeues a ScheduledArchival. The Entry will become available again once the processing_timeout has passed, if it hasn't been deleted by then
async fn dequeue(db_connection: &mut deadpool::Object<AsyncPgConnection>, processing_timeout_seconds: i64) -> Result<Option<ScheduledArchival>, diesel::result::Error> {
    
    db_connection.transaction::<Option<ScheduledArchival>, diesel::result::Error ,_>(|db_connection| async move {

        let result = scheduled_archivals::table
            .limit(1)
            .filter(scheduled_archivals::not_before.lt(chrono::Utc::now().naive_utc()))
            .for_update()
            .skip_locked()
            .first::<ScheduledArchival>(db_connection)
            .await
            .optional()
            .unwrap();

        // set not_before to now + timeout. This prevents other processes from trying to preform it as well and allows retry in case this process crashes
        update(scheduled_archivals::table)
            .set(scheduled_archivals::not_before.eq(chrono::Utc::now()
            .naive_utc()
            .checked_add_signed(Duration::seconds(processing_timeout_seconds))
            .unwrap()))
            .execute(db_connection)
            .await
            .unwrap();
        Ok(result)
    }.scope_boxed())
    .await
}

/// trims query params and downloads the image at the specified url. The image is saved with a Uuid which is returned along with the extension and the file size
async fn download_image(url: &str, file_storage_location: &str) -> (uuid::Uuid, String, usize) {
    let resp = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let thumbnail_id = uuid::Uuid::new_v4();
    let mut thumbnail_extension = url.split('.').last().unwrap();
    thumbnail_extension = &thumbnail_extension[0..thumbnail_extension.find('?').unwrap_or(thumbnail_extension.len())]; // trim params that may follow the extension

    fs::write(file_storage_location.to_string() + &thumbnail_id.to_string() + "." + thumbnail_extension, &resp).await.unwrap();
    (thumbnail_id, thumbnail_extension.into(), resp.len())
}
