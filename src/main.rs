// src/main.rs

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
use chrono::DateTime;

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
        .route("/suggest", post(suggest_sync_point))
        .nest_service("/", ServeDir::new("static"))
        .nest_service("/output", ServeDir::new("output"))
        .layer(DefaultBodyLimit::max((1024 * 1024 * 1024)*2)); // 2 GB

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
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

#[derive(Serialize)]
struct PointJson {
    lat: f64,
    lon: f64,
    time: Option<String>,
}

#[derive(Serialize)]
struct SuggestionResponse {
    message: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timestamp: Option<String>,
    interpolated_points: Option<Vec<PointJson>>,
}

#[derive(Debug, Default)]
struct ProcessParams {
    gpx_path: Option<PathBuf>,
    video_path: Option<PathBuf>,
    sync_timestamp: Option<String>,
    add_speedo_overlay: bool,
    speedo_position: Option<String>,
    add_track_overlay: bool,
    track_position: Option<String>,
    add_stats_overlay: bool,
    stats_position: Option<String>,
    lang: String,
    interpolation_level: i64,
}

async fn process_files(mut multipart: Multipart) -> impl IntoResponse {
    let mut params = ProcessParams {
        interpolation_level: 1,
        ..Default::default()
    };

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
                params.gpx_path = Some(absolute_path);
            } else if name == "videoFile" {
                params.video_path = Some(absolute_path);
            }
        } else {
            let data = field.bytes().await.unwrap();
            let value = String::from_utf8(data.to_vec()).unwrap();
            
            match name.as_str() {
                "syncTimestamp" => params.sync_timestamp = Some(value),
                "addSpeedoOverlay" => params.add_speedo_overlay = value.parse().unwrap_or(false),
                "speedoPosition" => params.speedo_position = Some(value),
                "addTrackOverlay" => params.add_track_overlay = value.parse().unwrap_or(false),
                "trackPosition" => params.track_position = Some(value),
                "addStatsOverlay" => params.add_stats_overlay = value.parse().unwrap_or(false),
                "statsPosition" => params.stats_position = Some(value),
                "lang" => params.lang = value,
                "interpolationLevel" => params.interpolation_level = value.parse().unwrap_or(1),
                _ => {}
            }
        }
    }

    if let (Some(gpx), Some(video), Some(timestamp)) = (params.gpx_path, params.video_path, params.sync_timestamp) {
        let result = tokio::task::spawn_blocking(move || {
            processing::run_processing(
                gpx,
                video,
                timestamp,
                params.add_speedo_overlay,
                params.speedo_position.unwrap_or_default(),
                params.add_track_overlay,
                params.track_position.unwrap_or_default(),
                params.add_stats_overlay,
                params.stats_position.unwrap_or_default(),
                params.lang,
                params.interpolation_level,
            )
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
            message: "Erro: Ficheiros ou ponto de sincronização em falta.".to_string(),
            download_url: None,
            logs: vec![],
        };
        (StatusCode::BAD_REQUEST, Json(response))
    }
}

async fn suggest_sync_point(mut multipart: Multipart) -> impl IntoResponse {
    let mut gpx_path: Option<PathBuf> = None;
    let mut video_path: Option<PathBuf> = None;
    let mut interpolation_level: i64 = 1;

    let upload_dir = PathBuf::from("uploads_temp_suggest");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().map(|s| s.to_string());

        if let Some(file_name_str) = file_name {
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name_str));
            tokio::fs::write(&path, &data).await.unwrap();
            
            if name == "gpxFile" { gpx_path = Some(path); }
            else if name == "videoFile" { video_path = Some(path); }
        } else {
            let data = field.bytes().await.unwrap();
            let value = String::from_utf8(data.to_vec()).unwrap();
            if name == "interpolationLevel" {
                interpolation_level = value.parse().unwrap_or(1);
            }
        }
    }

    let response = if let (Some(gpx_p), Some(video_p)) = (gpx_path, video_path) {
        match utils::get_video_time_range(&video_p, "en") {
            Ok((video_start_time, _)) => {
                match gpx::read(std::io::BufReader::new(std::fs::File::open(&gpx_p).unwrap())) {
                    Ok(gpx_data) => {
                        let interpolated_gpx = utils::interpolate_gpx_points(gpx_data, interpolation_level);
                        
                        let first_point_after = interpolated_gpx
                            .tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter())
                            .find(|p| p.time.and_then(|t| t.format().ok()).and_then(|ts| ts.parse::<DateTime<chrono::Utc>>().ok()).map_or(false, |pt| pt > video_start_time));

                        let points_for_json: Vec<PointJson> = interpolated_gpx.tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter()).map(|p| PointJson { lat: p.point().y(), lon: p.point().x(), time: p.time.and_then(|t| t.format().ok()) }).collect();

                        if let Some(point) = first_point_after {
                            let point_coords = point.point();
                            let timestamp_str = point.time.and_then(|t| t.format().ok()).unwrap();
                            Json(SuggestionResponse { message: "Sync point suggested.".to_string(), latitude: Some(point_coords.y()), longitude: Some(point_coords.x()), timestamp: Some(timestamp_str), interpolated_points: Some(points_for_json) })
                        } else {
                            Json(SuggestionResponse { message: "No GPX point found after video start.".to_string(), latitude: None, longitude: None, timestamp: None, interpolated_points: Some(points_for_json) })
                        }
                    },
                    Err(_) => Json(SuggestionResponse { message: "Error reading GPX file.".to_string(), latitude: None, longitude: None, timestamp: None, interpolated_points: None }),
                }
            },
            Err(e) => Json(SuggestionResponse { message: format!("Error reading video metadata: {}", e), latitude: None, longitude: None, timestamp: None, interpolated_points: None }),
        }
    } else {
        Json(SuggestionResponse { message: "Missing video or GPX file.".to_string(), latitude: None, longitude: None, timestamp: None, interpolated_points: None })
    };
    
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    (StatusCode::OK, response)
}