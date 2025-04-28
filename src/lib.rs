pub mod geometry;
pub mod geometries {
    pub mod hole;
    pub mod sphere;
    pub mod plane;
    mod zero;
}
pub mod fields;
pub mod integrate;
pub mod iso_surface;
pub mod resolution;
pub mod eq;
pub mod buffer;
pub mod netbm;
pub mod paper;
pub mod polyline;
pub mod raytracer;

#[cfg(test)]
mod tests;
