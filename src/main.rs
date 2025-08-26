// src/main.rs

mod drawing;
mod processing;
mod utils;
mod tcx_adapter;
mod fit_adapter;
mod strava_integration;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Query, Path},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::tcx_adapter::TcxExtraData;
use crate::strava_integration::{StravaClient, StravaConfig, StravaSession, StravaActivity};

// Estrutura para unificar os dados lidos do arquivo de trilha
pub struct TrackFileData {
    pub gpx: gpx::Gpx,
    pub extra_data: Option<TcxExtraData>,
}

fn get_strava_config() -> Option<StravaConfig> {
    let client_id = std::env::var("STRAVA_CLIENT_ID").ok()?;
    let client_secret = std::env::var("STRAVA_CLIENT_SECRET").ok()?;
    let redirect_uri = std::env::var("STRAVA_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3030/strava/callback".to_string());

    Some(StravaConfig {
        client_id,
        client_secret,
        redirect_uri,
    })
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
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
        .route("/strava/auth", get(strava_auth))
        .route("/strava/callback", get(strava_callback))
        .route("/strava/activities", get(strava_activities))
        .route("/strava/download/:activity_id", post(strava_download_activity))
        .route("/strava/status", get(strava_status))
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

#[derive(Serialize)]
struct StravaAuthResponse {
    auth_url: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct StravaCallbackQuery {
    code: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct StravaCallbackResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct StravaActivitiesResponse {
    success: bool,
    activities: Option<Vec<StravaActivity>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct StravaStatusResponse {
    authenticated: bool,
    athlete_id: Option<u64>,
    token_valid: bool,
}

#[derive(Deserialize)]
struct StravaDownloadRequest {
    format: String,
}

use std::sync::{Arc, Mutex};
use std::collections::HashMap as StdHashMap;

lazy_static::lazy_static! {
    static ref STRAVA_SESSIONS: Arc<Mutex<StdHashMap<String, StravaSession>>> = 
        Arc::new(Mutex::new(StdHashMap::new()));
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

pub fn read_track_file(path: &PathBuf) -> Result<TrackFileData, Box<dyn std::error::Error + Send + Sync>> {
    let file_type = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    
    match file_type.as_str() {
        "fit" => {
            let result = fit_adapter::read_and_process_fit(path)?;
            Ok(TrackFileData {
                gpx: result.gpx,
                extra_data: Some(result.extra_data.to_tcx_extra_data()),
            })
        },
        "tcx" => {
            let result = tcx_adapter::read_and_process_tcx(path)?;
            Ok(TrackFileData {
                gpx: result.gpx,
                extra_data: Some(result.extra_data),
            })
        },
        "gpx" => {
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

async fn strava_auth() -> Response {
    match get_strava_config() {
        Some(config) => {
            let client = StravaClient::new(config);
            let auth_data = client.get_auth_url(&["read_all", "activity:read_all"]);
            
            Json(StravaAuthResponse {
                auth_url: Some(auth_data.auth_url),
                error: None,
            }).into_response()
        },
        None => Json(StravaAuthResponse {
            auth_url: None,
            error: Some("Strava não configurado.".to_string()),
        }).into_response(),
    }
}

async fn strava_callback(Query(params): Query<StravaCallbackQuery>) -> Response {
    if let Some(error) = params.error {
        return Json(StravaCallbackResponse {
            success: false,
            message: format!("Erro na autenticação: {}", error),
        }).into_response();
    }

    let code = match params.code {
        Some(c) => c,
        None => return Json(StravaCallbackResponse {
            success: false,
            message: "Código de autorização não fornecido".to_string(),
        }).into_response(),
    };

    match get_strava_config() {
        Some(config) => {
            let client = StravaClient::new(config);
            match client.exchange_token(&code).await {
                Ok(token_response) => {
                    let session_id = Uuid::new_v4().to_string();
                    let session = StravaSession::new(token_response);
                    
                    STRAVA_SESSIONS.lock().unwrap().insert(session_id.clone(), session);
                    
                    Json(StravaCallbackResponse {
                        success: true,
                        message: format!("Autenticação realizada com sucesso! Session ID: {}", session_id),
                    }).into_response()
                },
                Err(e) => Json(StravaCallbackResponse {
                    success: false,
                    message: format!("Erro ao trocar token: {}", e),
                }).into_response(),
            }
        },
        None => Json(StravaCallbackResponse {
            success: false,
            message: "Strava não configurado".to_string(),
        }).into_response(),
    }
}

async fn strava_activities(Query(params): Query<HashMap<String, String>>) -> Response {
    let session_id = match params.get("session_id") {
        Some(id) => id,
        None => return Json(StravaActivitiesResponse {
            success: false,
            activities: None,
            error: Some("Session ID não fornecido".to_string()),
        }).into_response(),
    };

    let config = match get_strava_config() {
        Some(c) => c,
        None => return Json(StravaActivitiesResponse {
            success: false,
            activities: None,
            error: Some("Strava não configurado".to_string()),
        }).into_response(),
    };

    let client = StravaClient::new(config);
    let mut session = {
        let sessions = STRAVA_SESSIONS.lock().unwrap();
        match sessions.get(session_id).cloned() {
            Some(s) => s,
            None => return Json(StravaActivitiesResponse {
                success: false,
                activities: None,
                error: Some("Sessão não encontrada".to_string()),
            }).into_response(),
        }
    };

    if let Err(e) = session.refresh_if_needed(&client).await {
        return Json(StravaActivitiesResponse {
            success: false,
            activities: None,
            error: Some(format!("Erro ao atualizar token: {}", e)),
        }).into_response();
    }

    let per_page = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(30);
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);

    match client.list_activities(&session.access_token, Some(per_page), Some(page), None, None).await {
        Ok(activities) => {
            STRAVA_SESSIONS.lock().unwrap().insert(session_id.clone(), session);
            Json(StravaActivitiesResponse {
                success: true,
                activities: Some(activities),
                error: None,
            }).into_response()
        },
        Err(e) => Json(StravaActivitiesResponse {
            success: false,
            activities: None,
            error: Some(format!("Erro ao buscar atividades: {}", e)),
        }).into_response(),
    }
}

#[axum::debug_handler]
async fn strava_download_activity(
    Path(activity_id): Path<u64>,
    Query(params): Query<HashMap<String, String>>,
    Json(request): Json<StravaDownloadRequest>,
) -> Response {
    let session_id = match params.get("session_id") {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, "Session ID não fornecido").into_response();
        }
    };

    let config = match get_strava_config() {
        Some(c) => c,
        None => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Strava não configurado").into_response();
        }
    };

    let client = StravaClient::new(config);
    let mut session = {
        let sessions = STRAVA_SESSIONS.lock().unwrap();
        match sessions.get(session_id).cloned() {
            Some(s) => s,
            None => {
                return (StatusCode::UNAUTHORIZED, "Sessão não encontrada").into_response();
            }
        }
    };

    if let Err(e) = session.refresh_if_needed(&client).await {
        return (
            StatusCode::UNAUTHORIZED,
            format!("Erro ao atualizar token: {}", e),
        )
            .into_response();
    }

    let download_result = match request.format.as_str() {
        "fit" => client.download_activity_original(&session.access_token, activity_id).await,
        "tcx" => client.download_activity_tcx(&session.access_token, activity_id).await,
        "gpx" => client.download_activity_gpx(&session.access_token, activity_id).await,
        _ => {
            return (StatusCode::BAD_REQUEST, "Formato não suportado").into_response();
        }
    };

    let file_bytes = match download_result {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao baixar atividade: {}", e),
            )
                .into_response();
        }
    };
    
    let upload_dir = PathBuf::from("uploads_strava");
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
         return (StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao criar diretório: {}", e)).into_response();
    }
    
    let temp_file_path = upload_dir.join(format!("strava_{}_{}.{}", activity_id, Uuid::new_v4(), &request.format));
    
    if let Err(e) = tokio::fs::write(&temp_file_path, &file_bytes).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao salvar arquivo: {}", e)).into_response();
    }

    let track_file_data = match read_track_file(&temp_file_path) {
        Ok(data) => data,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao processar arquivo: {}", e)).into_response();
        }
    };

    let _ = tokio::fs::remove_file(&temp_file_path).await;
    STRAVA_SESSIONS.lock().unwrap().insert(session_id.to_string(), session);

    let interpolated_gpx = utils::interpolate_gpx_points(track_file_data.gpx, 1);

    let points_for_json: Vec<PointJson> = interpolated_gpx.tracks.iter()
        .flat_map(|t| t.segments.iter())
        .flat_map(|s| s.points.iter())
        .map(|p| {
            let (heart_rate, cadence, speed) = processing::extract_telemetry_from_waypoint(p);
            PointJson {
                lat: p.point().y(),
                lon: p.point().x(),
                time: p.time.as_ref().and_then(|t| t.format().ok()).map(|t| t.to_string()),
                heart_rate,
                cadence,
                speed,
            }
        })
        .collect();

    let first_point = points_for_json.first();

    let (extra_data_json, sport_type) = if let Some(extra) = track_file_data.extra_data {
        let json = TcxExtraDataJson {
            total_time_seconds: extra.total_time_seconds,
            total_distance_meters: extra.total_distance_meters,
            total_calories: extra.total_calories,
            max_speed: extra.max_speed,
            average_heart_rate: extra.average_heart_rate(),
            max_heart_rate: extra.max_heart_rate(),
            average_cadence: extra.average_cadence(),
            max_cadence: extra.max_cadence(),
            average_speed: extra.average_speed(),
        };
        (Some(json), extra.sport)
    } else {
        (None, None)
    };

    let response = if let Some(point) = first_point {
        SuggestionResponse {
            message: "Atividade do Strava importada com sucesso!".to_string(),
            latitude: Some(point.lat),
            longitude: Some(point.lon),
            timestamp: point.time.clone(),
            display_timestamp: point.time.as_ref().and_then(|t| t.parse::<DateTime<Utc>>().ok()).map(|utc_time| {
                let brt_offset = chrono::FixedOffset::west_opt(3 * 3600).unwrap();
                let local_time = utc_time.with_timezone(&brt_offset);
                format!("{} (-03:00)", local_time.format("%d/%m/%Y, %H:%M:%S"))
            }),
            interpolated_points: Some(points_for_json),
            file_type: Some(request.format.to_uppercase()),
            sport_type,
            extra_data: extra_data_json,
        }
    } else {
        SuggestionResponse {
            message: "Atividade importada, mas não contém dados de localização".to_string(),
            latitude: None, longitude: None, timestamp: None, display_timestamp: None,
            interpolated_points: Some(points_for_json),
            file_type: Some(request.format.to_uppercase()),
            sport_type,
            extra_data: extra_data_json,
        }
    };

    Json(response).into_response()
}

