use nalgebra_glm::{project, Mat4x4, Vec3, Vec4};

pub struct Camera {
    // TODO: this can probably be private if the reproject function is updated
    pub projection: Mat4x4,
    pub model: Mat4x4,
    pub viewport: Vec4,
}
impl Camera {
    pub fn project(&self, world: Vec3) -> Vec3 {
        project(&world, &self.model, &self.projection, self.viewport)
    }
}
