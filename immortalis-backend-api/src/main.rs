use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, post};
use immortalis_backend_common::data_transfer_models::video_with_downloads::VideoWithDownload;
use immortalis_backend_common::database_models::{download::Download, video::Video};
use immortalis_backend_common::schema::{videos, scheduled_archivals};

use diesel::{GroupedBy, insert_into, ExpressionMethods};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;

use dotenvy::dotenv;

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("true")
}

#[derive(Deserialize)]
struct ScheduleRequest {
    url: String
}

#[derive(Deserialize)]
struct SearchQuery {
    term: Option<String>,
}

#[post("schedule")]
async fn schedule(schedule_request: web::Json<ScheduleRequest>,app_state: web::Data<AppState>) -> impl Responder {
    insert_into(scheduled_archivals::table).values(scheduled_archivals::url.eq(&schedule_request.url)).execute(&mut app_state.db_connection_pool.get().await.unwrap()).await.unwrap();
    HttpResponse::Ok()
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>, app_state: web::Data<AppState>) -> impl Responder {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();
    let mut results = videos::table.into_boxed();

    if let Some(x) = &query.term {
        results = results.filter(videos::title.ilike("%".to_string() + x + "%"))
    }

    let results = results
        .load::<Video>(&mut conn)
        .await
        .expect("Error loading posts");

    let retrieved_downloads = Download::belonging_to(&results)
        .load::<Download>(&mut conn)
        .await
        .unwrap();

    let videos_with_downloads = retrieved_downloads
        .grouped_by(&results)
        .into_iter()
        .zip(results)
        .map(|(dl, vid)| VideoWithDownload {
            downloads: dl,
            video: vid,
        })
        .collect::<Vec<VideoWithDownload>>();

    HttpResponse::Ok().json(videos_with_downloads)
}

struct AppState {
    db_connection_pool: Pool<AsyncPgConnection>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var("DATABASE_URL").unwrap(),
    );
    let pool = Pool::builder(config).build().unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_connection_pool: pool.clone(),
            }))
            .service(health)
            .service(search)
            .service(schedule)
    })
    .bind(("0.0.0.0", 8080))?
    .bind("[::1]:8080")?
    .run()
    .await
}
