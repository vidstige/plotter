use std::ops::{Add, Sub};

use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use nalgebra_glm::unproject;

use crate::{eq::{linesearch, newton_raphson}, iso_surface::IsoSurface};


pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f32) -> Vec3 {
        self.origin.add(self.direction.scale(t))
    }
}

pub fn backproject(screen: &Vec2, model: &Mat4, projection: &Mat4, viewport: Vec4) -> Ray {
    let world = unproject(&Vec3::new(screen.x, screen.y, 1.0), &model, &projection, viewport);
    // recover eye position
    let model_inverse = model.try_inverse().unwrap();
    let eye = model_inverse.column(3).xyz();

    Ray{ origin: eye, direction: world.sub(eye).normalize() }
}

pub fn trace<S: IsoSurface>(ray: &Ray, surface: &S, lo: f32, hi: f32) -> Option<Vec3> {
    // first linesearch to find rough estimate
    let f = |t| surface.iso_level(&ray.at(t));
    if let Some((lo, hi)) = linesearch(f, lo, hi, 200) {
        // fine tune with newton_raphson
        if let Some(t) = newton_raphson(f, 0.5 * (hi + lo)) {
        //if let Some(t) = newton_raphson(f, lo) {
            return Some(ray.at(t));
        }
    }
    None
}