// src/fit_adapter.rs - Adaptador para arquivos FIT do Garmin/Strava

use std::error::Error;
use std::path::Path;
use chrono::{DateTime, Utc};
use gpx::{Gpx, Track, TrackSegment, Waypoint, GpxVersion};
use geo_types::Point;
use fitparser::{FitFile, FitDataRecord};
use crate::tcx_adapter::TcxExtraData;

/// Estrutura para o resultado do processamento de um arquivo FIT
pub struct FitProcessResult {
    pub gpx: Gpx,
    pub extra_data: FitExtraData,
}

/// Estrutura para armazenar dados extras específicos do FIT
#[derive(Debug, Default, Clone)]
pub struct FitExtraData {
    pub sport: Option<String>,
    pub device_name: Option<String>,
    pub manufacturer: Option<String>,
    pub total_time_seconds: f64,
    pub total_distance_meters: f64,
    pub total_calories: f64,
    pub max_speed: f64,
    pub heart_rate_data: Vec<f64>,
    pub cadence_data: Vec<f64>,
    pub speed_data: Vec<f64>,
    pub power_data: Vec<f64>,
    pub temperature_data: Vec<f64>,
}

/// Lê e processa um arquivo FIT, convertendo para GPX com dados extras
pub fn read_and_process_fit(path: &Path) -> Result<FitProcessResult, Box<dyn Error>> {
    let path_str = path.to_str().ok_or("Invalid path encoding")?;
    let fit_file = FitFile::new(path_str)?;

    let mut gpx = Gpx {
        version: GpxVersion::Gpx11,
        creator: Some("GPX Video Sync - FIT Adapter".to_string()),
        ..Default::default()
    };

    let mut extra_data = FitExtraData::default();
    let mut track = Track::new();
    let mut segment = TrackSegment::new();

    // Processar registros do arquivo FIT
    for record in fit_file.records() {
        match record {
            Ok(FitDataRecord::FileId(file_id)) => {
                extra_data.manufacturer = file_id.manufacturer.map(|m| format!("{:?}", m));
            },
            Ok(FitDataRecord::DeviceInfo(device_info)) => {
                if let Some(product_name) = device_info.product_name {
                    extra_data.device_name = Some(product_name);
                }
            },
            Ok(FitDataRecord::Sport(sport)) => {
                extra_data.sport = Some(format!("{:?}", sport.sport));
            },
            Ok(FitDataRecord::Session(session)) => {
                if let Some(total_elapsed_time) = session.total_elapsed_time {
                    extra_data.total_time_seconds = total_elapsed_time as f64;
                }
                if let Some(total_distance) = session.total_distance {
                    extra_data.total_distance_meters = total_distance as f64;
                }
                if let Some(total_calories) = session.total_calories {
                    extra_data.total_calories = total_calories as f64;
                }
                if let Some(max_speed) = session.max_speed {
                    extra_data.max_speed = max_speed as f64;
                }
            },
            Ok(FitDataRecord::Record(record)) => {
                // Processar pontos de trilha
                if let (Some(lat), Some(lon)) = (record.position_lat, record.position_long) {
                    let lat_degrees = (lat as f64) * (180.0 / 2_147_483_648.0);
                    let lon_degrees = (lon as f64) * (180.0 / 2_147_483_648.0);
                    
                    let mut waypoint = Waypoint::new(Point::new(lon_degrees, lat_degrees));
                    
                    // Adicionar elevação se disponível
                    if let Some(altitude) = record.altitude {
                        waypoint.elevation = Some((altitude as f64) / 5.0 - 500.0); // Conversão do FIT
                    }
                    
                    // Adicionar timestamp se disponível
                    if let Some(timestamp) = record.timestamp {
                        let datetime = fit_timestamp_to_utc(timestamp);
                        if let Ok(offset_dt) = time::OffsetDateTime::from_unix_timestamp(datetime.timestamp()) {
                            waypoint.time = Some(gpx::Time::from(offset_dt));
                        }
                    }
                    
                    // Processar dados de telemetria e armazenar no campo comment
                    let mut comment_parts = Vec::new();
                    
                    if let Some(heart_rate) = record.heart_rate {
                        let hr_value = heart_rate as f64;
                        extra_data.heart_rate_data.push(hr_value);
                        comment_parts.push(format!("HR:{}", hr_value));
                    }
                    
                    if let Some(cadence) = record.cadence {
                        let cad_value = cadence as f64;
                        extra_data.cadence_data.push(cad_value);
                        comment_parts.push(format!("CAD:{}", cad_value));
                    }
                    
                    if let Some(speed) = record.speed {
                        let speed_value = speed as f64;
                        extra_data.speed_data.push(speed_value);
                        comment_parts.push(format!("SPD:{:.2}", speed_value));
                    }
                    
                    if let Some(power) = record.power {
                        let power_value = power as f64;
                        extra_data.power_data.push(power_value);
                        comment_parts.push(format!("PWR:{}", power_value));
                    }
                    
                    if let Some(temperature) = record.temperature {
                        let temp_value = temperature as f64;
                        extra_data.temperature_data.push(temp_value);
                        comment_parts.push(format!("TEMP:{}", temp_value));
                    }
                    
                    if !comment_parts.is_empty() {
                        waypoint.comment = Some(comment_parts.join(";"));
                    }
                    
                    segment.points.push(waypoint);
                }
            },
            _ => {
                // Ignorar outros tipos de registros por enquanto
            }
        }
    }

    // Configurar trilha
    if !segment.points.is_empty() {
        track.segments.push(segment);
        track.name = Some(format!("FIT Activity - {}", 
            extra_data.device_name.as_deref().unwrap_or("Unknown Device")));
        track.type_ = Some(map_fit_sport_to_track_type(
            extra_data.sport.as_deref().unwrap_or("Unknown")));
        gpx.tracks.push(track);
    }

    Ok(FitProcessResult { gpx, extra_data })
}

