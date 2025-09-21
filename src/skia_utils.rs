use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::polyline::Polyline2;


pub fn draw_polyline(pixmap: &mut Pixmap, polyline: &Polyline2, paint: &Paint, stroke: &Stroke) {
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

pub fn draw_polylines(pixmap: &mut Pixmap, polylines: &[Polyline2], paint: &Paint, stroke: &Stroke) {
    for polyline in polylines {
        draw_polyline(pixmap, polyline, paint, stroke);
    }
}