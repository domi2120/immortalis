use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use actix::Addr;
use actix_http::header::{HeaderValue, CACHE_CONTROL};
use actix_web::http::header::{Charset, ContentDisposition, DispositionParam, ExtendedValue};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws::{self};
use immortalis_backend_common::data_transfer_models::video_dto::VideoDto;
use immortalis_backend_common::database_models::tracked_collection::TrackedCollection;
use immortalis_backend_common::database_models::{
    scheduled_archival::ScheduledArchival, video::Video,
};
use immortalis_backend_common::env_var_config::EnvVarConfigApi;
use immortalis_backend_common::schema::{files, scheduled_archivals, tracked_collections, videos};

use diesel::{insert_into, ExpressionMethods, SelectableHelper};
use diesel::{JoinOnDsl, PgTextExpressionMethods, QueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use immortalis_backend_common::utilities::{get_url_type, UrlType};
use serde::{Deserialize, Serialize};
use websocket_actor::WebSocketActor;

use dotenvy::dotenv;
use tracing::{error, info, warn};

use crate::websocket_actor::Message;
pub mod request_models;
pub mod utilities;
pub mod websocket_actor;
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

    match get_url_type(&schedule_request.url) {
        UrlType::VideoOrCollection | UrlType::Collection => (),
        _=> return HttpResponse::BadRequest()
    };

    let response = insert_into(tracked_collections::table)
        .values(tracked_collections::url.eq(&schedule_request.url))
        .on_conflict_do_nothing()
        .execute(&mut app_state.db_connection_pool.get().await.unwrap())
        .await
        .unwrap();
    if response > 0 {
        HttpResponse::Created()
    } else {
        HttpResponse::Ok()
    }
}

#[post("schedule")]
async fn schedule(
    schedule_request: web::Json<ScheduleRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {

    match get_url_type(&schedule_request.url) {
        UrlType::VideoOrCollection | UrlType::Video => (),
        _=> return HttpResponse::BadRequest()
    };

    // v is youtubes query param for the video, so its the only thing that we want to keep here
    if let Ok(video_url) = crate::utilities::filter_query_pairs(&schedule_request.url, vec!["v"]) {
        let db_connection = &mut app_state.db_connection_pool.get().await.unwrap();

        let already_exists = match videos::table
            .filter(videos::original_url.eq(&video_url))
            .first::<Video>(db_connection)
            .await
        {
            Ok(_x) => true,
            Err(_x) => false,
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

        HttpResponse::Created()
    } else {
        HttpResponse::BadRequest()
    }
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>, app_state: web::Data<AppState>) -> impl Responder {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();
    let mut results = videos::table
        .order(videos::archived_date.desc())
        .into_boxed();

    if let Some(x) = &query.term {
        results =
            results.filter(videos::title.ilike("%".to_string() + (x.to_owned() + "%").as_str()))
    }

    let results: Vec<(Video, i64)> = results
        .inner_join(
            immortalis_backend_common::schema::files::dsl::files.on(files::id.eq(videos::file_id)),
        )
        .select((Video::as_select(), files::size))
        .load::<(Video, i64)>(&mut conn)
        .await
        .expect("Error loading posts");

    HttpResponse::Ok().json(
        results
            .into_iter()
            .map(move |f| VideoDto {
                video: f.0,
                video_size: f.1,
            })
            .collect::<Vec<VideoDto>>(),
    )
}

#[get("/file")]
async fn get_file(
    req: HttpRequest,
    query: web::Query<GetFileRequestData>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::error::Error> {
    let mut conn = app_state.db_connection_pool.get().await.unwrap();

    let f: immortalis_backend_common::database_models::file::File = files::table
        .find(query.file_id)
        .first(&mut conn)
        .await
        .unwrap();

    // if s3 is used, redirect to a presigned link to the s3, otherwise return the file from disk
    if app_state.env_var_config.use_s3 {
        let mut custom_queries = HashMap::new();
        custom_queries.insert(
            "response-content-disposition".into(),
            format!(
                "attachment; filename=\"{}.{}\"",
                f.file_name, &f.file_extension
            ),
        );

        custom_queries.insert("cache-control".into(), format!("public, max-age={}", app_state.env_var_config.s3_file_cache_duration_seconds)); // the file downloaded from minio is cached for 7 days
        let presign = app_state
            .bucket
            .presign_get(
                format!("{}.{}", &f.id.to_string(), &f.file_extension),
                app_state.env_var_config.s3_file_cache_duration_seconds,
                Some(custom_queries),
            )
            .unwrap();

        let mut response = actix_web::web::Redirect::to(presign).respond_to(&req);
        response.headers_mut().append(CACHE_CONTROL, HeaderValue::from_str(format!("public, max-age={}", app_state.env_var_config.s3_file_cache_duration_seconds).as_str()).unwrap()); // cache the presigned link for as long as its valid (7 days, which is the maximum for s3)
        Ok(response.map_into_boxed_body())
    } else {
        let mut response = actix_files::NamedFile::open_async(format!(
            "{}{}.{}",
            app_state.file_storage_location.to_owned(),
            &f.id.to_string(),
            &f.file_extension
        ))
        .await?;
        response = response
        .set_content_disposition(ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
                value: format!("{}.{}", f.file_name, &f.file_extension)
                    .as_bytes()
                    .to_vec(),
                charset: Charset::Ext("UTF-8".to_string()),
                language_tag: None,
            })],
        });

        let response = response.customize().append_header(("cache-control", format!("public, max-age={}", app_state.env_var_config.disk_file_cache_duration_seconds))); // the file directly returned is cached for one year
        Ok(response.respond_to(&req).map_into_boxed_body())
    }
}

