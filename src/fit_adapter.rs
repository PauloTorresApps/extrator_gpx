// src/fit_adapter.rs - Adaptador para arquivos FIT do Garmin/Strava

use std::error::Error;
use std::path::Path;
use crate::tcx_adapter::TcxExtraData;

/// Estrutura para o resultado do processamento de um arquivo FIT
pub struct FitProcessResult {
    pub gpx: gpx::Gpx,
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
pub fn read_and_process_fit(_path: &Path) -> Result<FitProcessResult, Box<dyn Error>> {
    // Por enquanto, retornar erro informativo até implementar o parser FIT completo
    Err("Suporte FIT ainda não implementado. Use GPX ou TCX por enquanto.".into())
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