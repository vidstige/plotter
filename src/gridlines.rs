use nalgebra_glm::Vec2;

use crate::polyline::Polyline2;

// Straight uv-polyline split into n line segments
pub fn grid_line(start: Vec2, end: Vec2, n: usize) -> Polyline2 {
    Polyline2 {
        points: (0..n)
            .map(|i| i as f32 / (n - 1) as f32)
            .map(|t| (1.0 - t) * start + t * end)
            .collect(),
    }
}

pub fn generate_grid(
    u_range: (f32, f32),
    v_range: (f32, f32),
    n_lines: usize,
    segments_per_line: usize,
) -> Vec<Polyline2> {
    let (u0, u1) = u_range;
    let (v0, v1) = v_range;
    let mut lines = Vec::with_capacity(n_lines * 2);
    for i in 0..n_lines {
        let t = i as f32 / n_lines as f32;
        let u = (1.0 - t) * u0 + t * u1;
        let v = (1.0 - t) * v0 + t * v1;

        // Horizontal line: v fixed
        lines.push(grid_line(Vec2::new(u0, v), Vec2::new(u1, v), segments_per_line));
        // Vertical line: u fixed
        lines.push(grid_line(Vec2::new(u, v0), Vec2::new(u, v1), segments_per_line));
    }
    lines
}
