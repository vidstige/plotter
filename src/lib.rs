pub mod geometry;
pub mod geometries {
    pub mod blend;
    pub mod gaussian;
    pub mod heightmap;
    pub mod hole;
    pub mod plane;
    pub mod pulse;
    pub mod sphere;
    pub mod torus;
    mod zero;
}
pub mod buffer;
pub mod camera;
pub mod eq;
pub mod field;
pub mod fields;
pub mod gridlines;
pub mod integrate;
pub mod lerp;
pub mod marching_squares;
pub mod mesh2;
pub mod netbm;
pub mod paper;
pub mod polyline;
pub mod raytracer;
pub mod resolution;
pub mod sdf;
pub mod sdf_transform;
pub mod simplex;
pub mod time_estimator;
pub mod uv2xy;

#[cfg(test)]
mod tests;
