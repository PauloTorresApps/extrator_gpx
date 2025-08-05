// Declara os novos módulos que farão parte do projeto.
mod drawing;
mod processing;
mod utils;

use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "extrator_gpx=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/process", post(process_files))
        .nest_service("/", ServeDir::new("static"))
        .nest_service("/output", ServeDir::new("output"))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)); // 1 GB

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("A escutar em {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct ProcessResponse {
    message: String,
    download_url: Option<String>,
    logs: Vec<String>,
}

async fn process_files(mut multipart: Multipart) -> impl IntoResponse {
    let mut gpx_path: Option<PathBuf> = None;
    let mut video_path: Option<PathBuf> = None;
    let mut sync_timestamp: Option<String> = None;
    let mut overlay_position: Option<String> = None; // NOVO

    let upload_dir = PathBuf::from("uploads");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name));
            tokio::fs::write(&path, &data).await.unwrap();
            let absolute_path = std::fs::canonicalize(&path).unwrap();

            if name == "gpxFile" {
                gpx_path = Some(absolute_path);
            } else if name == "videoFile" {
                video_path = Some(absolute_path);
            }
        } else {
            let data = field.bytes().await.unwrap();
            let value = String::from_utf8(data.to_vec()).unwrap();
            if name == "syncTimestamp" {
                sync_timestamp = Some(value);
            } else if name == "overlayPosition" { // NOVO
                overlay_position = Some(value);
            }
        }
    }

    if let (Some(gpx), Some(video), Some(timestamp), Some(position)) = (gpx_path, video_path, sync_timestamp, overlay_position) {
        let result = tokio::task::spawn_blocking(move || {
            processing::run_processing(gpx, video, timestamp, position)
        }).await.unwrap();

        match result {
            Ok(logs) => {
                let output_filename = "output_video.mp4";
                let response = ProcessResponse {
                    message: "Processamento concluído com sucesso!".to_string(),
                    download_url: Some(format!("/output/{}", output_filename)),
                    logs,
                };
                (StatusCode::OK, Json(response))
            }
            Err((err_msg, logs)) => {
                let response = ProcessResponse {
                    message: err_msg,
                    download_url: None,
                    logs,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }
        }
    } else {
        let response = ProcessResponse {
            message: "Erro: Ficheiros, ponto de sincronização ou posição em falta.".to_string(),
            download_url: None,
            logs: vec![],
        };
        (StatusCode::BAD_REQUEST, Json(response))
    }
}
