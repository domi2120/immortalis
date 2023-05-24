use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use actix::Addr;
use actix_web::http::header::{Charset, ContentDisposition, DispositionParam, ExtendedValue};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws::{self};
use immortalis_backend_common::data_transfer_models::video_dto::VideoDto;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::database_models::{scheduled_archival::ScheduledArchival, video::Video};
use immortalis_backend_common::env_var_config::EnvVarConfig;
use immortalis_backend_common::schema::{files, scheduled_archivals, tracked_collections, videos};

use diesel::{insert_into, ExpressionMethods, GroupedBy};
use diesel::{BelongingToDsl, PgTextExpressionMethods, QueryDsl, JoinOnDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use scheduled_archivals_event_handler::ScheduledArchivalsEventHandler;
use serde::{Deserialize, Serialize};

use dotenvy::dotenv;
use tracing::info;
use uuid::Uuid;

use crate::scheduled_archivals_event_handler::Message;
pub mod request_models;
pub mod scheduled_archivals_event_handler;
pub mod utilities;
use request_models::{GetFileRequestData, ScheduleRequest, SearchQuery};

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

#[post("schedule")]
async fn schedule(
    schedule_request: web::Json<ScheduleRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    if schedule_request.url.is_empty() {
        return HttpResponse::BadRequest();
    }

    if let Ok(video_url) = crate::utilities::filter_query_pairs(&schedule_request.url, vec!["v"]) {
        
        let db_connection = &mut app_state.db_connection_pool.get().await.unwrap();

        let already_exists = match videos::table.filter(videos::original_url.eq(&video_url)).first::<Video>(db_connection).await {
            Ok(_x) => true,
            Err(_x) => false
        };

        if already_exists {
            return HttpResponse::Ok();
        }

        let inserted = insert_into(scheduled_archivals::table)
            .values(scheduled_archivals::url.eq(&video_url))
            .on_conflict_do_nothing()
            .execute(db_connection)
            .await
            .unwrap();
        info!("Scheduled {} entries for url {}", inserted, video_url);

        HttpResponse::Ok()
    } else {
        return HttpResponse::BadRequest()
    }
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>, app_state: web::Data<AppState>) -> impl Responder {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();
    let mut results = videos::table.order(videos::archived_date.desc()).into_boxed();

    if let Some(x) = &query.term {
        results = results.filter(videos::title.ilike("%".to_string() + x + "%"))
    }

    //let g = files::table.inner_join(videos.on).load::<File, Video>(&mut conn);
    use diesel::SelectableHelper;
    let results: Vec<(Video, i64)>  = results
        .inner_join(immortalis_backend_common::schema::files::dsl::files.on(files::id.eq(videos::file_id)))
        .select((Video::as_select(), files::size))
        .load::<(Video, i64)>(&mut conn)
        .await
        .expect("Error loading posts");

    HttpResponse::Ok().json(results.into_iter().map(move |f| VideoDto {
        video: f.0,
        video_size: f.1 
    }).collect::<Vec<VideoDto>>())
}

#[get("/file")]
async fn get_file(
    req: HttpRequest,
    query: web::Query<GetFileRequestData>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();

    let f: immortalis_backend_common::database_models::file::File = files::table
        .find(query.file_id)
        .first(&mut conn)
        .await
        .unwrap();

    let response = actix_files::NamedFile::open_async(format!(
        "{}{}.{}",
        app_state.file_storage_location.to_owned(),
        &f.id.to_string(),
        &f.file_extension
    ))
    .await?;

    Ok(
        response
            .set_content_disposition(ContentDisposition {
                disposition: actix_web::http::header::DispositionType::Attachment,
                parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
                    value: format!("{}.{}", f.file_name, &f.file_extension)
                        .as_bytes()
                        .to_vec(),
                    charset: Charset::Ext("UTF-8".to_string()),
                    language_tag: None,
                })],
            })
            .into_response(&req)
    )
}

struct AppState {
    db_connection_pool: Pool<AsyncPgConnection>,
    file_storage_location: String,
    web_socket_connections: Arc<RwLock<HashMap<String, Addr<ScheduledArchivalsEventHandler>>>>,
    env_var_config: Arc<EnvVarConfig>
}

async fn distribute_postgres_events(app_state: web::Data<AppState>) {
    let pool = sqlx::PgPool::connect(&app_state.env_var_config.database_url)
        .await
        .unwrap();

    let mut listener = sqlx::postgres::PgListener::connect_with(&pool)
        .await
        .unwrap();

    listener
        .listen_all(vec!["scheduled_archivals"])
        .await
        .unwrap();

    let mut interval = actix_web::rt::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        while let Ok(Some(notification)) = listener.try_recv().await {
            let postgres_event =
                serde_json::from_str::<PostgresEvent<ScheduledArchival>>(notification.payload())
                    .unwrap();

            for con in app_state
                .clone()
                .web_socket_connections
                .read()
                .unwrap()
                .iter()
            {
                con.1
                    .send(Message(
                        serde_json::to_string_pretty(&postgres_event).unwrap(),
                    ))
                    .await
                    .unwrap();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PostgresEvent<T> {
    action: String,
    record: T,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let env_var_config = Arc::new(envy::from_env::<EnvVarConfig>().unwrap());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &env_var_config.database_url
    );

    let pool = Pool::builder(config).build().unwrap();

    let app_state = web::Data::new(AppState {
        db_connection_pool: pool.clone(),
        file_storage_location: env_var_config.file_storage_location.clone(),
        web_socket_connections: Arc::new(RwLock::new(HashMap::new())),
        env_var_config: env_var_config.clone()
    });

    let worker_app_state = app_state.clone();
    let setup_app_state = app_state.clone();

    actix_web::rt::spawn(async move {
        distribute_postgres_events(worker_app_state).await;
    });

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/ws/", web::get().to(websocket))
            .service(health)
            .service(search)
            .service(schedule)
            .service(get_schedules)
            .service(get_tracked_collection)
            .service(tracked_collection)
            .service(get_file)
    })
    .bind(("0.0.0.0", 8080))?;

    if setup_app_state.env_var_config.use_ipv6 {
        server = server.bind("[::1]:8080")?; // can require special config in docker
    }

    server.run().await
}

async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::error::Error> {
    ws::start(
        ScheduledArchivalsEventHandler::new(app_state.web_socket_connections.clone()),
        &req,
        stream,
    )
}
