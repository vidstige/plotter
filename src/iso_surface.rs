use nalgebra_glm::Vec3;


pub trait IsoSurface {
    fn iso_level(&self, position: &Vec3) -> f32;
}