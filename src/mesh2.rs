use nalgebra_glm::Vec2;


pub struct Mesh2 {
    pub vertices: Vec<Vec2>,
    pub quads: Vec<[usize; 4]>,
}

impl Mesh2 {
    /// Creates a regular grid of quads over the given UV bounds.
    /// The grid will have `width` cells in the U direction and `height` in the V direction.
    pub fn from_grid(width: usize, height: usize, uv_min: Vec2, uv_max: Vec2) -> Self {
        let mut vertices = Vec::with_capacity((width + 1) * (height + 1));
        let mut quads = Vec::with_capacity(width * height);

        for j in 0..=height {
            let v = uv_min.y + (j as f32 / height as f32) * (uv_max.y - uv_min.y);
            for i in 0..=width {
                let u = uv_min.x + (i as f32 / width as f32) * (uv_max.x - uv_min.x);
                vertices.push(Vec2::new(u, v));
            }
        }

        for j in 0..height {
            for i in 0..width {
                let i0 = j * (width + 1) + i;
                let i1 = i0 + 1;
                let i2 = i0 + (width + 1);
                let i3 = i2 + 1;
                quads.push([i0, i1, i3, i2]);
            }
        }

        Self { vertices, quads }
    }

    pub fn edges(&self) -> Vec<[usize; 2]> {
        let mut edges = Vec::new();
        for quad in &self.quads {
            for i in 0..quad.len() {
                let next = (i + 1) % 4;
                edges.push([quad[i], quad[next]]);
            }
        }
        edges
    }
}