async fn strava_status(Query(params): Query<HashMap<String, String>>) -> Response {
    let session_id = match params.get("session_id") {
        Some(id) => id,
        None => return Json(StravaStatusResponse {
            authenticated: false,
            athlete_id: None,
            token_valid: false,
        }).into_response(),
    };

    let sessions = STRAVA_SESSIONS.lock().unwrap();
    if let Some(session) = sessions.get(session_id) {
        Json(StravaStatusResponse {
            authenticated: true,
            athlete_id: session.athlete_id,
            token_valid: !session.is_expired(),
        }).into_response()
    } else {
        Json(StravaStatusResponse {
            authenticated: false,
            athlete_id: None,
            token_valid: false,
        }).into_response()
    }
}

async fn process_files(mut multipart: Multipart) -> Response {
    let mut params = ProcessParams::default();

    let upload_dir = PathBuf::from("uploads");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        
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
                (StatusCode::OK, Json(response)).into_response()
            }
            Err((err_msg, logs)) => {
                let response = ProcessResponse {
                    message: err_msg,
                    download_url: None,
                    logs,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
    } else {
        let response = ProcessResponse {
            message: "Erro: Arquivos ou ponto de sincronização em falta.".to_string(),
            download_url: None,
            logs: vec![],
        };
        (StatusCode::BAD_REQUEST, Json(response)).into_response()
    }
}

