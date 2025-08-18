// src/tcx_adapter.rs - Módulo para conversão de TCX para GPX

use std::error::Error;
use std::path::Path;
use chrono::{DateTime, Utc};
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use geo_types::Point;
use tcx;

/// Converte um arquivo TCX para formato GPX
/// Mantém compatibilidade total com o código existente
pub fn read_tcx_as_gpx(path: &Path) -> Result<Gpx, Box<dyn Error>> {
    // CORRIGIDO: Converte Path para &str
    let path_str = path.to_str().ok_or("Invalid path encoding")?;
    let tcx_data = tcx::read_file(path_str)?;
    
    // Cria uma estrutura GPX vazia
    let mut gpx = Gpx::default();
    gpx.version = gpx::GpxVersion::Gpx11;
    gpx.creator = Some("GPX Video Sync - TCX Adapter".to_string());
    
    // Processa as atividades do TCX
    if let Some(activities) = tcx_data.activities {
        for activity in activities.activities {
            let mut track = Track::new();
            
            // CORRIGIDO: track.name espera Option<String>
            track.name = Some(activity.id.clone());
            track.type_ = Some(map_sport_to_track_type(&activity.sport));
            
            // Processa as voltas (laps) da atividade
            for lap in activity.laps {
                let mut segment = TrackSegment::new();
                
                // Processa as trilhas dentro de cada volta
                for track_data in lap.tracks {
                    for trackpoint in track_data.trackpoints {
                        if let Some(position) = trackpoint.position {
                            // CORRIGIDO: Campos corretos latitude/longitude (não *_degrees)
                            let mut waypoint = Waypoint::new(Point::new(
                                position.longitude,
                                position.latitude,
                            ));
                            
                            // Adiciona elevação se disponível
                            if let Some(altitude) = trackpoint.altitude_meters {
                                waypoint.elevation = Some(altitude);
                            }
                            
                            // CORRIGIDO: trackpoint.time é DateTime<Utc> direto, não Option
                            let time_str = trackpoint.time.to_rfc3339();
                            if let Ok(time_parsed) = DateTime::parse_from_rfc3339(&time_str) {
                                let utc_time = time_parsed.with_timezone(&Utc);
                                // Converte para o formato de tempo do GPX
                                if let Ok(offset_dt) = time::OffsetDateTime::from_unix_timestamp(utc_time.timestamp()) {
                                    waypoint.time = Some(gpx::Time::from(offset_dt));
                                }
                            }
                            
                            // Adiciona extensões específicas do TCX como comentários
                            let mut extensions = Vec::new();
                            
                            // CORRIGIDO: heart_rate é estrutura HeartRate, não número direto
                            if let Some(heart_rate) = trackpoint.heart_rate {
                                extensions.push(format!("HR:{}", heart_rate.value));
                            }
                            
                            if let Some(cadence) = trackpoint.cadence {
                                extensions.push(format!("Cadence:{}", cadence));
                            }
                            
                            // CORRIGIDO: extensions.tpx.speed em vez de extensions.speed
                            if let Some(extensions_data) = &trackpoint.extensions {
                                if let Some(tpx) = &extensions_data.tpx {
                                    if let Some(speed) = tpx.speed {
                                        extensions.push(format!("Speed:{:.2}", speed));
                                    }
                                }
                            }
                            
                            if !extensions.is_empty() {
                                waypoint.comment = Some(extensions.join(";"));
                            }
                            
                            segment.points.push(waypoint);
                        }
                    }
                }
                
                // Adiciona o segmento ao track se tiver pontos
                if !segment.points.is_empty() {
                    track.segments.push(segment);
                }
            }
            
            // Adiciona o track ao GPX se tiver segmentos
            if !track.segments.is_empty() {
                gpx.tracks.push(track);
            }
        }
    }
    
    Ok(gpx)
}

/// Mapeia tipos de esporte TCX para tipos de trilha GPX
fn map_sport_to_track_type(sport: &str) -> String {
    match sport.to_lowercase().as_str() {
        "running" => "Running".to_string(),
        "biking" | "cycling" => "Cycling".to_string(),
        "walking" => "Walking".to_string(),
        "hiking" => "Hiking".to_string(),
        "swimming" => "Swimming".to_string(),
        _ => sport.to_string(), // Mantém o original se não reconhecido
    }
}

/// Função auxiliar para extrair dados adicionais do TCX que não existem no GPX
/// Retorna informações extra que podem ser usadas para melhorar a telemetria
pub fn extract_tcx_extra_data(path: &Path) -> Result<TcxExtraData, Box<dyn Error>> {
    // CORRIGIDO: Converte Path para &str
    let path_str = path.to_str().ok_or("Invalid path encoding")?;
    let tcx_data = tcx::read_file(path_str)?;
    let mut extra_data = TcxExtraData::default();
    
    if let Some(activities) = tcx_data.activities {
        for activity in activities.activities {
            extra_data.sport = Some(activity.sport.clone());
            
            // Coleta estatísticas das voltas
            for lap in activity.laps {
                // CORRIGIDO: Acessar campos diretamente como f64, não Option<f64>
                extra_data.total_time_seconds += lap.total_time_seconds;
                extra_data.total_distance_meters += lap.distance_meters;
                
                // CORRIGIDO: calories é u16, não Option<u16>
                extra_data.total_calories += lap.calories as f64;
                
                // CORRIGIDO: verificar se maximum_speed existe e comparar
                if let Some(max_speed) = lap.maximum_speed {
                    if max_speed > extra_data.max_speed {
                        extra_data.max_speed = max_speed;
                    }
                }
                
                // Processa dados dos trackpoints para estatísticas detalhadas
                for track_data in lap.tracks {
                    for trackpoint in track_data.trackpoints {
                        // CORRIGIDO: heart_rate.value para obter o número
                        if let Some(hr) = trackpoint.heart_rate {
                            extra_data.heart_rate_data.push(hr.value as f64);
                        }
                        
                        if let Some(cadence) = trackpoint.cadence {
                            extra_data.cadence_data.push(cadence as f64);
                        }
                        
                        // CORRIGIDO: extensions.tpx.speed
                        if let Some(extensions_data) = &trackpoint.extensions {
                            if let Some(tpx) = &extensions_data.tpx {
                                if let Some(speed) = tpx.speed {
                                    extra_data.speed_data.push(speed);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(extra_data)
}

/// Estrutura para armazenar dados extras específicos do TCX
#[derive(Debug, Default)]
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

impl TcxExtraData {
    /// Calcula a frequência cardíaca média
    pub fn average_heart_rate(&self) -> Option<f64> {
        if self.heart_rate_data.is_empty() {
            None
        } else {
            Some(self.heart_rate_data.iter().sum::<f64>() / self.heart_rate_data.len() as f64)
        }
    }
    
    /// Calcula a cadência média
    pub fn average_cadence(&self) -> Option<f64> {
        if self.cadence_data.is_empty() {
            None
        } else {
            Some(self.cadence_data.iter().sum::<f64>() / self.cadence_data.len() as f64)
        }
    }
    
    /// Calcula a velocidade média
    pub fn average_speed(&self) -> Option<f64> {
        if self.speed_data.is_empty() {
            None
        } else {
            Some(self.speed_data.iter().sum::<f64>() / self.speed_data.len() as f64)
        }
    }
    
    /// Retorna a frequência cardíaca máxima
    pub fn max_heart_rate(&self) -> Option<f64> {
        self.heart_rate_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
    
    /// Retorna a cadência máxima
    pub fn max_cadence(&self) -> Option<f64> {
        self.cadence_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }
}