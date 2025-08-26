// src/tcx_adapter.rs

use gpx::{Gpx, GpxVersion, Track, TrackSegment, Waypoint};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct TcxExtraData {
    pub sport: Option<String>,
    pub total_time_seconds: f64,
    pub total_distance_meters: f64,
    pub total_calories: f64,
    pub max_speed: f64,
    pub heart_rates: Vec<f64>,
    pub cadences: Vec<f64>,
}

impl TcxExtraData {
    pub fn average_heart_rate(&self) -> Option<f64> {
        if self.heart_rates.is_empty() { None }
        else { Some(self.heart_rates.iter().sum::<f64>() / self.heart_rates.len() as f64) }
    }
    pub fn max_heart_rate(&self) -> Option<f64> {
        self.heart_rates.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap())
    }
    pub fn average_cadence(&self) -> Option<f64> {
        if self.cadences.is_empty() { None }
        else { Some(self.cadences.iter().sum::<f64>() / self.cadences.len() as f64) }
    }
    pub fn max_cadence(&self) -> Option<f64> {
        self.cadences.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap())
    }
    pub fn average_speed(&self) -> Option<f64> {
        if self.total_time_seconds > 0.0 { Some(self.total_distance_meters / self.total_time_seconds) }
        else { None }
    }
}

pub struct TcxProcessResult {
    pub gpx: Gpx,
    pub extra_data: TcxExtraData,
}

pub fn read_and_process_tcx(path: &Path) -> Result<TcxProcessResult, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    let mut track_segment = TrackSegment::new();
    let mut extra_data = TcxExtraData::default();
    
    let mut in_trackpoint = false;
    let mut current_tag = String::new();
    let mut lat = 0.0;
    let mut lon = 0.0;
    let mut current_time: Option<gpx::Time> = None;
    let mut current_elevation: Option<f64> = None;
    
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                current_tag = name.local_name.clone();
                if current_tag == "Activity" {
                    if let Some(sport) = attributes.iter().find(|attr| attr.name.local_name == "Sport") {
                        extra_data.sport = Some(sport.value.clone());
                    }
                }
                if current_tag == "Trackpoint" {
                    in_trackpoint = true;
                    lat = 0.0;
                    lon = 0.0;
                    current_time = None;
                    current_elevation = None;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "Trackpoint" {
                    in_trackpoint = false;
                    let mut new_point = Waypoint::new(geo_types::Point::new(lon, lat));
                    new_point.time = current_time;
                    new_point.elevation = current_elevation;
                    track_segment.points.push(new_point);
                }
                current_tag.clear();
            }
            Ok(XmlEvent::Characters(text)) => {
                match current_tag.as_str() {
                    "Time" if in_trackpoint => {
                        if let Ok(time) = chrono::DateTime::parse_from_rfc3339(&text) {
                            if let Ok(offset_dt) = OffsetDateTime::from_unix_timestamp(time.timestamp()) {
                                current_time = Some(gpx::Time::from(offset_dt));
                            }
                        }
                    },
                    "LatitudeDegrees" if in_trackpoint => lat = text.parse().unwrap_or(0.0),
                    "LongitudeDegrees" if in_trackpoint => lon = text.parse().unwrap_or(0.0),
                    "AltitudeMeters" if in_trackpoint => current_elevation = text.parse().ok(),
                    "Value" if in_trackpoint => {
                        if let Ok(hr) = text.parse::<f64>() {
                            extra_data.heart_rates.push(hr);
                        }
                    },
                    "RunCadence" if in_trackpoint => {
                         if let Ok(cad) = text.parse::<f64>() {
                            extra_data.cadences.push(cad);
                        }
                    },
                    "TotalTimeSeconds" => extra_data.total_time_seconds = text.parse().unwrap_or(0.0),
                    "DistanceMeters" => extra_data.total_distance_meters = text.parse().unwrap_or(0.0),
                    "Calories" => extra_data.total_calories = text.parse().unwrap_or(0.0),
                    "MaximumSpeed" => extra_data.max_speed = text.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
            Err(e) => {
                return Err(Box::new(e));
            }
            _ => {}
        }
    }

    let mut track = Track::new();
    track.segments.push(track_segment);

    let mut gpx = Gpx {
        version: GpxVersion::Gpx11,
        creator: Some("extrator_gpx_tcx_adapter".to_string()),
        ..Default::default()
    };
    gpx.tracks.push(track);

    Ok(TcxProcessResult { gpx, extra_data })
}