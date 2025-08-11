use std::error::Error;
use image::{Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::point::Point;
use imageproc::drawing::{draw_polygon_mut, draw_filled_circle_mut, draw_line_segment_mut, draw_text_mut};
use rusttype::{Font, Scale};
use gpx::Gpx;
use chrono::{DateTime, Utc, FixedOffset};
use crate::utils::calculate_speed_kmh;

pub fn generate_speedometer_image(speed_kmh: f64, bearing: f64, g_force: f64, elevation: f64, output_path: &str, lang: &str) -> Result<(), Box<dyn Error>> {
    const SCALE_FACTOR: u32 = 4;
    const FINAL_IMG_SIZE: u32 = 300;
    const IMG_SIZE: u32 = FINAL_IMG_SIZE * SCALE_FACTOR;

    const CENTER: (i32, i32) = (IMG_SIZE as i32 / 2, IMG_SIZE as i32 / 2);
    const RADIUS: f32 = 120.0 * SCALE_FACTOR as f32;
    const MAX_SPEED: f64 = 120.0;

    let mut img = RgbaImage::new(IMG_SIZE, IMG_SIZE);
    let white = Rgba([255u8, 255, 255, 255]);
    let transparent_black = Rgba([0u8, 0, 0, 180]);
    let blue_arc = Rgba([0, 150, 255, 255]);

    let font_data_regular = include_bytes!("../DejaVuSans.ttf");
    let font_data_bold = include_bytes!("../DejaVuSans-Bold.ttf");
    let font_regular = Font::try_from_bytes(&font_data_regular[..]).ok_or("Falha ao carregar a fonte regular")?;
    let font_bold = Font::try_from_bytes(&font_data_bold[..]).ok_or("Falha ao carregar a fonte em negrito")?;
    
    draw_filled_circle_mut(&mut img, CENTER, (RADIUS + 15.0 * SCALE_FACTOR as f32) as i32, transparent_black);

    let speed_ratio = speed_kmh / MAX_SPEED;
    let start_angle = 90.0;
    let sweep_angle = speed_ratio * 270.0;
    draw_arc_mut(&mut img, CENTER, RADIUS as i32, start_angle, sweep_angle, blue_arc, 12 * SCALE_FACTOR as i32);

    let end_angle_rad = (start_angle + sweep_angle).to_radians() as f32;
    let (end_x, end_y) = (
        CENTER.0 as f32 + RADIUS as f32 * end_angle_rad.cos(),
        CENTER.1 as f32 + RADIUS as f32 * end_angle_rad.sin(),
    );
    draw_filled_circle_mut(&mut img, (end_x as i32, end_y as i32), 6 * SCALE_FACTOR as i32, white);

    let scale_text = Scale::uniform(22.0 * SCALE_FACTOR as f32);
    for i in 0..=MAX_SPEED as i32 {
        if i % 5 == 0 {
            let angle = 90.0 + (i as f64 / MAX_SPEED) * 270.0;
            let rad = angle.to_radians() as f32;
            let tick_length = if i % 25 == 0 { 15.0 } else { 8.0 } * SCALE_FACTOR as f32;
            let (x1, y1) = (CENTER.0 as f32 + rad.cos() * (RADIUS - tick_length), CENTER.1 as f32 + rad.sin() * (RADIUS - tick_length));
            let (x2, y2) = (CENTER.0 as f32 + rad.cos() * RADIUS, CENTER.1 as f32 + rad.sin() * RADIUS);
            draw_line_segment_mut(&mut img, (x1, y1), (x2, y2), white);
            
            if i % 25 == 0 {
                let (tx, ty) = (CENTER.0 as f32 + rad.cos() * (RADIUS - 35.0 * SCALE_FACTOR as f32), CENTER.1 as f32 + rad.sin() * (RADIUS - 35.0 * SCALE_FACTOR as f32));
                draw_centered_text_mut(&mut img, white, tx as i32, ty as i32, scale_text, &font_bold, &i.to_string());
            }
        }
    }
    
    // Bússola internacionalizada
    let (east_label, west_label) = if lang == "en" { ("E", "W") } else { ("L", "O") };
    let compass_radius = 40.0 * SCALE_FACTOR as f32;
    let bearing_rad = bearing.to_radians() as f32;
    draw_centered_text_mut(&mut img, white, CENTER.0, CENTER.1 - (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, scale_text, &font_regular, "N");
    draw_centered_text_mut(&mut img, white, CENTER.0, CENTER.1 + (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, scale_text, &font_regular, "S");
    draw_centered_text_mut(&mut img, white, CENTER.0 + (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, CENTER.1, scale_text, &font_regular, east_label);
    draw_centered_text_mut(&mut img, white, CENTER.0 - (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, CENTER.1, scale_text, &font_regular, west_label);

    let p_n = Point { x: CENTER.0, y: CENTER.1 - compass_radius as i32 };
    let p_s = Point { x: CENTER.0, y: CENTER.1 + (15 * SCALE_FACTOR as i32) };
    let p_e = Point { x: CENTER.0 + (15 * SCALE_FACTOR as i32), y: CENTER.1 };
    let p_w = Point { x: CENTER.0 - (15 * SCALE_FACTOR as i32), y: CENTER.1 };
    
    let rotated_n = rotate_point(p_n, CENTER, bearing_rad);
    let rotated_s = rotate_point(p_s, CENTER, bearing_rad);
    let rotated_e = rotate_point(p_e, CENTER, bearing_rad);
    let rotated_w = rotate_point(p_w, CENTER, bearing_rad);

    draw_polygon_mut(&mut img, &[rotated_n, rotated_e, rotated_s, rotated_w], Rgba([255, 0, 0, 255]));

    let speed_color = speed_to_color(speed_kmh, MAX_SPEED);
    let scale_speed = Scale::uniform(60.0 * SCALE_FACTOR as f32);
    let speed_text = format!("{:.0}", speed_kmh);
    draw_text_mut(&mut img, speed_color, CENTER.0 + (30 * SCALE_FACTOR as i32), CENTER.1 + (50 * SCALE_FACTOR as i32), scale_speed, &font_bold, &speed_text);
    let scale_unit = Scale::uniform(20.0 * SCALE_FACTOR as f32);
    draw_text_mut(&mut img, white, CENTER.0 + (35 * SCALE_FACTOR as i32), CENTER.1 + (100 * SCALE_FACTOR as i32), scale_unit, &font_regular, "KM/H");

    // Coordenadas ajustadas para os marcadores de Força G e Altitude
    let g_force_text = format!("{:.1} g", g_force);
    let g_force_center = ((40 * SCALE_FACTOR as i32), (40 * SCALE_FACTOR as i32));
    draw_filled_circle_mut(&mut img, g_force_center, 35 * SCALE_FACTOR as i32, transparent_black);
    draw_centered_text_mut(&mut img, white, g_force_center.0, g_force_center.1, Scale::uniform(20.0 * SCALE_FACTOR as f32), &font_regular, &g_force_text);
    
    let elevation_text = format!("{:.0} m", elevation);
    let elevation_center = (
        (40 * SCALE_FACTOR as i32),
        (IMG_SIZE as i32 - (40 * SCALE_FACTOR as i32))
    );
    draw_filled_circle_mut(&mut img, elevation_center, 35 * SCALE_FACTOR as i32, transparent_black);
    draw_centered_text_mut(&mut img, white, elevation_center.0, elevation_center.1 - (5 * SCALE_FACTOR as i32), Scale::uniform(16.0 * SCALE_FACTOR as f32), &font_regular, "ALT");
    draw_centered_text_mut(&mut img, white, elevation_center.0, elevation_center.1 + (10 * SCALE_FACTOR as i32), Scale::uniform(20.0 * SCALE_FACTOR as f32), &font_bold, &elevation_text);

    let final_img = image::imageops::resize(
        &img,
        FINAL_IMG_SIZE,
        FINAL_IMG_SIZE,
        FilterType::Lanczos3,
    );
    final_img.save(output_path)?;

    Ok(())
}

// FUNÇÃO MODIFICADA: `generate_stats_image` agora converte o fuso horário e exibe a data
pub fn generate_stats_image(
    distance_km: f64,
    altitude_m: f64,
    elevation_gain_m: f64,
    current_time_utc: DateTime<Utc>,
    output_path: &str,
    lang: &str,
) -> Result<(), Box<dyn Error>> {
    const WIDTH: u32 = 220;
    const HEIGHT: u32 = 250;
    let mut img = RgbaImage::new(WIDTH, HEIGHT);

    // Fundo transparente
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    let white = Rgba([255, 255, 255, 255]);
    let font_data_bold = include_bytes!("../DejaVuSans-Bold.ttf");
    let font_bold = Font::try_from_bytes(&font_data_bold[..]).ok_or("Falha ao carregar a fonte em negrito")?;

    let scale_label = Scale::uniform(16.0); 
    let scale_value = Scale::uniform(28.0);
    let scale_sub_value = Scale::uniform(18.0); // Nova escala para a data
    let y_start = 15;
    let line_height = 60;

    // Distância
    let distance_label = if lang == "en" { "DISTANCE" } else { "DISTÂNCIA" };
    let distance_value_unit = format!("{:.1} KM", distance_km);
    draw_text_mut(&mut img, white, 10, y_start, scale_label, &font_bold, distance_label);
    draw_text_mut(&mut img, white, 10, y_start + 20, scale_value, &font_bold, &distance_value_unit);

    // Altitude
    let altitude_label = if lang == "en" { "ALTITUDE" } else { "ALTITUDE" };
    let altitude_value_unit = format!("{:.0} M", altitude_m);
    draw_text_mut(&mut img, white, 10, y_start + line_height, scale_label, &font_bold, altitude_label);
    draw_text_mut(&mut img, white, 10, y_start + line_height + 20, scale_value, &font_bold, &altitude_value_unit);

    // Ganho de elevação
    let elevation_gain_label = if lang == "en" { "ELEVATION GAIN" } else { "GANHO DE ELEVAÇÃO" };
    let elevation_gain_value_unit = format!("{:.0} M", elevation_gain_m);
    draw_text_mut(&mut img, white, 10, y_start + line_height * 2, scale_label, &font_bold, elevation_gain_label);
    draw_text_mut(&mut img, white, 10, y_start + line_height * 2 + 20, scale_value, &font_bold, &elevation_gain_value_unit);

    // Horário e Data
    let brt_offset = FixedOffset::west_opt(3 * 3600).unwrap(); // Fuso horário UTC-3 (Horário de Brasília)
    let local_time = current_time_utc.with_timezone(&brt_offset);

    let time_text = local_time.format("%H:%M").to_string();
    let date_text = local_time.format("%d/%m/%Y").to_string();

    draw_text_mut(&mut img, white, 10, y_start + line_height * 3, scale_value, &font_bold, &time_text);
    draw_text_mut(&mut img, white, 10, y_start + line_height * 3 + 25, scale_sub_value, &font_bold, &date_text);

    img.save(output_path)?;
    Ok(())
}


pub fn generate_dot_image(path: &str, size: u32, color: Rgba<u8>) -> Result<(), Box<dyn Error>> {
    let mut img = RgbaImage::new(size, size);
    let center = (size as i32 / 2, size as i32 / 2);
    let radius = (size / 2) as i32 - (size as i32 / 10);

    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    draw_filled_circle_mut(&mut img, center, radius, color);
    img.save(path)?;
    Ok(())
}

fn draw_thick_line_segment_mut(
    image: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
    color: Rgba<u8>,
) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1e-6 { return; }

    let nx = dx / length;
    let ny = dy / length;

    let px = -ny;
    let py = nx;

    let half_thickness = thickness / 2.0;

    let p1 = Point { x: (start.0 + px * half_thickness) as i32, y: (start.1 + py * half_thickness) as i32 };
    let p2 = Point { x: (end.0 + px * half_thickness) as i32, y: (end.1 + py * half_thickness) as i32 };
    let p3 = Point { x: (end.0 - px * half_thickness) as i32, y: (end.1 - py * half_thickness) as i32 };
    let p4 = Point { x: (start.0 - px * half_thickness) as i32, y: (start.1 - py * half_thickness) as i32 };

    draw_polygon_mut(image, &[p1, p2, p3, p4], color);
}

// NOVA FUNÇÃO: Gera a cor com base na velocidade para o mapa de calor
fn speed_to_gradient_color(speed: f64, min_speed: f64, max_speed: f64) -> Rgba<u8> {
    if max_speed <= min_speed {
        return Rgba([0, 0, 255, 255]); // Azul por defeito se não houver variação
    }

    // Normaliza a velocidade entre 0.0 e 1.0
    let ratio = (speed - min_speed) / (max_speed - min_speed);

    // Define as cores para o gradiente
    let blue = [0.0, 0.0, 255.0];
    let green = [0.0, 255.0, 0.0];
    let yellow = [255.0, 255.0, 0.0];
    let orange = [255.0, 165.0, 0.0];
    let red = [255.0, 0.0, 0.0];

    let (r, g, b) = if ratio < 0.25 { // Azul para Verde
        let t = ratio * 4.0;
        (blue[0] * (1.0 - t) + green[0] * t, blue[1] * (1.0 - t) + green[1] * t, blue[2] * (1.0 - t) + green[2] * t)
    } else if ratio < 0.5 { // Verde para Amarelo
        let t = (ratio - 0.25) * 4.0;
        (green[0] * (1.0 - t) + yellow[0] * t, green[1] * (1.0 - t) + yellow[1] * t, green[2] * (1.0 - t) + yellow[2] * t)
    } else if ratio < 0.75 { // Amarelo para Laranja
        let t = (ratio - 0.5) * 4.0;
        (yellow[0] * (1.0 - t) + orange[0] * t, yellow[1] * (1.0 - t) + orange[1] * t, yellow[2] * (1.0 - t) + orange[2] * t)
    } else { // Laranja para Vermelho
        let t = (ratio - 0.75) * 4.0;
        (orange[0] * (1.0 - t) + red[0] * t, orange[1] * (1.0 - t) + red[1] * t, orange[2] * (1.0 - t) + red[2] * t)
    };

    Rgba([r as u8, g as u8, b as u8, 255])
}


// FUNÇÃO MODIFICADA: `generate_track_map_image` agora usa o gradiente de cores
pub fn generate_track_map_image(
    gpx: &Gpx,
    width: u32,
    height: u32,
    path: &str,
    background_color: Rgba<u8>,
    line_thickness: f32,
) -> Result<(), Box<dyn Error>> {
    let all_points: Vec<_> = gpx.tracks.iter()
        .flat_map(|t| t.segments.iter())
        .flat_map(|s| s.points.iter())
        .collect();

    if all_points.len() < 2 {
        return Err("GPX não contém pontos suficientes para desenhar.".into());
    }

    // Calcula todas as velocidades para encontrar o mínimo e o máximo
    let speeds: Vec<Option<f64>> = all_points
        .windows(2)
        .map(|p| calculate_speed_kmh(p[0], p[1]))
        .collect();
    
    let valid_speeds: Vec<f64> = speeds.iter().filter_map(|&s| s).collect();
    let min_speed = if valid_speeds.is_empty() { 0.0 } else { valid_speeds.iter().fold(f64::INFINITY, |a, &b| a.min(b)) };
    let max_speed = if valid_speeds.is_empty() { 0.0 } else { valid_speeds.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) };

    let (min_lon, max_lon, min_lat, max_lat) = all_points.iter().fold(
        (all_points[0].point().x(), all_points[0].point().x(), all_points[0].point().y(), all_points[0].point().y()),
        |(min_x, max_x, min_y, max_y), p| {
            (min_x.min(p.point().x()), max_x.max(p.point().x()), min_y.min(p.point().y()), max_y.max(p.point().y()))
        }
    );

    let mut img = RgbaImage::from_pixel(width, height, background_color);
    let padding = 20.0;
    let map_width = width as f64 - 2.0 * padding;
    let map_height = height as f64 - 2.0 * padding;

    let lon_range = max_lon - min_lon;
    let lat_range = max_lat - min_lat;

    let scale_x = if lon_range.abs() > 1e-9 { map_width / lon_range } else { 0.0 };
    let scale_y = if lat_range.abs() > 1e-9 { map_height / lat_range } else { 0.0 };
    let scale = scale_x.min(scale_y);

    let get_pixel_coords = |lon: f64, lat: f64| -> (f32, f32) {
        let x = padding + (lon - min_lon) * scale;
        let y = padding + (max_lat - lat) * scale;
        (x as f32, y as f32)
    };

    let mut point_idx = 0;
    for track in &gpx.tracks {
        for segment in &track.segments {
            for pair in segment.points.windows(2) {
                let p1 = &pair[0]; // CORRIGIDO: Emprestando o valor
                let p2 = &pair[1]; // CORRIGIDO: Emprestando o valor
                let (x1, y1) = get_pixel_coords(p1.point().x(), p1.point().y());
                let (x2, y2) = get_pixel_coords(p2.point().x(), p2.point().y());

                let line_color = match speeds.get(point_idx).and_then(|s| *s) {
                    Some(speed) => speed_to_gradient_color(speed, min_speed, max_speed),
                    None => Rgba([0, 0, 255, 255]), // Azul por defeito
                };
                
                draw_thick_line_segment_mut(&mut img, (x1, y1), (x2, y2), line_thickness, line_color);
                point_idx += 1;
            }
        }
    }

    img.save(path)?;
    Ok(())
}


fn draw_arc_mut(image: &mut RgbaImage, center: (i32, i32), radius: i32, start_angle_deg: f64, sweep_angle_deg: f64, color: Rgba<u8>, thickness: i32) {
    let steps = (sweep_angle_deg.abs() * 2.0).ceil() as i32;
    for i in 0..=steps {
        let percent = i as f64 / steps as f64;
        let current_angle = (start_angle_deg + sweep_angle_deg * percent).to_radians();
        let x = center.0 as f32 + radius as f32 * current_angle.cos() as f32;
        let y = center.1 as f32 + radius as f32 * current_angle.sin() as f32;
        draw_filled_circle_mut(image, (x as i32, y as i32), thickness / 2, color);
    }
}

fn rotate_point(point: Point<i32>, center: (i32, i32), angle_rad: f32) -> Point<i32> {
    let s = angle_rad.sin();
    let c = angle_rad.cos();
    let px = (point.x - center.0) as f32;
    let py = (point.y - center.1) as f32;
    let x_new = px * c - py * s;
    let y_new = px * s + py * c;
    Point {
        x: (x_new + center.0 as f32) as i32,
        y: (y_new + center.1 as f32) as i32
    }
}

fn draw_centered_text_mut(img: &mut RgbaImage, color: Rgba<u8>, x: i32, y: i32, scale: Scale, font: &Font, text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let v_metrics = font.v_metrics(scale);
    let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let total_height = line_height * lines.len() as f32;

    for (i, line) in lines.iter().enumerate() {
        let glyphs: Vec<_> = font.layout(line, scale, rusttype::point(0.0, v_metrics.ascent)).collect();
        let text_width = glyphs.iter().map(|g| g.unpositioned().h_metrics().advance_width).sum::<f32>();
        let final_x = x as f32 - text_width / 2.0;
        let final_y = (y as f32 - total_height / 2.0) + (i as f32 * line_height);
        draw_text_mut(img, color, final_x as i32, final_y as i32, scale, font, line);
    }
}

fn speed_to_color(speed: f64, max_speed: f64) -> Rgba<u8> {
    let ratio = (speed / max_speed).min(1.0);
    let green = [127.0, 255.0, 0.0];
    let yellow = [255.0, 255.0, 0.0];
    let red = [255.0, 0.0, 0.0];

    let (r, g, b) = if ratio < 0.5 {
        let t = ratio * 2.0;
        (
            green[0] * (1.0 - t) + yellow[0] * t,
            green[1] * (1.0 - t) + yellow[1] * t,
            green[2] * (1.0 - t) + yellow[2] * t,
        )
    } else {
        let t = (ratio - 0.5) * 2.0;
        (
            yellow[0] * (1.0 - t) + red[0] * t,
            yellow[1] * (1.0 - t) + red[1] * t,
            yellow[2] * (1.0 - t) + red[2] * t,
        )
    };

    Rgba([r as u8, g as u8, b as u8, 255])
}
