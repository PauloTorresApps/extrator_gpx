use std::error::Error;
use image::{Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::point::Point;
use imageproc::drawing::{draw_polygon_mut, draw_filled_circle_mut, draw_line_segment_mut, draw_text_mut};
use rusttype::{Font, Scale};

// A função agora aceita a direção (bearing) e a força G
pub fn generate_speedometer_image(speed_kmh: f64, bearing: f64, g_force: f64, elevation: f64, output_path: &str) -> Result<(), Box<dyn Error>> {
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
    
    let compass_radius = 40.0 * SCALE_FACTOR as f32;
    let bearing_rad = bearing.to_radians() as f32;
    draw_centered_text_mut(&mut img, white, CENTER.0, CENTER.1 - (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, scale_text, &font_regular, "N");
    draw_centered_text_mut(&mut img, white, CENTER.0, CENTER.1 + (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, scale_text, &font_regular, "S");
    draw_centered_text_mut(&mut img, white, CENTER.0 + (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, CENTER.1, scale_text, &font_regular, "L");
    draw_centered_text_mut(&mut img, white, CENTER.0 - (compass_radius + 10.0 * SCALE_FACTOR as f32) as i32, CENTER.1, scale_text, &font_regular, "O");

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

    let g_force_text = format!("{:.1} g", g_force);
    let g_force_center = ((45 * SCALE_FACTOR as i32), (35 * SCALE_FACTOR as i32));
    draw_filled_circle_mut(&mut img, g_force_center, 30 * SCALE_FACTOR as i32, transparent_black);
    draw_centered_text_mut(&mut img, white, g_force_center.0, g_force_center.1, Scale::uniform(20.0 * SCALE_FACTOR as f32), &font_regular, &g_force_text);
    
    // --- NOVO: Bloco de Desenho da Elevação ---
    let elevation_text = format!("{:.0} m", elevation);
    let elevation_center = (
        (45 * SCALE_FACTOR as i32), // Mesma posição X da Força G
        (IMG_SIZE as i32 - (45 * SCALE_FACTOR as i32)) // Posição Y no canto inferior
    );
    draw_filled_circle_mut(&mut img, elevation_center, 30 * SCALE_FACTOR as i32, transparent_black);
    draw_centered_text_mut(&mut img, white, elevation_center.0, elevation_center.1 - (5 * SCALE_FACTOR as i32), Scale::uniform(16.0 * SCALE_FACTOR as f32), &font_regular, "ALT");
    draw_centered_text_mut(&mut img, white, elevation_center.0, elevation_center.1 + (10 * SCALE_FACTOR as i32), Scale::uniform(20.0 * SCALE_FACTOR as f32), &font_bold, &elevation_text);
    // --- Fim do Bloco ---

    let final_img = image::imageops::resize(
        &img,
        FINAL_IMG_SIZE,
        FINAL_IMG_SIZE,
        FilterType::Lanczos3,
    );

    final_img.save(output_path)?;
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
    let s = angle_rad.sin(); let c = angle_rad.cos();
    let px = (point.x - center.0) as f32; let py = (point.y - center.1) as f32;
    let x_new = px * c - py * s; let y_new = px * s + py * c;
    Point { x: (x_new + center.0 as f32) as i32, y: (y_new + center.1 as f32) as i32 }
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
    let green = [127.0, 255.0, 0.0]; let yellow = [255.0, 255.0, 0.0]; let red = [255.0, 0.0, 0.0];
    let (r, g, b) = if ratio < 0.5 {
        let t = ratio * 2.0;
        (green[0] * (1.0 - t) + yellow[0] * t, green[1] * (1.0 - t) + yellow[1] * t, green[2] * (1.0 - t) + yellow[2] * t)
    } else {
        let t = (ratio - 0.5) * 2.0;
        (yellow[0] * (1.0 - t) + red[0] * t, yellow[1] * (1.0 - t) + red[1] * t, yellow[2] * (1.0 - t) + red[2] * t)
    };
    Rgba([r as u8, g as u8, b as u8, 255])
}
