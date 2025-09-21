pub mod geometry;
pub mod geometries {
    pub mod heightmap;
    pub mod blend;
    pub mod hole;
    pub mod sphere;
    pub mod plane;
    pub mod gaussian;
    pub mod torus;
    pub mod pulse;
    mod zero;
}
pub mod camera;
pub mod lerp;
pub mod field;
pub mod fields;
pub mod integrate;
pub mod sdf;
pub mod gridlines;
pub mod resolution;
pub mod eq;
pub mod buffer;
pub mod marching_squares;
pub mod mesh2;
pub mod netbm;
pub mod paper;
pub mod polyline;
pub mod raytracer;
pub mod sdf_transform;
pub mod simplex;
pub mod uv2xy;
pub mod time_estimator;

#[cfg(test)]
mod tests;
