// src/main.rs

mod drawing;
mod processing;
mod utils;
mod tcx_adapter;

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
use crate::tcx_adapter::TcxExtraData;

// Estrutura para unificar os dados lidos do arquivo de trilha
struct TrackFileData {
    gpx: gpx::Gpx,
    extra_data: Option<TcxExtraData>,
}


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
    // Campos extras para telemetria
    heart_rate: Option<f64>,
    cadence: Option<f64>,
    speed: Option<f64>,
}

#[derive(Serialize)]
struct SuggestionResponse {
    message: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timestamp: Option<String>,
    display_timestamp: Option<String>,
    interpolated_points: Option<Vec<PointJson>>,
    file_type: Option<String>,
    sport_type: Option<String>,
    extra_data: Option<TcxExtraDataJson>,
}

#[derive(Serialize)]
struct TcxExtraDataJson {
    total_time_seconds: f64,
    total_distance_meters: f64,
    total_calories: f64,
    max_speed: f64,
    average_heart_rate: Option<f64>,
    max_heart_rate: Option<f64>,
    average_cadence: Option<f64>,
    max_cadence: Option<f64>,
    average_speed: Option<f64>,
}

#[derive(Debug, Default)]
struct ProcessParams {
    track_file_path: Option<PathBuf>,
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

/// Função auxiliar para detectar o tipo de arquivo baseado na extensão
fn detect_file_type(path: &PathBuf) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .map(|ext| match ext.as_str() {
            "tcx" => "TCX".to_string(),
            "gpx" => "GPX".to_string(),
            _ => "Unknown".to_string(),
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Função para ler arquivo de trilha (GPX ou TCX) e retornar dados unificados
fn read_track_file(path: &PathBuf) -> Result<TrackFileData, Box<dyn std::error::Error>> {
    let file_type = detect_file_type(path);
    
    match file_type.as_str() {
        "TCX" => {
            let result = tcx_adapter::read_and_process_tcx(path)?;
            Ok(TrackFileData {
                gpx: result.gpx,
                extra_data: Some(result.extra_data),
            })
        },
        "GPX" => {
            use std::io::BufReader;
            use std::fs::File;
            let gpx_data = gpx::read(BufReader::new(File::open(path)?))?;
            Ok(TrackFileData {
                gpx: gpx_data,
                extra_data: None,
            })
        },
        _ => Err(format!("Formato de arquivo não suportado: {}", file_type).into()),
    }
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
        
        // --- CORREÇÃO: Clona o file_name para evitar erro de borrow ---
        if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name));
            tokio::fs::write(&path, &data).await.unwrap();
            let absolute_path = std::fs::canonicalize(&path).unwrap();

            if name == "gpxFile" {
                params.track_file_path = Some(absolute_path);
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

    if let (Some(track_file), Some(video), Some(timestamp)) = (params.track_file_path, params.video_path, params.sync_timestamp) {
        let result = tokio::task::spawn_blocking(move || {
            processing::run_processing(
                track_file,
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
            message: "Erro: Arquivos ou ponto de sincronização em falta.".to_string(),
            download_url: None,
            logs: vec![],
        };
        (StatusCode::BAD_REQUEST, Json(response))
    }
}

async fn suggest_sync_point(mut multipart: Multipart) -> impl IntoResponse {
    let mut track_file_path: Option<PathBuf> = None;
    let mut video_path: Option<PathBuf> = None;
    let mut interpolation_level: i64 = 1;

    let upload_dir = PathBuf::from("uploads_temp_suggest");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        
        // --- CORREÇÃO: Clona o file_name para evitar erro de borrow ---
        if let Some(file_name_str) = field.file_name().map(|s| s.to_string()) {
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name_str));
            tokio::fs::write(&path, &data).await.unwrap();
            
            if name == "gpxFile" { track_file_path = Some(path); }
            else if name == "videoFile" { video_path = Some(path); }
        } else {
            let data = field.bytes().await.unwrap();
            let value = String::from_utf8(data.to_vec()).unwrap();
            if name == "interpolationLevel" {
                interpolation_level = value.parse().unwrap_or(1);
            }
        }
    }

    let response = if let (Some(track_p), Some(video_p)) = (track_file_path, video_path) {
        match utils::get_video_time_range(&video_p, "en") {
            Ok((video_start_time, _)) => {
                match read_track_file(&track_p) {
                    Ok(track_file_data) => {
                        let file_type = detect_file_type(&track_p);
                        let interpolated_gpx = utils::interpolate_gpx_points(track_file_data.gpx, interpolation_level);
                        
                        let first_point_after = interpolated_gpx
                            .tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter())
                            .find(|p| p.time.and_then(|t| t.format().ok()).and_then(|ts| ts.parse::<DateTime<chrono::Utc>>().ok()).map_or(false, |pt| pt > video_start_time));

                        let (extra_data_json, sport_type) = if let Some(tcx_extra) = track_file_data.extra_data {
                            let json = TcxExtraDataJson {
                                total_time_seconds: tcx_extra.total_time_seconds,
                                total_distance_meters: tcx_extra.total_distance_meters,
                                total_calories: tcx_extra.total_calories,
                                max_speed: tcx_extra.max_speed,
                                average_heart_rate: tcx_extra.average_heart_rate(),
                                max_heart_rate: tcx_extra.max_heart_rate(),
                                average_cadence: tcx_extra.average_cadence(),
                                max_cadence: tcx_extra.max_cadence(),
                                average_speed: tcx_extra.average_speed(),
                            };
                            (Some(json), tcx_extra.sport)
                        } else {
                            (None, None)
                        };

                        let points_for_json: Vec<PointJson> = interpolated_gpx.tracks.iter()
                            .flat_map(|t| t.segments.iter())
                            .flat_map(|s| s.points.iter())
                            .map(|p| {
                                let (heart_rate, cadence, speed) = processing::extract_telemetry_from_waypoint(p);
                                PointJson {
                                    lat: p.point().y(),
                                    lon: p.point().x(),
                                    time: p.time.and_then(|t| t.format().ok()),
                                    heart_rate,
                                    cadence,
                                    speed,
                                }
                            })
                            .collect();

                        if let Some(point) = first_point_after {
                            let point_coords = point.point();
                            let timestamp_iso_str = point.time.and_then(|t| t.format().ok()).unwrap();

                            let display_timestamp_str = if let Ok(utc_time) = timestamp_iso_str.parse::<DateTime<chrono::Utc>>() {
                                let brt_offset = chrono::FixedOffset::west_opt(3 * 3600).unwrap();
                                let local_time = utc_time.with_timezone(&brt_offset);
                                format!("{} (-03:00)", local_time.format("%d/%m/%Y, %H:%M:%S"))
                            } else {
                                timestamp_iso_str.clone()
                            };

                            Json(SuggestionResponse {
                                message: "Sync point suggested.".to_string(),
                                latitude: Some(point_coords.y()),
                                longitude: Some(point_coords.x()),
                                timestamp: Some(timestamp_iso_str),
                                display_timestamp: Some(display_timestamp_str),
                                interpolated_points: Some(points_for_json),
                                file_type: Some(file_type),
                                sport_type,
                                extra_data: extra_data_json,
                            })
                        } else {
                            Json(SuggestionResponse { 
                                message: "No track point found after video start.".to_string(), 
                                latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                                interpolated_points: Some(points_for_json),
                                file_type: Some(file_type),
                                sport_type,
                                extra_data: extra_data_json,
                            })
                        }
                    },
                    Err(e) => Json(SuggestionResponse { 
                        message: format!("Error reading track file: {}", e), 
                        latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                        interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
                    }),
                }
            },
            Err(e) => Json(SuggestionResponse { 
                message: format!("Error reading video metadata: {}", e), 
                latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
            }),
        }
    } else {
        Json(SuggestionResponse { 
            message: "Missing video or track file.".to_string(), 
            latitude: None, longitude: None, timestamp: None, display_timestamp: None,
            interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
        })
    };
    
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    (StatusCode::OK, response)
}
