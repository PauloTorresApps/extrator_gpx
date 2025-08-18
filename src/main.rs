// src/main.rs

mod drawing;
mod processing;
mod utils;
mod tcx_adapter; // NOVO: Módulo para suporte TCX

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
    // NOVOS: Campos extras do TCX
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
    // NOVOS: Dados extras do TCX
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

/// Função auxiliar para detectar o tipo de arquivo baseado na extensão
fn detect_file_type(path: &PathBuf) -> String {
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap_or("").to_lowercase().as_str() {
            "tcx" => "TCX".to_string(),
            "gpx" => "GPX".to_string(),
            _ => "Unknown".to_string(),
        }
    } else {
        "Unknown".to_string()
    }
}

/// Função para ler arquivo de trilha (GPX ou TCX) e converter para GPX
fn read_track_file(path: &PathBuf) -> Result<gpx::Gpx, Box<dyn std::error::Error>> {
    let file_type = detect_file_type(path);
    
    match file_type.as_str() {
        "TCX" => {
            // Lê TCX e converte para GPX
            tcx_adapter::read_tcx_as_gpx(path)
        },
        "GPX" => {
            // Lê GPX normalmente
            use std::io::BufReader;
            use std::fs::File;
            Ok(gpx::read(BufReader::new(File::open(path)?))?)
        },
        _ => {
            Err(format!("Formato de arquivo não suportado: {}", file_type).into())
        }
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
        
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name));
            tokio::fs::write(&path, &data).await.unwrap();
            let absolute_path = std::fs::canonicalize(&path).unwrap();

            // MODIFICADO: Aceita tanto GPX quanto TCX
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
            message: "Erro: Arquivos ou ponto de sincronização em falta.".to_string(),
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
            
            // MODIFICADO: Aceita tanto GPX quanto TCX
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
                // MODIFICADO: Usa a nova função para ler tanto GPX quanto TCX
                match read_track_file(&gpx_p) {
                    Ok(gpx_data) => {
                        let file_type = detect_file_type(&gpx_p);
                        let interpolated_gpx = utils::interpolate_gpx_points(gpx_data, interpolation_level);
                        
                        let first_point_after = interpolated_gpx
                            .tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter())
                            .find(|p| p.time.and_then(|t| t.format().ok()).and_then(|ts| ts.parse::<DateTime<chrono::Utc>>().ok()).map_or(false, |pt| pt > video_start_time));

                        // MODIFICADO: Extrai dados extras se for TCX
                        let (extra_data, sport_type) = if file_type == "TCX" {
                            match tcx_adapter::extract_tcx_extra_data(&gpx_p) {
                                Ok(tcx_extra) => {
                                    let extra_json = TcxExtraDataJson {
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
                                    (Some(extra_json), tcx_extra.sport)
                                },
                                Err(_) => (None, None)
                            }
                        } else {
                            (None, None)
                        };

                        // MODIFICADO: Extrai dados extras dos pontos se disponível
                        let points_for_json: Vec<PointJson> = interpolated_gpx.tracks.iter()
                            .flat_map(|t| t.segments.iter())
                            .flat_map(|s| s.points.iter())
                            .map(|p| {
                                // Tenta extrair dados extras do comentário (onde armazenamos dados TCX)
                                let (heart_rate, cadence, speed) = if let Some(comment) = &p.comment {
                                    let mut hr = None;
                                    let mut cad = None;
                                    let mut spd = None;
                                    
                                    for part in comment.split(';') {
                                        if part.starts_with("HR:") {
                                            hr = part[3..].parse().ok();
                                        } else if part.starts_with("Cadence:") {
                                            cad = part[8..].parse().ok();
                                        } else if part.starts_with("Speed:") {
                                            spd = part[6..].parse().ok();
                                        }
                                    }
                                    
                                    (hr, cad, spd)
                                } else {
                                    (None, None, None)
                                };

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
                                extra_data,
                            })
                        } else {
                            Json(SuggestionResponse { 
                                message: "No track point found after video start.".to_string(), 
                                latitude: None, 
                                longitude: None, 
                                timestamp: None, 
                                display_timestamp: None,
                                interpolated_points: Some(points_for_json),
                                file_type: Some(file_type),
                                sport_type,
                                extra_data,
                            })
                        }
                    },
                    Err(e) => Json(SuggestionResponse { 
                        message: format!("Error reading track file: {}", e), 
                        latitude: None, 
                        longitude: None, 
                        timestamp: None, 
                        display_timestamp: None,
                        interpolated_points: None,
                        file_type: None,
                        sport_type: None,
                        extra_data: None,
                    }),
                }
            },
            Err(e) => Json(SuggestionResponse { 
                message: format!("Error reading video metadata: {}", e), 
                latitude: None, 
                longitude: None, 
                timestamp: None, 
                display_timestamp: None,
                interpolated_points: None,
                file_type: None,
                sport_type: None,
                extra_data: None,
            }),
        }
    } else {
        Json(SuggestionResponse { 
            message: "Missing video or track file.".to_string(), 
            latitude: None, 
            longitude: None, 
            timestamp: None, 
            display_timestamp: None,
            interpolated_points: None,
            file_type: None,
            sport_type: None,
            extra_data: None,
        })
    };
    
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    (StatusCode::OK, response)
}