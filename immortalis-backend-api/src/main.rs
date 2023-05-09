use actix_web::http::header::{Charset, ContentDisposition, DispositionParam, ExtendedValue};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use immortalis_backend_common::data_transfer_models::video_with_downloads::VideoWithDownload;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::database_models::{
    download::Download, scheduled_archival::ScheduledArchival, video::Video,
};
use immortalis_backend_common::env_var_names;
use immortalis_backend_common::schema::{scheduled_archivals, tracked_collections, videos};

use diesel::{insert_into, ExpressionMethods, GroupedBy};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;

use dotenvy::dotenv;
use tracing::{info, debug};
use uuid::Uuid;

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

#[get("tracked_collection")]
async fn get_tracked_collection(app_state: web::Data<AppState>) -> impl Responder {
    let results = tracked_collections::table
        .load::<TrackedCollection>(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .unwrap();
    HttpResponse::Ok().json(results)
}

#[post("tracked_collection")]
async fn tracked_collection(
    schedule_request: web::Json<ScheduleRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    insert_into(tracked_collections::table)
        .values(tracked_collections::url.eq(&schedule_request.url))
        .execute(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .unwrap();
    HttpResponse::Ok()
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

    // trim query params other than 'v' which is the video (trims for example playlists)
    let url = url::Url::parse(&schedule_request.url).unwrap();
    let view_query_param = url.query_pairs().filter(|x| x.0 == "v");
    let mut new_url = url.clone();
    new_url
        .query_pairs_mut()
        .clear()
        .extend_pairs(view_query_param);
    let video_url = new_url.to_string();

    let inserted = insert_into(scheduled_archivals::table)
        .values(scheduled_archivals::url.eq(&video_url))
        .on_conflict_do_nothing()
        .execute(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .unwrap();
    info!("Scheduled {} entries for url {}", inserted, video_url);

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

    let videos_with_downloads: Vec<VideoWithDownload> = retrieved_downloads
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

#[derive(Deserialize)]
struct GetFileRequestData {
    file_id: Uuid,
    is_thumbnail: bool,
}

#[get("/file")]
async fn get_file(
    req: HttpRequest,
    query: web::Query<GetFileRequestData>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();
    let mut table = videos::table.into_boxed();

    let file_extension: String;
    let file_name: String;
    let mut location = app_state.file_storage_location.to_owned();
    let video: Video;

    if query.is_thumbnail {
        table = table.filter(videos::thumbnail_id.eq(query.file_id));
        video = table.first::<Video>(&mut conn).await.unwrap();
        file_name = video.thumbnail_id.to_string();
        file_extension = video.thumbnail_extension.to_string();
        location += "thumbnails/";
    } else {
        table = table.filter(videos::file_id.eq(query.file_id));
        video = table.first::<Video>(&mut conn).await.unwrap();
        file_name = video.file_id.to_string();
        file_extension = video.file_extension.to_string();
    }

    debug!("{}{}.{}", location, &file_name, &file_extension);
    let response =
        actix_files::NamedFile::open_async(location + &file_name + "." + &file_extension)
            .await
            .unwrap();
    response
        .set_content_disposition(ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
                value: (video.title + "." + &file_extension).as_bytes().to_vec(),
                charset: Charset::Ext("UTF-8".to_string()),
                language_tag: None,
            })],
        })
        .into_response(&req)
}

struct AppState {
    db_connection_pool: Pool<AsyncPgConnection>,
    file_storage_location: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        std::env::var(env_var_names::DATABASE_URL).unwrap(),
    );

    let pool = Pool::builder(config).build().unwrap();
    let file_storage_location = std::env::var(env_var_names::FILE_STORAGE_LOCATION)
        .expect("FILE_STORAGE_LOCATION invalid or missing");

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_connection_pool: pool.clone(),
                file_storage_location: file_storage_location.clone(),
            }))
            .service(health)
            .service(search)
            .service(schedule)
            .service(get_schedules)
            .service(get_tracked_collection)
            .service(tracked_collection)
            .service(get_file)
    })
    .bind(("0.0.0.0", 8080))?;

    if std::env::var(env_var_names::USE_IPV6).unwrap() == "1" {
        server = server.bind("[::1]:8080")?; // can require special config in docker
    }

    server.run().await
}