/// Converte timestamp do FIT (segundos desde 31/12/1989) para UTC
fn fit_timestamp_to_utc(fit_timestamp: u32) -> DateTime<Utc> {
    // FIT usa epoch de 31/12/1989 00:00:00 UTC
    let fit_epoch = DateTime::parse_from_rfc3339("1989-12-31T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    
    fit_epoch + chrono::Duration::seconds(fit_timestamp as i64)
}

/// Mapeia tipos de esporte FIT para tipos de trilha GPX
fn map_fit_sport_to_track_type(sport: &str) -> String {
    match sport.to_lowercase().as_str() {
        "running" | "trail_running" | "track_running" => "Running".to_string(),
        "cycling" | "road_cycling" | "mountain_biking" | "cyclocross" => "Cycling".to_string(),
        "walking" | "casual_walking" => "Walking".to_string(),
        "hiking" | "mountaineering" => "Hiking".to_string(),
        "swimming" | "pool_swimming" | "open_water_swimming" => "Swimming".to_string(),
        _ => sport.to_string(),
    }
}

impl FitExtraData {
    /// Converte para TcxExtraData para compatibilidade com o sistema existente
    pub fn to_tcx_extra_data(&self) -> TcxExtraData {
        TcxExtraData {
            sport: self.sport.clone(),
            total_time_seconds: self.total_time_seconds,
            total_distance_meters: self.total_distance_meters,
            total_calories: self.total_calories,
            max_speed: self.max_speed,
            heart_rate_data: self.heart_rate_data.clone(),
            cadence_data: self.cadence_data.clone(),
            speed_data: self.speed_data.clone(),
        }
    }
    
    pub fn average_heart_rate(&self) -> Option<f64> {
        if self.heart_rate_data.is_empty() { 
            None 
        } else { 
            Some(self.heart_rate_data.iter().sum::<f64>() / self.heart_rate_data.len() as f64) 
        }
    }
    
    pub fn average_cadence(&self) -> Option<f64> {
        if self.cadence_data.is_empty() { 
            None 
        } else { 
            Some(self.cadence_data.iter().sum::<f64>() / self.cadence_data.len() as f64) 
        }
    }
    
    pub fn average_speed(&self) -> Option<f64> {
        if self.speed_data.is_empty() { 
            None 
        } else { 
            Some(self.speed_data.iter().sum::<f64>() / self.speed_data.len() as f64) 
        }
    }
    
    pub fn average_power(&self) -> Option<f64> {
        if self.power_data.is_empty() { 
            None 
        } else { 
            Some(self.power_data.iter().sum::<f64>() / self.power_data.len() as f64) 
        }
    }
    
    pub fn max_heart_rate(&self) -> Option<f64> {
        self.heart_rate_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
    
    pub fn max_cadence(&self) -> Option<f64> {
        self.cadence_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
    
    pub fn max_power(&self) -> Option<f64> {
        self.power_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
}