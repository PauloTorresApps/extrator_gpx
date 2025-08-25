// src/strava_integration.rs - Integração com API do Strava

use std::error::Error;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::io::StreamReader;
use futures_util::StreamExt;

const STRAVA_API_BASE: &str = "https://www.strava.com/api/v3";
const STRAVA_AUTH_URL: &str = "https://www.strava.com/oauth/authorize";
const STRAVA_TOKEN_URL: &str = "https://www.strava.com/oauth/token";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StravaTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StravaActivity {
    pub id: u64,
    pub name: String,
    pub start_date: String,
    pub start_date_local: String,
    pub distance: f64,
    pub moving_time: i32,
    pub total_elevation_gain: f64,
    pub sport_type: String,
    pub has_heartrate: bool,
    pub average_heartrate: Option<f64>,
    pub max_heartrate: Option<f64>,
    pub average_speed: f64,
    pub max_speed: f64,
    pub summary_polyline: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StravaAuthUrl {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StravaTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub grant_type: String,
}

pub struct StravaClient {
    client: Client,
    config: StravaConfig,
}

impl StravaClient {
    pub fn new(config: StravaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Gera URL de autorização para o usuário
    pub fn get_auth_url(&self, scopes: &[&str]) -> StravaAuthUrl {
        let state = uuid::Uuid::new_v4().to_string();
        let scope_string = scopes.join(",");
        
        let mut url = url::Url::parse(STRAVA_AUTH_URL).unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("approval_prompt", "force")
            .append_pair("scope", &scope_string)
            .append_pair("state", &state);

        StravaAuthUrl {
            auth_url: url.to_string(),
            state,
        }
    }

    /// Troca o código de autorização por um token de acesso
    pub async fn exchange_token(&self, code: &str) -> Result<StravaTokenResponse, Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");

        let response = self.client
            .post(STRAVA_TOKEN_URL)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: StravaTokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro na troca de token: {}", error_text).into())
        }
    }

    /// Atualiza o token de acesso usando o refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<StravaTokenResponse, Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("refresh_token", refresh_token);
        params.insert("grant_type", "refresh_token");

        let response = self.client
            .post(STRAVA_TOKEN_URL)
            .json(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: StravaTokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro na atualização do token: {}", error_text).into())
        }
    }

    /// Lista as atividades do usuário autenticado
    pub async fn list_activities(
        &self,
        access_token: &str,
        per_page: Option<u32>,
        page: Option<u32>,
        after: Option<i64>,
        before: Option<i64>,
    ) -> Result<Vec<StravaActivity>, Box<dyn Error>> {
        let mut url = format!("{}/athlete/activities", STRAVA_API_BASE);
        let mut query_params = Vec::new();
        
        if let Some(pp) = per_page {
            query_params.push(format!("per_page={}", pp));
        }
        if let Some(p) = page {
            query_params.push(format!("page={}", p));
        }
        if let Some(a) = after {
            query_params.push(format!("after={}", a));
        }
        if let Some(b) = before {
            query_params.push(format!("before={}", b));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let activities: Vec<StravaActivity> = response.json().await?;
            Ok(activities)
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro ao listar atividades: {} - {}", response.status(), error_text).into())
        }
    }

    /// Obtém detalhes de uma atividade específica
    pub async fn get_activity(&self, access_token: &str, activity_id: u64) -> Result<StravaActivity, Box<dyn Error>> {
        let url = format!("{}/activities/{}", STRAVA_API_BASE, activity_id);

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let activity: StravaActivity = response.json().await?;
            Ok(activity)
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro ao obter atividade: {} - {}", response.status(), error_text).into())
        }
    }

    /// Baixa o arquivo GPX de uma atividade
    pub async fn download_activity_gpx(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error>> {
        let url = format!("{}/activities/{}/export_gpx", STRAVA_API_BASE, activity_id);

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro ao baixar GPX: {} - {}", response.status(), error_text).into())
        }
    }

    /// Baixa o arquivo TCX de uma atividade
    pub async fn download_activity_tcx(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error>> {
        let url = format!("{}/activities/{}/export_tcx", STRAVA_API_BASE, activity_id);

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro ao baixar TCX: {} - {}", response.status(), error_text).into())
        }
    }

    /// NOVA FUNCIONALIDADE: Baixa o arquivo original (normalmente FIT) de uma atividade
    pub async fn download_activity_original(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error>> {
        let url = format!("{}/activities/{}/export_original", STRAVA_API_BASE, activity_id);

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await?;
            Err(format!("Erro ao baixar arquivo original: {} - {}", response.status(), error_text).into())
        }
    }

    /// Verifica se o token ainda é válido
    pub fn is_token_valid(&self, expires_at: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        expires_at > now + 300 // 5 minutos de margem
    }
}

/// Estrutura para gerenciar tokens de sessão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub athlete_id: Option<u64>,
}

