use std::{collections::HashMap, fs, path::PathBuf, borrow::Cow};
use chrono::Duration;
use diesel::associations::HasTable;
use diesel::query_dsl::methods::LockingDsl;
use diesel::{GroupedBy, insert_into, ExpressionMethods, delete, update};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::database_models::video::{Video, InsertableVideo};
use immortalis_backend_common::database_models::video_status::VideoStatus;
use immortalis_backend_common::schema::{scheduled_archivals, videos};
use dotenvy::dotenv;
use async_process::Command;

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
        tokio::spawn(async move { test(pool_instance).await; });
    }
    
}

async fn test(pool: Pool<AsyncPgConnection>) {
    let mut db_connection = &mut pool.get().await.unwrap();
    
    let results = scheduled_archivals::table.limit(1).filter(scheduled_archivals::not_before.lt(chrono::Utc::now().naive_utc())).load::<ScheduledArchival>(&mut db_connection).await.unwrap();
    
    if results.len() == 0 {
        return;
    }

    let result = &results[0];

    let yt_video_result = YoutubeDl::new(&result.url).run_async().await;
    if let Ok(video) = yt_video_result {
        let video = video
        .into_single_video()
        .unwrap();
    
        delete(scheduled_archivals::table).filter(scheduled_archivals::id.eq(result.id)).execute(&mut db_connection).await.unwrap();
        println!("Dequeued entry: {} with url: {}", result.id, result.url);

    /*
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
            + "%(title)s"
        )
        .arg("--no-simulate")
        .output();

        cmd.await.unwrap();
        */

        let uploadDate = video.upload_date.unwrap();
        //println!("{:#?}", video.upload_date.unwrap());
        let video = InsertableVideo {
            title: video.title,
            channel: video.channel.unwrap(),
            views: video.view_count.unwrap(),
            upload_date: chrono::NaiveDateTime::new(chrono::NaiveDate::from_ymd_opt(uploadDate[0..=3].parse::<i32>().unwrap(), uploadDate[4..=5].parse::<u32>().unwrap(), uploadDate[6..=7].parse::<u32>().unwrap()).unwrap(), chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()),
            archived_date: chrono::Utc::now().naive_utc(),
            duration: i32::try_from(video.duration.unwrap().as_i64().unwrap()).unwrap(),
            thumbnail_address: video.thumbnail.unwrap(),
            original_url: result.url.clone(),
            status: immortalis_backend_common::database_models::video_status::VideoStatus::BeingArchived,
        };

        insert_into(videos::table).values(video).execute(&mut db_connection).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        update(videos::table).set(videos::status.eq(VideoStatus::Archived)).execute(&mut db_connection).await.unwrap();
    } else {
        // try again in 10 minutes
        update(scheduled_archivals::table).set(scheduled_archivals::not_before.eq(chrono::Utc::now().naive_utc().checked_add_signed(Duration::minutes(10)).unwrap())).execute(&mut db_connection).await.unwrap();
        println!("Received error {:#?}. Video {} will be retried in 10 minutes", yt_video_result, result.url);
    }


}
/*
async fn download() {
    let redis_hostname = std::env::var("REDIS_HOSTNAME").expect("Error, REDIS_HOSTNAME not set");
    let client =
    redis::Client::open(format!("redis://{}/", redis_hostname)).unwrap_or_else(|error| {
        panic!(
            "REDIS_HOSTNAME: {}\n ErrorCategory {}\n ErrorDetails {}",
            redis_hostname,
            error.category(),
            error.detail().unwrap_or("")
        )
    });

    let manager = ConnectionManager::new(client)
        .await
        .expect("Error, Redis could not be reached");
    
    
    let entry: Result<String, RedisError> = manager.clone().lpop("scheduled", None).await;

    match entry {
        Ok(url) =>  {

            let mut index = get_index(manager.clone()).await;
            let index_entry = index.existing.get_mut(&url).unwrap();
            index_entry.status = common::models::DownloadStatus::Downloading;
            let _: String = manager.clone().set("index", serde_json::to_string_pretty(&index).unwrap()).await.unwrap();


            let cmd = Command::new("yt-dlp")
            .arg(&url)
            .arg("-o")
            .arg(
                std::env::var(common::env_vars::FILE_STORAGE_LOCATION)
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
                std::env::var(common::env_vars::FILE_STORAGE_LOCATION)
                .expect("FILE_STORAGE_LOCATION invalid or missing")
                + "%(title)s"
            )
            .arg("--no-simulate")
            .output();
        
            
            index.existing.insert(url.clone(), common::models::Download { url: url.clone(), status: common::models::DownloadStatus::Downloading});

            let _: Option<String> = manager.clone().set("index", serde_json::to_string_pretty(&index).unwrap()).await.unwrap();

            let result = cmd.await.unwrap();

            let mut index = get_index(manager.clone()).await;
            let index_entry = index.existing.get_mut(&url).unwrap();

            if result.status.success() {
                index_entry.status = common::models::DownloadStatus::Saved;
            } else {
                index_entry.status = common::models::DownloadStatus::Failed;
            }
            let _: String = manager.clone().set("index", serde_json::to_string_pretty(&index).unwrap()).await.unwrap();

            let mut fileName = String::from_utf8(result.stdout).unwrap();
            _ = fileName.pop(); // remove \n at the end
            fileName += ".mkv";
            let filepath = PathBuf::from(&fileName).as_os_str().to_str().unwrap().to_string();
            print!("{:}", fileName);
            let file = fs::read(&filepath).unwrap();
   
            //let key = Aes256Gcm::generate_key(&mut OsRng);
            let baum: Vec<u8> = std::env::var(common::env_vars::ENCRYPTION_KEY_AES256).unwrap().bytes().collect();
            let key = Key::<Aes256Gcm>::from_slice(&baum);


            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
            let ciphertext = cipher.encrypt(nonce, file.as_ref()).unwrap();

            fs::write(filepath.to_owned() + ".encrypted", ciphertext).unwrap();

            let encryptedFile = fs::read(filepath.to_owned() + ".encrypted").unwrap();
            let decryptedFile = cipher.decrypt(nonce, encryptedFile.as_ref()).unwrap();
            fs::write(filepath.to_owned() + "decrypted", decryptedFile).unwrap();
        },
        Err(_) => (),
    }
}*/