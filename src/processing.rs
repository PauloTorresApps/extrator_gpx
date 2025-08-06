use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::error::Error;

use chrono::{DateTime, Utc};
use gpx::{Gpx, Waypoint, read};

use crate::drawing::generate_speedometer_image;
use crate::utils::{calculate_speed_kmh, get_video_time_range, calculate_g_force, calculate_bearing, interpolate_gpx_points};

pub struct FrameInfo {
    path: String,
    timestamp_sec: f64,
}

pub fn run_processing(gpx_path: PathBuf, video_path: PathBuf, sync_timestamp_str: String, overlay_position: String) -> Result<Vec<String>, (String, Vec<String>)> {
    let mut logs = Vec::new();
    
    // Usamos uma função interna para facilitar o tratamento de erros com `?`
    // e ainda assim capturar os logs.
    match process_internal(gpx_path.clone(), video_path.clone(), &mut logs, sync_timestamp_str, overlay_position) {
        Ok(_) => {
            logs.push("Processo concluído com sucesso!".to_string());
            cleanup_files(&gpx_path, &mut logs);
            Ok(logs)
        },
        Err(e) => {
            let error_message = e.to_string();
            logs.push(format!("Ocorreu um erro: {}", error_message));
            // Mesmo em caso de erro, tenta limpar os ficheiros.
            cleanup_files(&gpx_path, &mut logs);
            Err((error_message, logs))
        }
    }
}

fn process_internal(gpx_path: PathBuf, video_path: PathBuf, logs: &mut Vec<String>, sync_timestamp_str: String, overlay_position: String) -> Result<(), Box<dyn Error>> {
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
    let original_gpx: Gpx = read(BufReader::new(File::open(&gpx_path)?))?;
    logs.push("Ficheiro GPX lido com sucesso!".to_string());
    
    // *** NOVA FUNCIONALIDADE: Interpolação de pontos GPX ***
    logs.push("A analisar intervalos entre pontos GPX...".to_string());
    let total_original_points: usize = original_gpx.tracks.iter()
        .flat_map(|track| track.segments.iter())
        .map(|segment| segment.points.len())
        .sum();
    logs.push(format!("Pontos originais no GPX: {}", total_original_points));
    
    // Interpolar pontos para garantir máximo de 2 segundos entre pontos
    let gpx = interpolate_gpx_points(original_gpx, 2);
    
    let total_interpolated_points: usize = gpx.tracks.iter()
        .flat_map(|track| track.segments.iter())
        .map(|segment| segment.points.len())
        .sum();
    let added_points = total_interpolated_points - total_original_points;
    logs.push(format!("Pontos após interpolação: {} (adicionados: {})", total_interpolated_points, added_points));
    logs.push("Interpolação de pontos GPX concluída!".to_string());
    // *** Fim da nova funcionalidade ***
    
    let mut frame_infos: Vec<FrameInfo> = Vec::new();
    let mut frame_counter = 0;

    for (track_idx, track) in gpx.tracks.iter().enumerate() {
        for (segment_idx, segment) in track.segments.iter().enumerate() {
            let synced_points: Vec<&Waypoint> = segment.points.iter().filter(|point| {
                if let Some(time_str) = point.time.as_ref().and_then(|t| t.format().ok()) {
                    if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                        let adjusted_point_time = point_time - time_offset;
                        adjusted_point_time >= video_start_time && adjusted_point_time <= video_end_time
                    } else { false }
                } else { false }
            }).collect();

            if synced_points.len() < 3 { continue; }
            logs.push(format!("Segmento {}/{} - Encontrados {} pontos sincronizados. A gerar imagens...", track_idx + 1, segment_idx + 1, synced_points.len()));

            for i in 1..synced_points.len() - 1 {
                let p1 = synced_points[i - 1];
                let p2 = synced_points[i];
                let p3 = synced_points[i + 1];

                let speed_kmh = calculate_speed_kmh(p1, p2).unwrap_or(0.0);
                let g_force = calculate_g_force(p1, p2, p3).unwrap_or(0.0);
                let bearing = calculate_bearing(p1, p2);
                let elevation = p2.elevation.unwrap_or(0.0);

                let output_path = format!("{}/frame_{:05}.png", output_dir, frame_counter);
                frame_counter += 1;
                generate_speedometer_image(speed_kmh, bearing, g_force, elevation, &output_path)?;

                if let Some(time_str) = p2.time.as_ref().and_then(|t| t.format().ok()) {
                     if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                        let adjusted_point_time = point_time - time_offset;
                        let timestamp_sec = (adjusted_point_time - video_start_time).num_milliseconds() as f64 / 1000.0;
                        frame_infos.push(FrameInfo { path: output_path, timestamp_sec });
                    }
                }
            }
        }
    }

    if !frame_infos.is_empty() {
        logs.push(format!("Geração de {} imagens concluída!", frame_infos.len()));
        logs.push("A gerar o vídeo final...".to_string());
        generate_final_video(&video_path, &frame_infos, &overlay_position)?;
        logs.push("Vídeo final gerado com sucesso!".to_string());
    } else {
        logs.push("Nenhum frame foi gerado.".to_string());
    }

    Ok(())
}

fn generate_final_video(video_path: &Path, frame_infos: &[FrameInfo], position: &str) -> Result<(), Box<dyn Error>> {
    if frame_infos.is_empty() { return Ok(()); }

    let overlay_coords = match position {
        "top-left" => "10:10",
        "top-right" => "main_w-overlay_w-10:10",
        "bottom-right" => "main_w-overlay_w-10:main_h-overlay_h-10",
        _ => "10:main_h-overlay_h-10", // Padrão: inferior esquerdo
    };

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
            "{}[{}:v]overlay={}:enable='between(t,{},{})'{}",
            last_stream, i + 1, overlay_coords, start_time, end_time,
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

fn cleanup_files(gpx_path: &Path, logs: &mut Vec<String>) {
    logs.push("A limpar ficheiros temporários...".to_string());

    if let Some(upload_dir) = gpx_path.parent() {
        if let Err(e) = fs::remove_dir_all(upload_dir) {
            logs.push(format!("Aviso: Não foi possível apagar a pasta de uploads: {}", e));
        }
    }

    if let Err(e) = fs::remove_dir_all("output_frames") {
        logs.push(format!("Aviso: Não foi possível apagar a pasta de frames temporária: {}", e));
    }

    logs.push("Limpeza concluída.".to_string());
}