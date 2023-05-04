use immortalis_backend_common::{video::Video, download::Download};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};


#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("true")
}

#[get("/search")]
async fn search() -> impl Responder {
    let test_data =vec![
        Video {
            title: "Ghost - Rats (Official Music Video)".to_string(),
            channel: "Ghost".to_string(),
            views: 5000000,
            upload_date: chrono::Utc::now(),
            archived_date: chrono::Utc::now(),
            duration: 265,
            thumbnail_address: "https://img.youtube.com/vi/C_ijc7A5oAc/maxresdefault.jpg".to_string(),
            original_url: "https://www.youtube.com/watch?v=C_ijc7A5oAc".to_string(),
            downloads: vec![ Download {
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }, Download {
                title: "Audio Only".to_string(),
                value: "Audio Only".to_string()
            }],
            selected_download: 
            Download {
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }
        },
        Video {
            title: "I Am".to_string(),
            channel: "Theocracy - Topic".to_string(),
            views: 388000,
            upload_date: chrono::Utc::now(),
            archived_date: chrono::Utc::now(),
            duration: 660,
            thumbnail_address: "https://img.youtube.com/vi/vfc8EjDuYNw/maxresdefault.jpg".to_string(),
            original_url: "https://www.youtube.com/watch?v=vfc8EjDuYNw".to_string(),
            downloads: vec![ Download {
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }, Download {
                title: "Audio Only".to_string(),
                value: "Audio Only".to_string()
            }],
            selected_download: 
            Download {
                title: "Download(1080p30)".to_string(),
                value: "Download(1080p30)".to_string()
            }
        }
    ];
    HttpResponse::Ok().json(test_data)
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