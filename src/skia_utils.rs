use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::polyline::{Polyline2, Polyline3};


pub fn draw_polyline(pixmap: &mut Pixmap, polyline: &Polyline2, paint: &Paint, stroke: &Stroke, transform: Transform) {
    let mut pb = PathBuilder::new();
    for (index, point) in polyline.points.iter().enumerate() {
        if index == 0 {
            pb.move_to(point.x, point.y);
        } else {
            pb.line_to(point.x, point.y);
        }
    }
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
    }
}

pub fn draw_polylines(pixmap: &mut Pixmap, polylines: &[Polyline2], paint: &Paint, stroke: &Stroke, transform: Transform) {
    for polyline in polylines {
        draw_polyline(pixmap, polyline, paint, stroke, transform);
    }
}

pub fn draw_polylines_z(
    pixmap: &mut Pixmap,
    polylines: &[Polyline3],
    paint: &Paint,
    stroke: &Stroke,
    transform: Transform,
) {
    for screen_polyline in polylines {
        if screen_polyline.points.is_empty() {
            continue;
        }

        let mut depth_sum = 0.0;
        let mut xy_polyline = Polyline2::new();
        for screen in &screen_polyline.points {
            depth_sum += screen.z;
            xy_polyline.add(screen.xy());
        }

        let mean_depth = depth_sum / xy_polyline.points.len() as f32;
        let mut stroke = stroke.clone();
        stroke.width = stroke.width / mean_depth;
        draw_polyline(pixmap, &xy_polyline, paint, &stroke, transform);
    }
}