struct AppState {
    db_connection_pool: Pool<AsyncPgConnection>,
    file_storage_location: String,
    web_socket_connections: Arc<RwLock<HashMap<String, Addr<WebSocketActor>>>>,
    env_var_config: Arc<EnvVarConfigApi>,
    bucket: Arc<s3::Bucket>,
}

async fn distribute_postgres_events(app_state: web::Data<AppState>) {
    let pool = loop {
        match sqlx::PgPool::connect(&app_state.env_var_config.general_config.database_url).await {
            Ok(r) => break r,
            Err(e) => {
                error!("Encountered Database error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    };

    let mut listener = sqlx::postgres::PgListener::connect_with(&pool)
        .await
        .unwrap();

    listener
        .listen_all(vec!["scheduled_archivals", "tracked_collections"])
        .await
        .unwrap();

    let mut interval = actix_web::rt::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        while let Ok(Some(notification)) = listener.try_recv().await {
            match notification.channel() {
                "scheduled_archivals" => {
                    let postgres_event = serde_json::from_str::<PostgresEvent<ScheduledArchival>>(
                        notification.payload(),
                    )
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
                                serde_json::to_string_pretty(&WebSocketEvent {
                                    channel: "scheduled_archivals".to_string(),
                                    data: &postgres_event,
                                })
                                .unwrap(),
                            ))
                            .await
                            .unwrap();
                    }
                }
                "tracked_collections" => {
                    info!("tracked collections event received");
                    let postgres_event = serde_json::from_str::<PostgresEvent<TrackedCollection>>(
                        notification.payload(),
                    )
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
                                serde_json::to_string_pretty(&WebSocketEvent {
                                    channel: "tracked_collections".to_string(),
                                    data: &postgres_event,
                                })
                                .unwrap(),
                            ))
                            .await
                            .unwrap();
                    }
                }
                _ => {
                    warn!(
                        "received postgres event on channel {} without handler",
                        notification.channel()
                    )
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PostgresEvent<T> {
    action: String,
    record: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketEvent<T> {
    channel: String,
    data: T,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let env_var_config = Arc::new(envy::from_env::<EnvVarConfigApi>().unwrap());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &env_var_config.general_config.database_url,
    );

    let pool = Pool::builder(config).build().unwrap();

    let bucket = Arc::new(
        s3::Bucket::new(
            &env_var_config.storage_config.s3_bucket_name,
            s3::Region::Custom {
                region: "eu-central-1".to_owned(),
                endpoint: env_var_config.storage_config.s3_external_url.to_owned(),
            },
            s3::creds::Credentials::new(
                Some(&env_var_config.storage_config.s3_access_key),
                Some(&env_var_config.storage_config.s3_secret_key),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap()
        .with_path_style(),
    );

    let app_state = web::Data::new(AppState {
        db_connection_pool: pool.clone(),
        file_storage_location: env_var_config.storage_config.file_storage_location.clone(),
        web_socket_connections: Arc::new(RwLock::new(HashMap::new())),
        env_var_config: env_var_config.clone(),
        bucket: bucket.clone(),
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
        WebSocketActor::new(app_state.web_socket_connections.clone()),
        &req,
        stream,
    )
}
