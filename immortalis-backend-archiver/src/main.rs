use std::{collections::HashMap, fs, path::PathBuf, borrow::Cow};
use diesel::{GroupedBy, insert_into, ExpressionMethods, delete};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use immortalis_backend_common::database_models::scheduled_archival::ScheduledArchival;
use immortalis_backend_common::schema::scheduled_archivals;
use tokio::time::Duration;
use dotenvy::dotenv;
use async_process::Command;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var("DATABASE_URL").unwrap(),
    );
    let pool = Pool::builder(config).build().unwrap();

    let mut interval_timer = tokio::time::interval(Duration::from_secs(5));
    loop {
        interval_timer.tick().await;
        let pool_instance = pool.clone();
        tokio::spawn(async move { test(pool_instance).await; });
    }
    
}

async fn test(pool: Pool<AsyncPgConnection>) {
    let mut db_connection = &mut pool.get().await.unwrap();
    let mut results = scheduled_archivals::table.limit(1).load::<ScheduledArchival>(&mut db_connection).await.unwrap();
    if results.len() == 0 {
        return;
    }
    println!("Archiving entry: {} with url: {}", results[0].id, results[0].url);
    
    delete(scheduled_archivals::table).filter(scheduled_archivals::id.eq(results[0].id)).execute(&mut db_connection).await.unwrap();
    println!("Archived and deleted entry: {} with url: {}", results[0].id, results[0].url);
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