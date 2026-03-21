use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceVertex {
    pub vertex: usize,
    pub normal: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3 {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Vec<FaceVertex>>,
}

impl Mesh3 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Mesh3 {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
        }
    }
}
