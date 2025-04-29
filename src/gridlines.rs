use nalgebra_glm::Vec2;

use crate::{geometry::{compute_gamma, Geometry}, polyline::Polyline2};

// Straight uv-polyline split into n line segments
pub fn grid_line(start: Vec2, end: Vec2, n: usize) -> Polyline2 {
    Polyline2 { points: (0..n)
        .map(|i| i as f32 / (n - 1) as f32)
        .map(|t| (1.0 - t) * start + t * end)
        .collect()
    }
}

fn grid_line_offset_at(gamma: &[[[f32; 2]; 2]; 2], direction: Vec2) -> Vec2 {
    let d = direction;
    let mut offset = Vec2::zeros();
    for i in 0..2 {
        offset[i] = gamma[i][0][0] * d.x * d.x
                  + 2.0 * gamma[i][0][1] * d.x * d.y
                  + gamma[i][1][1] * d.y * d.y;
    }
    offset
}

// Gridline bent by the local affine connection
pub fn bent_grid_line(
    geometry: &impl Geometry,
    start: Vec2,
    end: Vec2,
    amplitude: f32,
) -> Polyline2 {
    let mut polyline = Polyline2::new();

    let direction = (end - start).normalize();
    let mut t = 0.0;
    while t < 1.0 {
        // lerp position between start & end
        let p = (1.0 - t) * start + t * end;

        // Compute Christoffel symbols at current point
        let gamma = compute_gamma(geometry, &p);
        // Determine how much sideways push we want
        let offset = grid_line_offset_at(&gamma, direction);

        // Apply the sideways offset and add to polyline
        let pos = p + direction * amplitude + 0.5 * offset * amplitude * amplitude;

        polyline.add(pos);

        // Figure out step length
        // TODO: metric already computed inside compute_gamma
        let g = geometry.metric(&p);
        let eps = 1e-5;
        let base_step = 1.0 / 8.0;
        let step = base_step / (g.determinant().sqrt() + eps);
        t += step.clamp(1.0 / 1024.0, 1.0 / 8.0);
    }
    polyline
}