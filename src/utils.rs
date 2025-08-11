// src/utils.rs

use std::path::Path;
use std::error::Error;
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::America::Sao_Paulo;
use gpx::{Gpx, Waypoint, Track, TrackSegment};
use geo_types::Point;

pub fn get_video_time_range(video_path: &Path, lang: &str) -> Result<(DateTime<Utc>, DateTime<Utc>), Box<dyn Error>> {
    let metadata = ffprobe::ffprobe(video_path).map_err(|e| {
        let error_message = e.to_string();
        if error_message.contains("No such file or directory") || error_message.contains("not found") {
            let msg = if lang == "en" {
                "Command 'ffprobe' not found. Make sure FFmpeg is installed and in the system's PATH."
            } else {
                "Comando 'ffprobe' não encontrado. Verifique se o FFmpeg está instalado e no PATH do sistema."
            };
            Box::<dyn Error>::from(msg)
        } else {
            Box::<dyn Error>::from(format!("Error executing ffprobe: {}", error_message))
        }
    })?;

    let creation_time_str = metadata.streams
        .iter()
        .find(|s| s.codec_type == Some("video".to_string()))
        .and_then(|s| s.tags.as_ref())
        .and_then(|t| t.creation_time.as_deref())
        .ok_or(if lang == "en" {
            "Tag 'creation_time' not found in the video stream."
        } else {
            "Tag 'creation_time' não encontrada no stream de vídeo."
        })?;
        
    let naive_datetime_from_video = DateTime::parse_from_rfc3339(creation_time_str)?.naive_utc();
    let local_datetime = Sao_Paulo.from_local_datetime(&naive_datetime_from_video).single()
        .ok_or(if lang == "en" {
            "Could not convert local time for the São Paulo timezone."
        } else {
            "Não foi possível converter a hora local para o fuso de São Paulo."
        })?;
    let start_time_utc = local_datetime.with_timezone(&Utc);

    let duration_str = metadata.format.duration.ok_or(if lang == "en" { "Duration not found." } else { "Duração não encontrada." })?;
    let duration_secs = duration_str.parse::<f64>()?;
    let duration = Duration::microseconds((duration_secs * 1_000_000.0) as i64);
    
    Ok((start_time_utc, start_time_utc + duration))
}

fn distance_2d(p1: &Waypoint, p2: &Waypoint) -> f64 {
    const EARTH_RADIUS_METERS: f64 = 6371000.0;
    let lat1 = p1.point().y().to_radians(); 
    let lon1 = p1.point().x().to_radians();
    let lat2 = p2.point().y().to_radians(); 
    let lon2 = p2.point().x().to_radians();
    
    let dlat = lat2 - lat1; 
    let dlon = lon2 - lon1;
    
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS_METERS * c
}

fn distance_3d(p1: &Waypoint, p2: &Waypoint) -> f64 {
    let horizontal_distance = distance_2d(p1, p2);
    
    let vertical_distance = match (p1.elevation, p2.elevation) {
        (Some(e1), Some(e2)) => e2 - e1,
        _ => 0.0,
    };

    (horizontal_distance.powi(2) + vertical_distance.powi(2)).sqrt()
}

pub fn calculate_speed_kmh(p1: &Waypoint, p2: &Waypoint) -> Option<f64> {
    let distance_m = distance_3d(p1, p2);
    if let (Some(time1_str), Some(time2_str)) = (p1.time.as_ref().and_then(|t| t.format().ok()), p2.time.as_ref().and_then(|t| t.format().ok())) {
        if let (Ok(time1), Ok(time2)) = (time1_str.parse::<DateTime<Utc>>(), time2_str.parse::<DateTime<Utc>>()) {
            let time_diff_secs = (time2 - time1).num_seconds();
            if time_diff_secs > 0 { 
                return Some((distance_m / time_diff_secs as f64) * 3.6); 
            }
        }
    }
    None
}

pub fn calculate_g_force(p1: &Waypoint, p2: &Waypoint, p3: &Waypoint) -> Option<f64> {
    let speed1_kmh = calculate_speed_kmh(p1, p2)?;
    let speed2_kmh = calculate_speed_kmh(p2, p3)?;

    let speed1_mps = speed1_kmh / 3.6;
    let speed2_mps = speed2_kmh / 3.6;

    if let (Some(time1_str), Some(time2_str)) = (p2.time.as_ref().and_then(|t| t.format().ok()), p3.time.as_ref().and_then(|t| t.format().ok())) {
        if let (Ok(time1), Ok(time2)) = (time1_str.parse::<DateTime<Utc>>(), time2_str.parse::<DateTime<Utc>>()) {
            let time_diff_secs = (time2 - time1).num_seconds();

            if time_diff_secs > 0 {
                let acceleration_mps2 = (speed2_mps - speed1_mps) / time_diff_secs as f64;
                const STANDARD_GRAVITY: f64 = 9.80665;
                return Some(acceleration_mps2 / STANDARD_GRAVITY);
            }
        }
    }

    None
}

