use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::error::Error;

use chrono::{DateTime, Duration, Utc};
use gpx::{Gpx, Waypoint, read};

use crate::drawing::generate_speedometer_image;
use crate::utils::{calculate_speed_kmh, get_video_time_range, calculate_g_force, calculate_bearing};

pub struct FrameInfo {
    path: String,
    timestamp_sec: f64,
}

// A função já não aceita o `time_offset_seconds`
pub fn run_processing(gpx_path: PathBuf, video_path: PathBuf) -> Result<Vec<String>, (String, Vec<String>)> {
    let mut logs = Vec::new();
    
    match process_internal(gpx_path, video_path, &mut logs) {
        Ok(_) => {
            logs.push("Processo concluído com sucesso!".to_string());
            Ok(logs)
        },
        Err(e) => {
            let error_message = e.to_string();
            logs.push(format!("Ocorreu um erro: {}", error_message));
            Err((error_message, logs))
        }
    }
}

fn process_internal(gpx_path: PathBuf, video_path: PathBuf, logs: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    let output_dir = "output_frames";
    logs.push(format!("A criar diretório de saída: {}", output_dir));
    fs::create_dir_all(output_dir)?;

    logs.push(format!("A ler metadados do vídeo: {:?}", video_path));
    let (video_start_time, video_end_time) = get_video_time_range(&video_path)?;
    logs.push(format!("Início do vídeo (UTC): {}", video_start_time));
    logs.push(format!("Fim do vídeo (UTC):   {}", video_end_time));
    
    logs.push(format!("A ler ficheiro GPX: {:?}", gpx_path));
    let gpx: Gpx = read(BufReader::new(File::open(&gpx_path)?))?;
    logs.push("Ficheiro GPX lido com sucesso!".to_string());
    
    let mut frame_infos: Vec<FrameInfo> = Vec::new();
    let mut frame_counter = 0; // Contador global de frames

    for (track_idx, track) in gpx.tracks.iter().enumerate() {
        logs.push(format!("A processar Trilha #{}", track_idx + 1));
        for (segment_idx, segment) in track.segments.iter().enumerate() {
            logs.push(format!("A processar Segmento #{}", segment_idx + 1));
            
            let synced_points: Vec<&Waypoint> = segment.points.iter().filter(|point| {
                if let Some(time_str) = point.time.as_ref().and_then(|t| t.format().ok()) {
                    if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                        // Volta a usar o ajuste de tempo fixo.
                        let adjusted_point_time = point_time - Duration::hours(3);
                        adjusted_point_time >= video_start_time && adjusted_point_time <= video_end_time
                    } else { false }
                } else { false }
            }).collect();

            if synced_points.len() < 3 {
                logs.push("Pontos de telemetria insuficientes (< 3) para processar este segmento.".to_string());
                continue;
            }
            logs.push(format!("Encontrados {} pontos sincronizados. A gerar imagens...", synced_points.len()));

            for i in 1..synced_points.len() - 1 {
                let p1 = synced_points[i - 1];
                let p2 = synced_points[i];
                let p3 = synced_points[i + 1];

                let speed_kmh = calculate_speed_kmh(p1, p2).unwrap_or(0.0);
                let g_force = calculate_g_force(p1, p2, p3).unwrap_or(0.0);
                let bearing = calculate_bearing(p1, p2);

                let output_path = format!("{}/frame_{:05}.png", output_dir, frame_counter);
                frame_counter += 1;
                generate_speedometer_image(speed_kmh, bearing, g_force, &output_path)?;

                if let Some(time_str) = p2.time.as_ref().and_then(|t| t.format().ok()) {
                     if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                        let adjusted_point_time = point_time - Duration::hours(3);
                        let timestamp_sec = (adjusted_point_time - video_start_time).num_milliseconds() as f64 / 1000.0;
                        frame_infos.push(FrameInfo { path: output_path, timestamp_sec });
                    }
                }
            }
            logs.push("Geração de imagens para este segmento concluída!".to_string());
        }
    }

    if !frame_infos.is_empty() {
        logs.push("A gerar o vídeo final com todos os overlays (pode demorar)...".to_string());
        generate_final_video(&video_path, &frame_infos)?;
        logs.push("Vídeo final 'output_video.mp4' gerado com sucesso!".to_string());
    } else {
        logs.push("Nenhum frame foi gerado, o vídeo final não será criado.".to_string());
    }

    Ok(())
}

fn generate_final_video(video_path: &Path, frame_infos: &[FrameInfo]) -> Result<(), Box<dyn Error>> {
    if frame_infos.is_empty() { return Ok(()); }
    let mut complex_filter = String::new();
    let mut inputs: Vec<String> = vec!["-i".to_string(), video_path.to_str().unwrap().to_string()];
    for info in frame_infos.iter() { inputs.push("-i".to_string()); inputs.push(info.path.clone()); }

    let mut last_stream = "[0:v]".to_string();
    for i in 0..frame_infos.len() {
        let info = &frame_infos[i];
        let next_info = frame_infos.get(i + 1);
        let end_time = next_info.map_or(f64::MAX, |ni| ni.timestamp_sec);
        let start_time = info.timestamp_sec;
        let current_stream = format!("[v{}]", i);
        
        let filter_part = format!(
            "{}[{}:v]overlay=10:main_h-overlay_h-10:enable='between(t,{},{})'{}",
            last_stream, i + 1, start_time, end_time,
            if i == frame_infos.len() - 1 { "".to_string() } else { current_stream.clone() }
        );
        complex_filter.push_str(&filter_part);
        if i < frame_infos.len() - 1 { complex_filter.push(';'); last_stream = current_stream; }
    }

    let output_file = "output_video.mp4";
    if Path::new(output_file).exists() { fs::remove_file(output_file)?; }

    let status = StdCommand::new("ffmpeg").args(&inputs).arg("-filter_complex").arg(&complex_filter).arg("-c:a").arg("copy").arg(output_file).status()?;
    if !status.success() { return Err("O comando FFmpeg falhou.".into()); }
    Ok(())
}
