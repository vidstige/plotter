use nalgebra_glm::Vec2;

use crate::polyline::Polyline2;

// Straight uv-polyline split into n line segments
pub fn grid_line(start: Vec2, end: Vec2, n: usize) -> Polyline2 {
    Polyline2 { points: (0..n)
        .map(|i| i as f32 / (n - 1) as f32)
        .map(|t| (1.0 - t) * start + t * end)
        .collect()
    }
}
