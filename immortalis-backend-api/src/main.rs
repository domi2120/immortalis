use immortalis_backend_common::data_transfer_models::video_with_downloads::VideoWithDownload;
use immortalis_backend_common::database_models::{video::Video, download::Download};
use immortalis_backend_common::schema::{videos, downloads};
use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use diesel::BelongingToDsl;
use diesel::GroupedBy;

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

    let retrieved_downloads = Download::belonging_to(&results)
        .load::<Download>(&mut conn).await.unwrap();

    let videos_with_downloads = retrieved_downloads
        .grouped_by(&results)
        .into_iter()
        .zip(results)
        .map(|(dl, vid)| VideoWithDownload{downloads: dl, video: vid} )
        .collect::<Vec<VideoWithDownload>>();
    
    HttpResponse::Ok().json(videos_with_downloads)
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