use std::path::Path;
use std::error::Error;
use chrono::{DateTime, Duration, Utc};
use gpx::{Gpx, Waypoint}; // Adicione Gpx ao "use" no topo do arquivo se não estiver lá


pub fn get_video_time_range(video_path: &Path) -> Result<(DateTime<Utc>, DateTime<Utc>), Box<dyn Error>> {
    // Simplifica o tratamento de erros para ser mais genérico
    let metadata = match ffprobe::ffprobe(video_path) {
        Ok(data) => data,
        Err(e) => {
            let error_message = e.to_string();
            // Verifica se o erro parece ser sobre comando não encontrado
            if error_message.contains("No such file or directory") || 
               error_message.contains("not found") ||
               error_message.contains("cannot find") {
                return Err("Comando 'ffprobe' não encontrado. Verifique se o FFmpeg está instalado e se o seu diretório está no PATH do sistema.".into());
            }
            // Para outros erros, propaga-os com mais contexto
            return Err(format!("Erro ao executar o ffprobe: {}", error_message).into());
        }
    };
    
    let start_time = if let Some(stream) = metadata.streams.iter().find(|s| s.codec_type == Some("video".to_string())) {
        if let Some(tags) = &stream.tags {
            if let Some(creation_time_str) = &tags.creation_time {
                creation_time_str.parse::<DateTime<Utc>>()?
            } else { return Err("Tag 'creation_time' não encontrada.".into()); }
        } else { return Err("Nenhuma tag encontrada.".into()); }
    } else { return Err("Nenhum stream de vídeo encontrado.".into()); };
    
    let duration_str = metadata.format.duration.ok_or("Duração não encontrada.")?;
    let duration_secs = duration_str.parse::<f64>()?;
    let duration = Duration::microseconds((duration_secs * 1_000_000.0) as i64);
    
    Ok((start_time, start_time + duration))
}

// Calcula a distância 2D (horizontal) em metros entre dois pontos de GPS.
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

// Calcula a distância 3D real, considerando a elevação.
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

// Calcula a aceleração em Gs.
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

// Calcula a direção (bearing) em graus entre dois pontos.
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

    // Normaliza para um azimute de 0-360 graus
    (initial_bearing_deg + 360.0) % 360.0
}

/// Encontra o ponto de trilha GPX cujo timestamp é o mais próximo do horário alvo.
pub fn find_closest_gpx_point(gpx_data: &Gpx, target_time: DateTime<Utc>) -> Option<Waypoint> {
    gpx_data
        .tracks
        .iter()
        .flat_map(|track| track.segments.iter())
        .flat_map(|segment| segment.points.iter())
        // Filtra apenas pontos que têm timestamp
        .filter_map(|point| {
            point.time.and_then(|t| t.format().ok()).and_then(|time_str| {
                if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                    // Calcula a diferença de tempo absoluta
                    let duration = (target_time - point_time).num_seconds().abs();
                    Some((duration, point.clone())) // Clonamos o ponto para retorná-lo
                } else {
                    None
                }
            })
        })
        // Encontra o ponto com a menor diferença de tempo
        .min_by(|(duration1, _), (duration2, _)| duration1.partial_cmp(duration2).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, point)| point)
}