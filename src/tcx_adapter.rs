// src/tcx_adapter.rs - Módulo otimizado para leitura de TCX e conversão para GPX

use std::error::Error;
use std::path::Path;
use chrono::{DateTime, Utc};
use gpx::{Gpx, Track, TrackSegment, Waypoint, GpxVersion};
use geo_types::Point;
use tcx;

/// Estrutura para armazenar dados extras específicos do TCX
#[derive(Debug, Default, Clone)]
pub struct TcxExtraData {
    pub sport: Option<String>,
    pub total_time_seconds: f64,
    pub total_distance_meters: f64,
    pub total_calories: f64,
    pub max_speed: f64,
    pub heart_rate_data: Vec<f64>,
    pub cadence_data: Vec<f64>,
    pub speed_data: Vec<f64>,
}

/// Estrutura para o resultado do processamento de um arquivo TCX
pub struct TcxProcessResult {
    pub gpx: Gpx,
    pub extra_data: TcxExtraData,
}

/// Lê e processa um arquivo TCX uma única vez, retornando a estrutura GPX e os dados extras.
/// Esta função unificada melhora a eficiência ao evitar a leitura dupla do arquivo.
pub fn read_and_process_tcx(path: &Path) -> Result<TcxProcessResult, Box<dyn Error>> {
    let path_str = path.to_str().ok_or("Invalid path encoding")?;
    let tcx_data = tcx::read_file(path_str)?;

    // Estruturas para armazenar os dados processados
    let mut gpx = Gpx {
        version: GpxVersion::Gpx11,
        creator: Some("GPX Video Sync - TCX Adapter".to_string()),
        ..Default::default()
    };
    let mut extra_data = TcxExtraData::default();

    if let Some(activities) = tcx_data.activities {
        for activity in activities.activities {
            extra_data.sport = Some(activity.sport.clone());
            
            let mut track = Track::new();
            track.name = Some(activity.id.clone());
            track.type_ = Some(map_sport_to_track_type(&activity.sport));

            for lap in activity.laps {
                // Acumula estatísticas para extra_data
                extra_data.total_time_seconds += lap.total_time_seconds;
                extra_data.total_distance_meters += lap.distance_meters;
                extra_data.total_calories += lap.calories as f64;
                if let Some(max_speed) = lap.maximum_speed {
                    if max_speed > extra_data.max_speed {
                        extra_data.max_speed = max_speed;
                    }
                }

                let mut segment = TrackSegment::new();
                for track_data in lap.tracks {
                    for trackpoint in track_data.trackpoints {
                        if let Some(position) = trackpoint.position {
                            let mut waypoint = Waypoint::new(Point::new(
                                position.longitude,
                                position.latitude,
                            ));

                            if let Some(altitude) = trackpoint.altitude_meters {
                                waypoint.elevation = Some(altitude);
                            }

                            let time_str = trackpoint.time.to_rfc3339();
                            if let Ok(time_parsed) = DateTime::parse_from_rfc3339(&time_str) {
                                let utc_time = time_parsed.with_timezone(&Utc);
                                if let Ok(offset_dt) = time::OffsetDateTime::from_unix_timestamp(utc_time.timestamp()) {
                                    waypoint.time = Some(gpx::Time::from(offset_dt));
                                }
                            }

                            // --- CORREÇÃO: Armazena dados de telemetria no campo `comment` ---
                            let mut comment_parts = Vec::new();
                            
                            if let Some(heart_rate) = trackpoint.heart_rate {
                                let hr_value = heart_rate.value as f64;
                                extra_data.heart_rate_data.push(hr_value);
                                comment_parts.push(format!("HR:{}", hr_value));
                            }
                            
                            if let Some(cadence) = trackpoint.cadence {
                                let cad_value = cadence as f64;
                                extra_data.cadence_data.push(cad_value);
                                comment_parts.push(format!("CAD:{}", cad_value));
                            }
                            
                            if let Some(extensions_data) = &trackpoint.extensions {
                                if let Some(tpx) = &extensions_data.tpx {
                                    if let Some(speed) = tpx.speed {
                                        extra_data.speed_data.push(speed);
                                        comment_parts.push(format!("SPD:{:.2}", speed));
                                    }
                                }
                            }

                            if !comment_parts.is_empty() {
                                waypoint.comment = Some(comment_parts.join(";"));
                            }
                            // --- FIM DA CORREÇÃO ---

                            segment.points.push(waypoint);
                        }
                    }
                }
                if !segment.points.is_empty() {
                    track.segments.push(segment);
                }
            }
            if !track.segments.is_empty() {
                gpx.tracks.push(track);
            }
        }
    }

    Ok(TcxProcessResult { gpx, extra_data })
}

/// Mapeia tipos de esporte TCX para tipos de trilha GPX
fn map_sport_to_track_type(sport: &str) -> String {
    match sport.to_lowercase().as_str() {
        "running" => "Running".to_string(),
        "biking" | "cycling" => "Cycling".to_string(),
        "walking" => "Walking".to_string(),
        "hiking" => "Hiking".to_string(),
        "swimming" => "Swimming".to_string(),
        _ => sport.to_string(),
    }
}

impl TcxExtraData {
    pub fn average_heart_rate(&self) -> Option<f64> {
        if self.heart_rate_data.is_empty() { None } 
        else { Some(self.heart_rate_data.iter().sum::<f64>() / self.heart_rate_data.len() as f64) }
    }
    
    pub fn average_cadence(&self) -> Option<f64> {
        if self.cadence_data.is_empty() { None } 
        else { Some(self.cadence_data.iter().sum::<f64>() / self.cadence_data.len() as f64) }
    }
    
    pub fn average_speed(&self) -> Option<f64> {
        if self.speed_data.is_empty() { None } 
        else { Some(self.speed_data.iter().sum::<f64>() / self.speed_data.len() as f64) }
    }
    
    pub fn max_heart_rate(&self) -> Option<f64> {
        self.heart_rate_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
    
    pub fn max_cadence(&self) -> Option<f64> {
        self.cadence_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
}
