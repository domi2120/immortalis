use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use actix::prelude::*;
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web::http::header::{Charset, ContentDisposition, DispositionParam, ExtendedValue};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws::{self};
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
use serde::{Deserialize, Serialize};

use dotenvy::dotenv;
use tracing::{debug, info};
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
    web_socket_connections: Arc<RwLock<HashMap<String, Addr<ScheduledArchivalsEventHandler>>>>,
}

async fn distribute_postgres_events(app_state: web::Data<AppState>) {
    let pool = sqlx::PgPool::connect(std::env::var(env_var_names::DATABASE_URL).unwrap().as_str())
    .await
    .unwrap();

    info!("doing stuff");
    let mut listener = sqlx::postgres::PgListener::connect_with(&pool)
    .await
    .unwrap();

    listener
    .listen_all(vec![
            "scheduled_archivals"
        ])
        .await
        .unwrap();

    let mut interval = actix_web::rt::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        while let Ok(Some(notification)) = listener.try_recv().await {
            let postgres_event = serde_json::from_str::<PostgresEvent<ScheduledArchival>>(notification.payload()).unwrap();

            for con in app_state.clone().web_socket_connections.read().unwrap().iter() {
                con.1
                    .send(Message(serde_json::to_string_pretty(&postgres_event).unwrap()))
                    .await
                    .unwrap();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PostgresEvent<T> {
    action: String,
    record: T
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

    let app_state = web::Data::new(AppState {
        db_connection_pool: pool.clone(),
        file_storage_location: file_storage_location.clone(),
        web_socket_connections: Arc::new(RwLock::new(HashMap::new())),
    });

    let cloned_app_state = app_state.clone();

    actix_web::rt::spawn(async move {
        distribute_postgres_events(cloned_app_state).await;
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

    if std::env::var(env_var_names::USE_IPV6).unwrap() == "1" {
        server = server.bind("[::1]:8080")?; // can require special config in docker
    }

    server.run().await
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Clone)]
pub struct ScheduledArchivalsEventHandler {
    web_socket_connections: Arc<RwLock<HashMap<String, Addr<ScheduledArchivalsEventHandler>>>>,
}

impl Actor for ScheduledArchivalsEventHandler {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.web_socket_connections
            .write()
            .unwrap()
            .insert(Uuid::new_v4().to_string(), ctx.address());
    }
}

impl Handler<Message> for ScheduledArchivalsEventHandler {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ScheduledArchivalsEventHandler {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let resp = ws::start(
        ScheduledArchivalsEventHandler {
            web_socket_connections: app_state.web_socket_connections.clone(),
        },
        &req,
        stream,
    );
    print!("baum");
    resp
}