impl StravaSession {
    pub fn new(token_response: StravaTokenResponse) -> Self {
        Self {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: token_response.expires_at,
            athlete_id: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at <= now + 300 // 5 minutos de margem
    }

    pub async fn refresh_if_needed(&mut self, client: &StravaClient) -> Result<bool, Box<dyn Error>> {
        if self.is_expired() {
            let new_token = client.refresh_token(&self.refresh_token).await?;
            self.access_token = new_token.access_token;
            self.refresh_token = new_token.refresh_token;
            self.expires_at = new_token.expires_at;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Utilitário para converter dados do Strava para nossos formatos internos
pub mod conversion_utils {
    use super::*;
    use crate::utils::interpolate_gpx_points;
    use gpx::{Gpx, Track, TrackSegment, Waypoint};
    use geo_types::Point;
    use std::io::Cursor;

    /// Converte dados de uma atividade Strava para um objeto GPX simplificado
    pub fn activity_to_basic_gpx(activity: &StravaActivity) -> Result<Gpx, Box<dyn Error>> {
        let mut gpx = Gpx {
            version: gpx::GpxVersion::Gpx11,
            creator: Some("Strava Activity Import".to_string()),
            ..Default::default()
        };

        let mut track = Track::new();
        track.name = Some(activity.name.clone());
        track.comment = Some(format!("Atividade Strava ID: {}", activity.id));

        // Se tivermos polyline, podemos decodificar para pontos básicos
        if let Some(polyline) = &activity.summary_polyline {
            let points = decode_polyline(polyline)?;
            
            let mut segment = TrackSegment::new();
            for (i, (lat, lon)) in points.iter().enumerate() {
                let mut waypoint = Waypoint::new(Point::new(*lon, *lat));
                
                // Estimativa básica de tempo baseada na duração total
                if i > 0 {
                    let time_offset_secs = (activity.moving_time as f64 * i as f64) / points.len() as f64;
                    if let Ok(start_time) = activity.start_date.parse::<DateTime<Utc>>() {
                        let point_time = start_time + chrono::Duration::seconds(time_offset_secs as i64);
                        if let Ok(offset_dt) = time::OffsetDateTime::from_unix_timestamp(point_time.timestamp()) {
                            waypoint.time = Some(gpx::Time::from(offset_dt));
                        }
                    }
                }
                
                segment.points.push(waypoint);
            }
            
            if !segment.points.is_empty() {
                track.segments.push(segment);
            }
        }

        if !track.segments.is_empty() {
            gpx.tracks.push(track);
        }

        Ok(gpx)
    }

    /// Decodifica um polyline do Google/Strava para coordenadas lat/lon
    fn decode_polyline(polyline: &str) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
        let mut points = Vec::new();
        let mut index = 0;
        let chars: Vec<char> = polyline.chars().collect();
        let mut lat = 0i32;
        let mut lon = 0i32;

        while index < chars.len() {
            let mut shift = 0;
            let mut result = 0i32;

            // Decodificar latitude
            loop {
                let byte = chars[index] as u8 - 63;
                index += 1;
                result |= ((byte & 0x1f) as i32) << shift;
                shift += 5;
                if byte < 0x20 {
                    break;
                }
            }
            let dlat = if result & 1 != 0 { !(result >> 1) } else { result >> 1 };
            lat += dlat;

            shift = 0;
            result = 0;

            // Decodificar longitude
            loop {
                let byte = chars[index] as u8 - 63;
                index += 1;
                result |= ((byte & 0x1f) as i32) << shift;
                shift += 5;
                if byte < 0x20 {
                    break;
                }
            }
            let dlon = if result & 1 != 0 { !(result >> 1) } else { result >> 1 };
            lon += dlon;

            points.push((lat as f64 / 1e5, lon as f64 / 1e5));
        }

        Ok(points)
    }

    /// Converte bytes de arquivo GPX baixado do Strava para objeto Gpx
    pub fn bytes_to_gpx(bytes: &[u8]) -> Result<Gpx, Box<dyn Error>> {
        let cursor = Cursor::new(bytes);
        let gpx = gpx::read(cursor)?;
        Ok(gpx)
    }

    /// Converte bytes de arquivo TCX baixado do Strava para objeto Gpx usando nosso adaptador
    pub fn bytes_to_tcx_gpx(bytes: &[u8]) -> Result<crate::tcx_adapter::TcxProcessResult, Box<dyn Error>> {
        // Salvar temporariamente o arquivo para usar com nosso leitor TCX existente
        let temp_file = std::env::temp_dir().join(format!("strava_tcx_{}.tcx", uuid::Uuid::new_v4()));
        std::fs::write(&temp_file, bytes)?;
        
        let result = crate::tcx_adapter::read_and_process_tcx(&temp_file)?;
        
        // Limpar arquivo temporário
        let _ = std::fs::remove_file(&temp_file);
        
        Ok(result)
    }
}