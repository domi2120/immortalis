use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use immortalis_backend_common::data_transfer_models::video_with_downloads::VideoWithDownload;
use immortalis_backend_common::database_models::{
    download::Download, scheduled_archival::ScheduledArchival, video::Video,
};
use immortalis_backend_common::schema::{scheduled_archivals, tracked_collections, videos};

use diesel::{insert_into, ExpressionMethods, GroupedBy};
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

#[get("/schedule")]
async fn get_schedules(app_state: web::Data<AppState>) -> impl Responder {
    let results = scheduled_archivals::table.into_boxed();

    let results = results
        .load::<ScheduledArchival>(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .expect("Error loading posts");

    HttpResponse::Ok().json(results)
}

#[derive(Deserialize)]
struct ScheduleRequest {
    url: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    term: Option<String>,
}

#[post("schedule")]
async fn schedule(
    schedule_request: web::Json<ScheduleRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    if schedule_request.url.is_empty() {
        return HttpResponse::BadRequest();
    }

    // for the moment, tracking is mixed into the schedule endpoint for simplicity
    if schedule_request.url.contains("channel")
        || schedule_request.url.contains('@')
        || schedule_request.url.contains("list")
    {
        insert_into(tracked_collections::table)
            .values(tracked_collections::url.eq(&schedule_request.url))
            .execute(&mut app_state.db_connection_pool.get().await.unwrap())
            .await
            .unwrap();
        return HttpResponse::Ok();
    }

    let inserted = insert_into(scheduled_archivals::table)
        .values(scheduled_archivals::url.eq(&schedule_request.url))
        .on_conflict_do_nothing()
        .execute(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .unwrap();
    println!("Scheduled {} entries", inserted);

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
    let mut file_storage_location = std::env::var("FILE_STORAGE_LOCATION")
        .expect("FILE_STORAGE_LOCATION invalid or missing")
        .to_string();
    file_storage_location = file_storage_location[..file_storage_location.len()].to_string();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_connection_pool: pool.clone(),
            }))
            .service(health)
            .service(search)
            .service(schedule)
            .service(get_schedules)
            .service(
                actix_files::Files::new("/download", &file_storage_location).show_files_listing(),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .bind("[::1]:8080")? // can require special config in docker
    .run()
    .await
}
