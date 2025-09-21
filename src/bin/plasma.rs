use std::io::{self, Write};

use nalgebra_glm::Vec3;
use plotter::{
    field::Field, marching_squares::find_contours, resolution::Resolution,
    sdf_transform::sdf_from_pixmap, simplex::simplex3, skia_utils::draw_polylines,
};
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform};

fn sample_at(resolution: &Resolution, z: f32) -> Field<f32> {
    let mut values = Vec::with_capacity(resolution.area());
    for i in 0..resolution.height {
        for j in 0..resolution.width {
            let x = j as f32 / resolution.width as f32;
            let y = i as f32 / resolution.height as f32;
            let p = Vec3::new(x, y, z);
            let value = simplex3(&p);
            values.push(value);
        }
    }
    Field { resolution: resolution.clone(), values }
}

fn linspace(lo: f32, hi: f32, n: usize) -> Vec<f32> {
    let size = hi - lo;
    (0..n).map(|i| lo + size * (i as f32) / n as f32).collect()
}

fn hann(t: f32) -> f32 {
    (0.5 - 0.5 * (std::f32::consts::TAU * t).cos()).max(0.0)
}

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let level_count = 8;

    // Load mask
    let pattern = Pixmap::load_png("data/volumental.png")?;
    let tmp = sdf_from_pixmap(&pattern);
    let lo = tmp.values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let hi = tmp.values.iter().fold(-f32::INFINITY, |a, &b| a.max(b));
    let scale = hi.abs().max(lo.abs());
    let pattern_sdf = tmp * (1.0 / scale / level_count as f32);

    let resolution = Resolution::new(720, 720);
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    // setup stroke & paint
    let color = Color::BLACK;
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 8.0;

    let transform = Transform::from_scale(
        resolution.width as f32 / pattern.width() as f32,
        resolution.height as f32 / pattern.height() as f32,
    );

    let levels = linspace(-1.0, 1.0, level_count);
    let frames = 256;
    let speed = 4.0;
    for frame in 0..frames {
        let t = frame as f32 / frames as f32;
        let z = speed * t;

        let pattern_resolution = Resolution::new(pattern.width(), pattern.height());
        let simplex_sdf = sample_at(&pattern_resolution, z);
        // combine pattern sdf with field sdf
        let s = hann(t);
        let field = simplex_sdf * (1.0 - s) + &pattern_sdf * s;

        pixmap.fill(Color::WHITE);
        for level in &levels {
            let polylines = find_contours(&field, *level);
            draw_polylines(&mut pixmap, &polylines, &paint, &stroke, transform);
        }

        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
