use async_process::Command;
use chrono::Duration;
use diesel::QueryDsl;
use diesel::{delete, insert_into, update, ExpressionMethods};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::database_models::video::InsertableVideo;
use immortalis_backend_common::database_models::video_status::VideoStatus;
use immortalis_backend_common::schema::{scheduled_archivals, videos};
use youtube_dl::YoutubeDl;

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
        tokio::spawn(async move {
            test(pool_instance).await;
        });
    }
}

async fn test(pool: Pool<AsyncPgConnection>) {
    let mut db_connection = &mut pool.get().await.unwrap();

    let results = scheduled_archivals::table
        .limit(1)
        .filter(scheduled_archivals::not_before.lt(chrono::Utc::now().naive_utc()))
        .load::<ScheduledArchival>(&mut db_connection)
        .await
        .unwrap();

    if results.is_empty() {
        return;
    }

    let result = &results[0];

    let yt_video_result = YoutubeDl::new(&result.url).run_async().await;
    if let Ok(video) = yt_video_result {
        let yt_dl_video = video.into_single_video().unwrap();

        delete(scheduled_archivals::table)
            .filter(scheduled_archivals::id.eq(result.id))
            .execute(&mut db_connection)
            .await
            .unwrap();
        println!("Dequeued entry: {} with url: {}", result.id, result.url);

        let upload_date = yt_dl_video.upload_date.unwrap();
        let video_duration = match yt_dl_video.duration {
            Some(x) => i32::try_from(x.as_i64().unwrap()).unwrap(),
            None => 0,
        };

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
            thumbnail_address: yt_dl_video.thumbnail.unwrap(),
            original_url: result.url.clone(),
            status:
                immortalis_backend_common::database_models::video_status::VideoStatus::BeingArchived,
        };

        insert_into(videos::table)
            .values(video)
            .execute(&mut db_connection)
            .await
            .unwrap();

        // if SKIP_DOWNLOAD is set, we skip the download
        if std::env::var("SKIP_DOWNLOAD").unwrap_or_default().is_empty() {
            let cmd = Command::new("yt-dlp")
                    .arg(&result.url)
                    .arg("-o")
                    .arg(
                        std::env::var("FILE_STORAGE_LOCATION")
                            .expect("FILE_STORAGE_LOCATION invalid or missing")
                            + "%(title)s.%(ext)s",
                    )
                    .arg("--embed-thumbnail")
                    .arg("--embed-metadata")
                    .arg("--embed-chapters")
                    .arg("--embed-info-json")
                    .arg("--embed-subs")
                    .arg("--wait-for-video")
                    .arg("60")
                    .arg("--live-from-start")
                    .arg("--print")
                    .arg(
                        std::env::var("FILE_STORAGE_LOCATION")
                            .expect("FILE_STORAGE_LOCATION invalid or missing")
                            + "%(title)s",
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
                .execute(&mut db_connection)
                .await
                .unwrap();
            return;
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
            .execute(&mut db_connection)
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
            .execute(&mut db_connection)
            .await
            .unwrap();
        println!(
            "Received error {:#?}. Video {} will be retried in 10 minutes",
            yt_video_result, result.url
        );
    }
}