pub fn calculate_bearing(p1: &Waypoint, p2: &Waypoint) -> f64 {
    let lat1 = p1.point().y().to_radians();
    let lon1 = p1.point().x().to_radians();
    let lat2 = p2.point().y().to_radians();
    let lon2 = p2.point().x().to_radians();

    let d_lon = lon2 - lon1;

    let y = d_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();

    let initial_bearing_rad = y.atan2(x);
    let initial_bearing_deg = initial_bearing_rad.to_degrees();
    
    (initial_bearing_deg + 360.0) % 360.0
}

// Função para calcular distância acumulada até um ponto específico
pub fn calculate_distance_to_point(all_points: &[&Waypoint], current_index: usize) -> f64 {
    let mut distance = 0.0;
    
    for i in 1..=current_index.min(all_points.len() - 1) {
        distance += distance_2d(all_points[i-1], all_points[i]);
    }
    
    distance / 1000.0 // Converter para km
}

// Nova função para calcular ganho de elevação até um ponto específico
pub fn calculate_elevation_gain_to_point(all_points: &[&Waypoint], current_index: usize) -> f64 {
    let mut gain = 0.0;
    
    for i in 1..=current_index.min(all_points.len() - 1) {
        if let (Some(prev_elev), Some(curr_elev)) = (all_points[i-1].elevation, all_points[i].elevation) {
            if curr_elev > prev_elev {
                gain += curr_elev - prev_elev;
            }
        }
    }
    
    gain
}

fn interpolate_points(p1: &Waypoint, p2: &Waypoint, max_interval_secs: i64) -> Vec<Waypoint> {
    let mut interpolated_points = Vec::new();
    
    let time1_str = match p1.time.as_ref().and_then(|t| t.format().ok()) {
        Some(t) => t,
        None => return interpolated_points,
    };
    
    let time2_str = match p2.time.as_ref().and_then(|t| t.format().ok()) {
        Some(t) => t,
        None => return interpolated_points,
    };
    
    let time1 = match time1_str.parse::<DateTime<Utc>>() {
        Ok(t) => t,
        Err(_) => return interpolated_points,
    };
    
    let time2 = match time2_str.parse::<DateTime<Utc>>() {
        Ok(t) => t,
        Err(_) => return interpolated_points,
    };
    
    let time_diff_secs = (time2 - time1).num_seconds();
    
    if time_diff_secs <= max_interval_secs {
        return interpolated_points;
    }
    
    let num_intervals = (time_diff_secs as f64 / max_interval_secs as f64).ceil() as i64;
    let num_points_to_add = num_intervals - 1;
    
    let lat1 = p1.point().y();
    let lon1 = p1.point().x();
    let lat2 = p2.point().y();
    let lon2 = p2.point().x();
    
    let elev1 = p1.elevation.unwrap_or(0.0);
    let elev2 = p2.elevation.unwrap_or(0.0);
    
    for i in 1..=num_points_to_add {
        let ratio = i as f64 / num_intervals as f64;
        
        let new_lat = lat1 + (lat2 - lat1) * ratio;
        let new_lon = lon1 + (lon2 - lon1) * ratio;
        let new_elev = elev1 + (elev2 - elev1) * ratio;
        
        let time_offset_ms = (time_diff_secs as f64 * ratio * 1000.0) as i64;
        let new_time = time1 + Duration::milliseconds(time_offset_ms);
        
        let mut new_waypoint = Waypoint::new(Point::new(new_lon, new_lat));
        new_waypoint.elevation = Some(new_elev);
        
        let timestamp = new_time.timestamp();
        let nanos = new_time.timestamp_subsec_nanos();
        
        if let Ok(offset_dt) = time::OffsetDateTime::from_unix_timestamp_nanos((timestamp * 1_000_000_000 + nanos as i64) as i128) {
            new_waypoint.time = Some(gpx::Time::from(offset_dt));
        }
        
        interpolated_points.push(new_waypoint);
    }
    
    interpolated_points
}

pub fn interpolate_gpx_points(mut gpx: Gpx, max_interval_secs: i64) -> Gpx {
    let mut new_tracks = Vec::new();
    
    for track in gpx.tracks.iter() {
        let mut new_segments = Vec::new();
        
        for segment in track.segments.iter() {
            let mut new_points = Vec::new();
            let points = &segment.points;
            
            if points.is_empty() {
                continue;
            }
            
            new_points.push(points[0].clone());
            
            for i in 1..points.len() {
                let p1 = &points[i - 1];
                let p2 = &points[i];
                
                let interpolated = interpolate_points(p1, p2, max_interval_secs);
                new_points.extend(interpolated);
                
                new_points.push(p2.clone());
            }
            
            let mut new_segment = TrackSegment::new();
            new_segment.points = new_points;
            new_segments.push(new_segment);
        }
        
        let mut new_track = Track::new();
        new_track.name = track.name.clone();
        new_track.comment = track.comment.clone();
        new_track.description = track.description.clone();
        new_track.source = track.source.clone();
        new_track.number = track.number;
        new_track.type_ = track.type_.clone();
        new_track.segments = new_segments;
        
        new_tracks.push(new_track);
    }
    
    gpx.tracks = new_tracks;
    gpx
}