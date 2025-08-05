use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::error::Error;

use chrono::{DateTime, Utc};
use gpx::{Gpx, Waypoint, read};

use crate::drawing::generate_speedometer_image;
use crate::utils::{calculate_speed_kmh, get_video_time_range, calculate_g_force, calculate_bearing};

pub struct FrameInfo {
    path: String,
    timestamp_sec: f64,
}

pub fn run_processing(gpx_path: PathBuf, video_path: PathBuf, sync_timestamp_str: String) -> Result<Vec<String>, (String, Vec<String>)> {
    let mut logs = Vec::new();
    
    // Usamos uma função interna para facilitar o tratamento de erros com `?`
    // e ainda assim capturar os logs.
    match process_internal(gpx_path.clone(), video_path.clone(), &mut logs, sync_timestamp_str) {
        Ok(_) => {
            logs.push("Processo concluído com sucesso!".to_string());
            Ok(logs)
        },
        Err(e) => {
            let error_message = e.to_string();
            logs.push(format!("Ocorreu um erro: {}", error_message));
            // Mesmo em caso de erro, tenta limpar os ficheiros.
            cleanup_files(&gpx_path, &video_path, &mut logs);
            Err((error_message, logs))
        }
    }
}

fn process_internal(gpx_path: PathBuf, video_path: PathBuf, logs: &mut Vec<String>, sync_timestamp_str: String) -> Result<(), Box<dyn Error>> {
    let output_dir = "output_frames";
    let final_video_dir = "output";
    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(final_video_dir)?;

    logs.push(format!("A ler metadados do vídeo: {:?}", video_path));
    let (video_start_time, video_end_time) = get_video_time_range(&video_path)?;
    logs.push(format!("Início do vídeo (UTC): {}", video_start_time));
    
    let selected_gpx_time = sync_timestamp_str.parse::<DateTime<Utc>>()?;
    logs.push(format!("Ponto de sincronização GPX selecionado (UTC): {}", selected_gpx_time));
    
    let time_offset = selected_gpx_time - video_start_time;
    logs.push(format!("Desvio de tempo calculado: {} segundos.", time_offset.num_seconds()));

    logs.push(format!("A ler ficheiro GPX: {:?}", gpx_path));
    let gpx: Gpx = read(BufReader::new(File::open(&gpx_path)?))?;
    logs.push("Ficheiro GPX lido com sucesso!".to_string());
    
    let mut frame_infos: Vec<FrameInfo> = Vec::new();
    let mut frame_counter = 0;

    for (track_idx, track) in gpx.tracks.iter().enumerate() {
        logs.push(format!("A processar Trilha #{}", track_idx + 1));
        for (segment_idx, segment) in track.segments.iter().enumerate() {
            logs.push(format!("A processar Segmento #{}", segment_idx + 1));
            
            let synced_points: Vec<&Waypoint> = segment.points.iter().filter(|point| {
                if let Some(time_str) = point.time.as_ref().and_then(|t| t.format().ok()) {
                    if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                        let adjusted_point_time = point_time - time_offset;
                        adjusted_point_time >= video_start_time && adjusted_point_time <= video_end_time
                    } else { false }
                } else { false }
            }).collect();

            if synced_points.len() < 3 {
                logs.push("Pontos de telemetria insuficientes (< 3).".to_string());
                continue;
            }
            logs.push(format!("Encontrados {} pontos. A gerar imagens...", synced_points.len()));

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
                        let adjusted_point_time = point_time - time_offset;
                        let timestamp_sec = (adjusted_point_time - video_start_time).num_milliseconds() as f64 / 1000.0;
                        frame_infos.push(FrameInfo { path: output_path, timestamp_sec });
                    }
                }
            }
            logs.push("Geração de imagens concluída!".to_string());
        }
    }

    if !frame_infos.is_empty() {
        logs.push("A gerar o vídeo final...".to_string());
        generate_final_video(&video_path, &frame_infos)?;
        logs.push("Vídeo final gerado com sucesso!".to_string());
    } else {
        logs.push("Nenhum frame foi gerado.".to_string());
    }

    cleanup_files(&gpx_path, &video_path, logs);

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

    let output_file = "output/output_video.mp4";
    if Path::new(output_file).exists() { fs::remove_file(output_file)?; }

    let status = StdCommand::new("ffmpeg").args(&inputs).arg("-filter_complex").arg(&complex_filter).arg("-c:a").arg("copy").arg(output_file).status()?;
    if !status.success() { return Err("O comando FFmpeg falhou.".into()); }
    Ok(())
}

fn cleanup_files(gpx_path: &Path, video_path: &Path, logs: &mut Vec<String>) {
    logs.push("A limpar ficheiros temporários...".to_string());

    if let Err(e) = fs::remove_file(gpx_path) {
        logs.push(format!("Aviso: Não foi possível apagar o ficheiro GPX temporário: {}", e));
    }
    if let Err(e) = fs::remove_file(video_path) {
        logs.push(format!("Aviso: Não foi possível apagar o ficheiro de vídeo temporário: {}", e));
    }
    if let Err(e) = fs::remove_dir_all("output_frames") {
        logs.push(format!("Aviso: Não foi possível apagar a pasta de frames temporária: {}", e));
    }

    logs.push("Limpeza concluída.".to_string());
}
