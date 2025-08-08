use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::error::Error;

use chrono::{DateTime, Utc};
use gpx::{Gpx, Waypoint, read};
use image::Rgba;
use crate::drawing::{generate_speedometer_image, generate_track_map_image, generate_dot_image};
use crate::utils::{calculate_speed_kmh, get_video_time_range, calculate_g_force, calculate_bearing, interpolate_gpx_points};

pub struct FrameInfo {
    path: String,
    timestamp_sec: f64,
    gpx_point: Waypoint,
}

pub fn run_processing(
    gpx_path: PathBuf, 
    video_path: PathBuf, 
    sync_timestamp_str: String, 
    add_speedo_overlay: bool,
    speedo_position: String,
    add_track_overlay: bool,
    track_position: String,
) -> Result<Vec<String>, (String, Vec<String>)> {
    let mut logs = Vec::new();
    
    match process_internal(gpx_path.clone(), video_path.clone(), &mut logs, sync_timestamp_str, add_speedo_overlay, speedo_position, add_track_overlay, track_position) {
        Ok(_) => {
            logs.push("Processo concluído com sucesso!".to_string());
            cleanup_files(&gpx_path, &mut logs);
            Ok(logs)
        },
        Err(e) => {
            let error_message = e.to_string();
            logs.push(format!("Ocorreu um erro: {}", error_message));
            cleanup_files(&gpx_path, &mut logs);
            Err((error_message, logs))
        }
    }
}

fn process_internal(
    gpx_path: PathBuf, 
    video_path: PathBuf, 
    logs: &mut Vec<String>, 
    sync_timestamp_str: String, 
    add_speedo_overlay: bool,
    speedo_position: String,
    add_track_overlay: bool,
    track_position: String,
) -> Result<(), Box<dyn Error>> {
    let output_dir = "output_frames";
    let final_video_dir = "output";
    let map_assets_dir = "output_map_assets";
    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(final_video_dir)?;
    fs::create_dir_all(map_assets_dir)?;

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
    
    let gpx = interpolate_gpx_points(original_gpx, 1);
    
    let map_image_path = format!("{}/track_base.png", map_assets_dir);
    let dot_image_path = format!("{}/marker_dot.png", map_assets_dir);
    if add_track_overlay {
        logs.push("A gerar imagem base do trajeto...".to_string());
        generate_track_map_image(&gpx, 300, 300, &map_image_path, Rgba([0, 0, 0, 100]), Rgba([57, 255, 20, 255]), 2.0)?;
        generate_dot_image(&dot_image_path, 8, Rgba([255, 0, 0, 255]))?;
        logs.push("Assets do mapa gerados.".to_string());
    }
    
    let mut frame_infos: Vec<FrameInfo> = Vec::new();
    if add_speedo_overlay || add_track_overlay {
        logs.push("A processar pontos GPX para gerar frames...".to_string());
        let mut frame_counter = 0;
        for track in gpx.tracks.iter() {
            for segment in track.segments.iter() {
                let all_points = &segment.points;
                if all_points.len() < 3 { continue; }

                for i in 1..all_points.len() - 1 {
                    let p2 = &all_points[i];
                    if let Some(time_str) = p2.time.as_ref().and_then(|t| t.format().ok()) {
                         if let Ok(point_time) = time_str.parse::<DateTime<Utc>>() {
                            let adjusted_point_time = point_time - time_offset;
                            if adjusted_point_time >= video_start_time && adjusted_point_time <= video_end_time {
                                let output_path = if add_speedo_overlay {
                                    let p1 = &all_points[i - 1];
                                    let p3 = &all_points[i + 1];
                                    let speed_kmh = calculate_speed_kmh(p1, p2).unwrap_or(0.0);
                                    let g_force = calculate_g_force(p1, p2, p3).unwrap_or(0.0);
                                    let bearing = calculate_bearing(p1, p2);
                                    let elevation = p2.elevation.unwrap_or(0.0);
                                    let path = format!("{}/frame_{:05}.png", output_dir, frame_counter);
                                    generate_speedometer_image(speed_kmh, bearing, g_force, elevation, &path)?;
                                    frame_counter += 1;
                                    path
                                } else {
                                    String::new()
                                };

                                let timestamp_sec = (adjusted_point_time - video_start_time).num_milliseconds() as f64 / 1000.0;
                                frame_infos.push(FrameInfo { path: output_path, timestamp_sec, gpx_point: p2.clone() });
                            }
                        }
                    }
                }
            }
        }
    }

    if !frame_infos.is_empty() {
        logs.push(format!("Geração de {} frames de dados concluída!", frame_infos.len()));
        logs.push("A gerar o vídeo final...".to_string());
        generate_final_video(&video_path, &frame_infos, add_speedo_overlay, &speedo_position, add_track_overlay, &track_position, &gpx, &map_image_path, &dot_image_path)?;
        logs.push("Vídeo final gerado com sucesso!".to_string());
    } else if !add_speedo_overlay && !add_track_overlay {
        logs.push("Nenhum overlay foi selecionado. A gerar cópia do vídeo original.".to_string());
        fs::copy(video_path, "output/output_video.mp4")?;
    } else {
        logs.push("Nenhum ponto GPX coincidiu com o tempo do vídeo. A gerar cópia do vídeo original.".to_string());
        fs::copy(video_path, "output/output_video.mp4")?;
    }

    Ok(())
}

