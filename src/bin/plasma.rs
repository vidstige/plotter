use std::io::{self, Write};

use nalgebra_glm::Vec3;
use plotter::{field::Field, marching_squares::find_contours, polyline::Polyline2, resolution::Resolution, sdf_transform::sdf_from_pixmap, simplex::simplex3};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

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

fn draw_polyline(pixmap: &mut Pixmap, polyline: Polyline2, paint: &Paint, stroke: &Stroke) {
    let mut pb = PathBuilder::new();
    for (index, point) in polyline.points.iter().enumerate() {
        if index == 0 {
            pb.move_to(point.x, point.y);
        } else {
            pb.line_to(point.x, point.y);
        }
    }
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn linspace(lo: f32, hi: f32, n: usize) -> Vec<f32> {
    let size = hi - lo;
    (0..n).map(|i| lo + size * (i as f32) / n as f32).collect()
}

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();

    let resolution = Resolution::new(720, 720);
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    let color = Color::BLACK;
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 8.0;

    let levels = linspace(-1.0, 1.0, 8);
    let frames = 512;
    let speed = 4.0;
    for frame in 0..frames {
        let t = frame as f32 / frames as f32;
        let z = speed * t;

        pixmap.fill(Color::WHITE);
        let field = sample_at(&resolution, z);
        for level in &levels {
            let polylines = find_contours(&field, *level);
            
            for polyline in polylines {
                draw_polyline(&mut pixmap, polyline, &paint, &stroke);
            }
        }

        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    /*let mask = Pixmap::load_png("data/volumental.png")?;
    let sdf = sdf_from_pixmap(&mask);
    let polylines = find_contours(&sdf, 0.0);
    for polyline in polylines {
        draw_polyline(&mut pixmap, polyline, &paint, &stroke);
    }
    pixmap.save_png("output.png")?;*/

    Ok(())
}
