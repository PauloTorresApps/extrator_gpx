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
    // Inicializa o sistema de logs para o terminal
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "extrator_gpx=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Define as rotas da aplicação
    let app = Router::new()
        .route("/process", post(process_files))
        // Serve a pasta 'static' que contém o nosso frontend (index.html)
        .nest_service("/", ServeDir::new("static"))
        // Serve a pasta 'output' para que o vídeo final possa ser descarregado
        .nest_service("/output", ServeDir::new("output"))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)); // Aumenta o limite de upload para 1GB

    // Inicia o servidor na porta 3000
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

// Handler que recebe os ficheiros e inicia o processamento
async fn process_files(mut multipart: Multipart) -> impl IntoResponse {
    let mut gpx_path: Option<PathBuf> = None;
    let mut video_path: Option<PathBuf> = None;
    let mut sync_timestamp: Option<String> = None;

    // Cria uma pasta para os uploads temporários
    let upload_dir = PathBuf::from("uploads");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    // Processa os ficheiros enviados
    // CORREÇÃO: Remove o 'mut' desnecessário
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name));
            tokio::fs::write(&path, &data).await.unwrap();

            if name == "gpxFile" {
                gpx_path = Some(path);
            } else if name == "videoFile" {
                video_path = Some(path);
            }
        } else if name == "syncTimestamp" {
            let data = field.bytes().await.unwrap();
            sync_timestamp = Some(String::from_utf8(data.to_vec()).unwrap());
        }
    }

    if let (Some(gpx), Some(video), Some(timestamp)) = (gpx_path, video_path, sync_timestamp) {
        match processing::run_processing(gpx.clone(), video.clone(), timestamp) {
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
            message: "Erro: Ficheiros ou ponto de sincronização em falta.".to_string(),
            download_url: None,
            logs: vec![],
        };
        (StatusCode::BAD_REQUEST, Json(response))
    }
}
