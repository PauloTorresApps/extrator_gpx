// src/fit_adapter.rs

use fitparser::FitObject;
use gpx::{Gpx, GpxVersion, Track, TrackSegment, Waypoint};
use std::path::Path;
use std::fs::File;
use chrono::{TimeZone, Utc};
use std::time::SystemTime;

use crate::tcx_adapter::TcxExtraData;

pub struct FitProcessResult {
    pub gpx: Gpx,
    pub extra_data: FitExtraData,
}

#[derive(Default)]
pub struct FitExtraData {
    pub sport: Option<String>,
    pub total_time_seconds: f64,
    pub total_distance_meters: f64,
    pub total_calories: f64,
    pub max_speed: f64,
    pub heart_rates: Vec<f64>,
    pub cadences: Vec<f64>,
}

impl FitExtraData {
    pub fn to_tcx_extra_data(self) -> TcxExtraData {
        TcxExtraData {
            sport: self.sport,
            total_time_seconds: self.total_time_seconds,
            total_distance_meters: self.total_distance_meters,
            total_calories: self.total_calories,
            max_speed: self.max_speed,
            heart_rates: self.heart_rates,
            cadences: self.cadences,
        }
    }
}

pub fn read_and_process_fit(path: &Path) -> Result<FitProcessResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = File::open(path)?;
    let fit_data = fitparser::from_reader(&mut file)?;

    let mut track_segment = TrackSegment::new();
    let mut extra_data = FitExtraData::default();

    for record in fit_data {
        if let FitObject::Record(data_messages) = record {
            for data_message in data_messages {
                let mut point = Waypoint::new(
                    geo_types::Point::new(0.0, 0.0)
                );
                
                let mut lat = None;
                let mut lon = None;

                for field in data_message.fields() {
                    match field.name() {
                        "position_lat" => lat = field.value().as_f64(),
                        "position_long" => lon = field.value().as_f64(),
                        "timestamp" => {
                            if let Some(ts_val) = field.value().as_u32() {
                                let dt = Utc.with_ymd_and_hms(1989, 12, 31, 0, 0, 0).unwrap();
                                let timestamp = dt + chrono::Duration::seconds(ts_val as i64);
                                point.time = Some(SystemTime::from(timestamp).into());
                            }
                        },
                        "speed" => {
                            if let Some(val) = field.value().as_f64() {
                                point.speed = Some(val);
                                if val > extra_data.max_speed { extra_data.max_speed = val; }
                            }
                        },
                        "distance" => {
                            if let Some(val) = field.value().as_f64() {
                                extra_data.total_distance_meters = val;
                            }
                        }
                        "heart_rate" => {
                            if let Some(val) = field.value().as_f64() {
                                extra_data.heart_rates.push(val);
                            }
                        },
                        "cadence" => {
                            if let Some(val) = field.value().as_f64() {
                                extra_data.cadences.push(val);
                            }
                        },
                        _ => {}
                    }
                }

                if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
                    point.point = geo_types::Point::new(lon_val, lat_val);
                    track_segment.points.push(point);
                }
            }
        } else if let FitObject::Session(sessions) = record {
             for session in sessions {
                for field in session.fields() {
                    match field.name() {
                        "sport" => extra_data.sport = field.value().as_string(),
                        "total_elapsed_time" => extra_data.total_time_seconds = field.value().as_f64().unwrap_or(0.0),
                        "total_calories" => extra_data.total_calories = field.value().as_u16().unwrap_or(0) as f64,
                        _ => {}
                    }
                }
            }
        }
    }
    
    let mut track = Track::new();
    track.segments.push(track_segment);

    let mut gpx = Gpx {
        version: GpxVersion::Gpx11,
        creator: Some("extrator_gpx".to_string()),
        ..Default::default()
    };
    gpx.tracks.push(track);

    Ok(FitProcessResult { gpx, extra_data })
}
