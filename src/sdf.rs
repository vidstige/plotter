use nalgebra_glm::Vec3;


pub trait SDF {
    fn sdf(&self, position: &Vec3) -> f32;
}