// src/strava_integration.rs

use serde::{Deserialize, Serialize};
use reqwest::{Client, header};
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const STRAVA_API_BASE_URL: &str = "https://www.strava.com/api/v3";

#[derive(Debug, Clone)]
pub struct StravaConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug)]
pub struct StravaClient {
    config: StravaConfig,
    http_client: Client,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StravaTokenResponse {
    pub token_type: String,
    pub expires_at: u64,
    pub expires_in: u64,
    pub refresh_token: String,
    pub access_token: String,
    pub athlete: Option<StravaAthlete>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StravaAthlete {
    pub id: u64,
    pub username: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StravaActivity {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub start_date: String,
    pub distance: f64,
    pub moving_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub athlete_id: Option<u64>,
}

impl StravaSession {
    pub fn new(token_response: StravaTokenResponse) -> Self {
        Self {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: token_response.expires_at,
            athlete_id: token_response.athlete.map(|a| a.id),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now >= self.expires_at
    }

    pub async fn refresh_if_needed(&mut self, client: &StravaClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.is_expired() {
            let response = client.refresh_token(&self.refresh_token).await?;
            self.access_token = response.access_token;
            self.refresh_token = response.refresh_token;
            self.expires_at = response.expires_at;
        }
        Ok(())
    }
}


pub struct StravaAuthData {
    pub auth_url: String,
}

impl StravaClient {
    pub fn new(config: StravaConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    pub fn get_auth_url(&self, scopes: &[&str]) -> StravaAuthData {
        let state = Uuid::new_v4().to_string();
        let scope_str = scopes.join(",");
        let auth_url = format!(
            "https://www.strava.com/oauth/authorize?client_id={}&response_type=code&redirect_uri={}&approval_prompt=force&scope={}&state={}",
            self.config.client_id, self.config.redirect_uri, scope_str, state
        );
        StravaAuthData { auth_url }
    }

    pub async fn exchange_token(&self, code: &str) -> Result<StravaTokenResponse, Box<dyn Error + Send + Sync>> {
        let mut params = HashMap::new();
        params.insert("client_id", self.config.client_id.clone());
        params.insert("client_secret", self.config.client_secret.clone());
        params.insert("code", code.to_string());
        params.insert("grant_type", "authorization_code".to_string());

        let response = self.http_client
            .post("https://www.strava.com/oauth/token")
            .form(&params)
            .send()
            .await?
            .json::<StravaTokenResponse>()
            .await?;
        
        Ok(response)
    }
    
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<StravaTokenResponse, Box<dyn Error + Send + Sync>> {
        let mut params = HashMap::new();
        params.insert("client_id", self.config.client_id.clone());
        params.insert("client_secret", self.config.client_secret.clone());
        params.insert("refresh_token", refresh_token.to_string());
        params.insert("grant_type", "refresh_token".to_string());

        let response = self.http_client
            .post("https://www.strava.com/oauth/token")
            .form(&params)
            .send()
            .await?
            .json::<StravaTokenResponse>()
            .await?;
        
        Ok(response)
    }

    pub async fn list_activities(
        &self,
        access_token: &str,
        per_page: Option<u32>,
        page: Option<u32>,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<Vec<StravaActivity>, Box<dyn Error + Send + Sync>> {
        let mut url = format!("{}/athlete/activities", STRAVA_API_BASE_URL);
        let mut query_params = Vec::new();

        if let Some(p) = per_page { query_params.push(format!("per_page={}", p)); }
        if let Some(p) = page { query_params.push(format!("page={}", p)); }
        if let Some(b) = before { query_params.push(format!("before={}", b)); }
        if let Some(a) = after { query_params.push(format!("after={}", a)); }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }
        
        let activities = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Vec<StravaActivity>>()
            .await?;

        Ok(activities)
    }

    pub async fn download_activity_original(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let download_url = format!("https://www.strava.com/activities/{}/export_original", activity_id);
        
        let bytes = self.http_client
            .get(&download_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes.to_vec())
    }

    pub async fn download_activity_tcx(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let download_url = format!("https://www.strava.com/activities/{}/export_tcx", activity_id);
        
        let bytes = self.http_client
            .get(&download_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes.to_vec())
    }

    pub async fn download_activity_gpx(&self, access_token: &str, activity_id: u64) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let download_url = format!("https://www.strava.com/activities/{}/export_gpx", activity_id);
        
        let bytes = self.http_client
            .get(&download_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes.to_vec())
    }
    
    // pub fn bytes_to_tcx_gpx(bytes: &[u8]) -> Result<crate::tcx_adapter::TcxProcessResult, Box<dyn Error + Send + Sync>> {
    //     let temp_dir = std::env::temp_dir();
    //     let temp_file_name = format!("{}.tcx", Uuid::new_v4());
    //     let temp_file = temp_dir.join(temp_file_name);
        
    //     {
    //         let mut file = File::create(&temp_file)?;
    //         file.write_all(bytes)?;
    //     }

    //     let result = crate::tcx_adapter::read_and_process_tcx(&temp_file)?;

    //     std::fs::remove_file(&temp_file)?;

    //     Ok(result)
    // }
}
