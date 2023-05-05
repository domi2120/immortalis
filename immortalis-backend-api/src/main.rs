//use immortalis_backend_common::{video::Video, download::Download};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub mod schema;
pub mod video;
pub mod download;

use video::Video;
use download::Download;

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("true")
}

#[get("/search")]
async fn search() -> impl Responder {


    use self::schema::videos::dsl::*;
    use self::schema::videodownloads::dsl::*;

    let connection = &mut establish_connection();
    let results = videos
    .limit(5)
    .load::<Video>(connection)
    .expect("Error loading posts");

    let test_data =vec![
        Video {
            id: 4,
            title: "Ghost - Rats (Official Music Video)".to_string(),
            channel: "Ghost".to_string(),
            views: 5000000,
            upload_date: chrono::Utc::now().naive_utc(),
            archived_date: chrono::Utc::now().naive_utc(),
            duration: 265,
            thumbnail_address: "https://img.youtube.com/vi/C_ijc7A5oAc/maxresdefault.jpg".to_string(),
            original_url: "https://www.youtube.com/watch?v=C_ijc7A5oAc".to_string(),
/*
            downloads: vec![ Download {
                id: 5,
                video_id: 5,
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }, Download {
                id: 6,
                video_id: 5,
                title: "Audio Only".to_string(),
                value: "Audio Only".to_string()
            }],
            selected_download: 
            Download {
                id: 5,
                video_id: 5,
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }
*/
        },
        Video {
            id: 5,
            title: "I Am".to_string(),
            channel: "Theocracy - Topic".to_string(),
            views: 388000,
            upload_date: chrono::Utc::now().naive_utc(),
            archived_date: chrono::Utc::now().naive_utc(),
            duration: 660,
            thumbnail_address: "https://img.youtube.com/vi/vfc8EjDuYNw/maxresdefault.jpg".to_string(),
            original_url: "https://www.youtube.com/watch?v=vfc8EjDuYNw".to_string(),
 /*http://localhost:5050/browser/
            downloads: vec![ Download {
                id: 5,
                video_id: 5,
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }, Download {
                id: 5,
                video_id: 5,
                title: "Audio Only".to_string(),
                value: "Audio Only".to_string()
            }],
            selected_download: 
            Download {
                id: 5,
                video_id: 5,
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }
        */
        }
    ];
    HttpResponse::Ok().json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(search)
    })
    .bind(("0.0.0.0", 8080))?
    .bind("[::1]:8080")?
    .run()
    .await
}

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}