async fn suggest_sync_point(mut multipart: Multipart) -> Response {
    let mut track_file_path: Option<PathBuf> = None;
    let mut video_path: Option<PathBuf> = None;
    let mut interpolation_level: i64 = 1;

    let upload_dir = PathBuf::from("uploads_temp_suggest");
    tokio::fs::create_dir_all(&upload_dir).await.unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        
        if let Some(file_name_str) = field.file_name().map(|s| s.to_string()) {
            let data = field.bytes().await.unwrap();
            let unique_id = Uuid::new_v4();
            let path = upload_dir.join(format!("{}-{}", unique_id, file_name_str));
            tokio::fs::write(&path, &data).await.unwrap();
            
            if name == "gpxFile" { track_file_path = Some(path); }
            else if name == "videoFile" { video_path = Some(path); }
        } else {
            let data = field.bytes().await.unwrap();
            if let Ok(value) = String::from_utf8(data.to_vec()) {
                if name == "interpolationLevel" {
                    interpolation_level = value.parse().unwrap_or(1);
                }
            }
        }
    }

    let result = if let (Some(track_p), Some(video_p)) = (track_file_path, video_path) {
        match utils::get_video_time_range(&video_p, "en") {
            Ok((video_start_time, _)) => {
                match read_track_file(&track_p) {
                    Ok(track_file_data) => {
                        let file_type = track_p.extension().and_then(|s| s.to_str()).unwrap_or("").to_uppercase();
                        let interpolated_gpx = utils::interpolate_gpx_points(track_file_data.gpx, interpolation_level);
                        
                        let first_point_after = interpolated_gpx
                            .tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter())
                            .find(|p| {
                                if let Some(time) = p.time.as_ref().and_then(|t| t.format().ok()) {
                                    if let Ok(parsed_time) = time.parse::<DateTime<Utc>>() {
                                        return parsed_time > video_start_time;
                                    }
                                }
                                false
                            });

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
                                    time: p.time.as_ref().and_then(|t| t.format().ok()).map(|t| t.to_string()),
                                    heart_rate,
                                    cadence,
                                    speed,
                                }
                            })
                            .collect();

                        if let Some(point) = first_point_after {
                            let point_coords = point.point();
                            let timestamp_iso_str = point.time.as_ref().and_then(|t| t.format().ok()).unwrap_or_default();

                            let display_timestamp_str = if let Ok(utc_time) = timestamp_iso_str.parse::<DateTime<Utc>>() {
                                let brt_offset = chrono::FixedOffset::west_opt(3 * 3600).unwrap();
                                let local_time = utc_time.with_timezone(&brt_offset);
                                format!("{} (-03:00)", local_time.format("%d/%m/%Y, %H:%M:%S"))
                            } else {
                                timestamp_iso_str.clone()
                            };

                            (StatusCode::OK, Json(SuggestionResponse {
                                message: "Sync point suggested.".to_string(),
                                latitude: Some(point_coords.y()),
                                longitude: Some(point_coords.x()),
                                timestamp: Some(timestamp_iso_str),
                                display_timestamp: Some(display_timestamp_str),
                                interpolated_points: Some(points_for_json),
                                file_type: Some(file_type),
                                sport_type,
                                extra_data: extra_data_json,
                            }))
                        } else {
                            (StatusCode::OK, Json(SuggestionResponse { 
                                message: "No track point found after video start.".to_string(), 
                                latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                                interpolated_points: Some(points_for_json),
                                file_type: Some(file_type),
                                sport_type,
                                extra_data: extra_data_json,
                            }))
                        }
                    },
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SuggestionResponse { 
                        message: format!("Error reading track file: {}", e), 
                        latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                        interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
                    })),
                }
            },
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(SuggestionResponse { 
                message: format!("Error reading video metadata: {}", e), 
                latitude: None, longitude: None, timestamp: None, display_timestamp: None,
                interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
            })),
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(SuggestionResponse { 
            message: "Missing video or track file.".to_string(), 
            latitude: None, longitude: None, timestamp: None, display_timestamp: None,
            interpolated_points: None, file_type: None, sport_type: None, extra_data: None,
        }))
    };
    
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    result.into_response()
}