fn get_position_coords(position: &str) -> String {
    match position {
        "top-left" => "10:10".to_string(),
        "top-right" => "main_w-overlay_w-10:10".to_string(),
        "bottom-right" => "main_w-overlay_w-10:main_h-overlay_h-10".to_string(),
        "bottom-left" | _ => "10:main_h-overlay_h-10".to_string(),
    }
}

fn generate_final_video(
    video_path: &Path,
    frame_infos: &[FrameInfo],
    add_speedo_overlay: bool,
    speedo_position: &str,
    add_track_overlay: bool,
    track_position: &str,
    gpx: &Gpx,
    map_image_path: &str,
    dot_image_path: &str,
) -> Result<(), Box<dyn Error>> {
    if frame_infos.is_empty() { return Ok(()); }

    let mut complex_filter = String::new();
    let mut inputs: Vec<String> = vec!["-i".to_string(), video_path.to_str().unwrap().to_string()];
    let mut last_stream = "[0:v]".to_string();
    
    if add_speedo_overlay {
        let speedo_coords = get_position_coords(speedo_position);
        for (i, info) in frame_infos.iter().enumerate() {
            inputs.push("-i".to_string());
            inputs.push(info.path.clone());
            let input_idx = inputs.len() / 2 - 1;
            let output_stream = format!("[v_s_{}]", i);
            let end_time = frame_infos.get(i + 1).map_or(info.timestamp_sec + 1.0, |ni| ni.timestamp_sec);

            complex_filter.push_str(&format!(";{}[{}:v]overlay={}:enable='between(t,{},{})'{}", last_stream, input_idx, speedo_coords, info.timestamp_sec, end_time, output_stream));
            last_stream = output_stream;
        }
    }

    if add_track_overlay {
        let map_coords = get_position_coords(track_position);
        let map_input_idx = inputs.len() / 2;
        inputs.push("-i".to_string());
        inputs.push(map_image_path.to_string());
        
        let dot_input_idx = inputs.len() / 2;
        inputs.push("-i".to_string());
        inputs.push(dot_image_path.to_string());
        
        let map_stream_name = "[with_map]";
        complex_filter.push_str(&format!(";{}[{}:v]overlay={}{}", last_stream, map_input_idx, map_coords, map_stream_name));
        last_stream = map_stream_name.to_string();

        let all_points: Vec<_> = gpx.tracks.iter().flat_map(|t| t.segments.iter()).flat_map(|s| s.points.iter()).collect();
        let (min_lon, max_lon, min_lat, max_lat) = all_points.iter().fold(
            (all_points[0].point().x(), all_points[0].point().x(), all_points[0].point().y(), all_points[0].point().y()),
            |(min_x, max_x, min_y, max_y), p| (min_x.min(p.point().x()), max_x.max(p.point().x()), min_y.min(p.point().y()), max_y.max(p.point().y()))
        );
        let (map_width, map_height, padding) = (300.0, 300.0, 20.0);
        let lon_range = max_lon - min_lon; let lat_range = max_lat - min_lat;
        let scale = if lon_range.abs() > 1e-9 && lat_range.abs() > 1e-9 {
            ((map_width - 2.0 * padding) / lon_range).min((map_height - 2.0 * padding) / lat_range)
        } else { 0.0 };
        
        // --- INÍCIO DA CORREÇÃO: Lógica de posicionamento do marcador ---
        // Substitui 'overlay_w' e 'overlay_h' pelas dimensões fixas do mapa (300)
        let map_base_coords = map_coords.replace("overlay_w", "300").replace("overlay_h", "300");
        let map_coords_parts: Vec<&str> = map_base_coords.split(':').collect();
        let map_base_x = map_coords_parts.get(0).cloned().unwrap_or("0");
        let map_base_y = map_coords_parts.get(1).cloned().unwrap_or("0");

        for (i, info) in frame_infos.iter().enumerate() {
            let point = &info.gpx_point;
            let (lon, lat) = (point.point().x(), point.point().y());

            let dot_x_on_map = padding + (lon - min_lon) * scale - 4.0;
            let dot_y_on_map = padding + (max_lat - lat) * scale - 4.0;
            
            // Envolve as expressões base em parênteses para garantir a ordem correta de operações no FFmpeg
            let final_dot_x = format!("({}) + {:.2}", map_base_x, dot_x_on_map);
            let final_dot_y = format!("({}) + {:.2}", map_base_y, dot_y_on_map);
            
            let end_time = frame_infos.get(i + 1).map_or(info.timestamp_sec + 1.0, |ni| ni.timestamp_sec);
            let output_stream = format!("[v_d_{}]", i);

            complex_filter.push_str(&format!(";{}[{}:v]overlay=x='{}':y='{}':enable='between(t,{},{})'{}", last_stream, dot_input_idx, final_dot_x, final_dot_y, info.timestamp_sec, end_time, output_stream));
            last_stream = output_stream;
        }
        // --- FIM DA CORREÇÃO ---
    }
    
    let final_filter = complex_filter.strip_prefix(';').unwrap_or(&complex_filter).to_string();
    let output_file = "output/output_video.mp4";
    if Path::new(output_file).exists() { fs::remove_file(output_file)?; }

    let status = StdCommand::new("ffmpeg").args(&inputs).arg("-filter_complex").arg(&final_filter).arg("-map").arg(&last_stream).arg("-c:a").arg("copy").arg(output_file).status()?;
    if !status.success() { return Err(format!("O comando FFmpeg falhou. Filtro: {}", final_filter).into()); }
    Ok(())
}

fn cleanup_files(gpx_path: &Path, logs: &mut Vec<String>) {
    logs.push("A limpar ficheiros temporários...".to_string());

    if let Some(upload_dir) = gpx_path.parent() {
        if upload_dir.ends_with("uploads") {
            if let Err(e) = fs::remove_dir_all(upload_dir) {
                logs.push(format!("Aviso: Não foi possível apagar a pasta de uploads: {}", e));
            }
        }
    }

    if let Err(e) = fs::remove_dir_all("output_frames") {
        logs.push(format!("Aviso: Não foi possível apagar a pasta de frames de telemetria: {}", e));
    }
    
    if let Err(e) = fs::remove_dir_all("output_map_assets") {
        logs.push(format!("Aviso: Não foi possível apagar a pasta de assets do mapa: {}", e));
    }

    logs.push("Limpeza concluída.".to_string());
}
