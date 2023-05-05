use immortalis_backend_common::database_models::{video::Video, download::Download};
use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};

pub mod schema;
//use diesel::associations::HasTable;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use self::schema::videos;
use dotenvy::dotenv;

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("true")
}

#[get("/search")]
async fn search(app_state: web::Data<AppState>) -> impl Responder {

    let mut conn = app_state.db_connection_pool.get().await.unwrap();
    let results = videos::table
    .load::<Video>(&mut conn).await
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

struct AppState {
    db_connection_pool: Pool<AsyncPgConnection>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(std::env::var("DATABASE_URL").unwrap());
    let pool = Pool::builder(config).build().unwrap();

    HttpServer::new(move|| {
        App::new()
            .app_data(web::Data::new(AppState {
                db_connection_pool: pool.clone()
            }))
            .service(health)
            .service(search)
    })
    .bind(("0.0.0.0", 8080))?
    .bind("[::1]:8080")?
    .run()
    .await
}