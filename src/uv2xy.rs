use nalgebra_glm::{distance, Vec3};

use crate::{
    camera::Camera, eq::NewtonRaphsonOptions, geometry::Geometry, paper::ViewBox, polyline::Polyline2, raytracer::{backproject, Tracer}, sdf::SDF
};

fn in_front_of_camera(screen: &Vec3) -> bool {
    screen.z > 0.0
}

// handle occlusions
fn visible(world: &Vec3, screen: &Vec3, camera: &Camera, geometry: &impl SDF, tracer: &Tracer) -> bool {
    const HIT_EPSILON: f32 = 0.005;
    // back project and ray trace to find occlusions
    let ray = backproject(&screen.xy(), &camera.model, &camera.projection, camera.viewport);
    if let Some(intersection) = tracer.trace(&ray, geometry) {
        return distance(world, &intersection) < HIT_EPSILON;
    }
    false
}

// Takes uv-coordinates and returns xy-cordinates
// 1. Evaluates geometry
// 2. Project using camera
// 3. Drop points behind the camera
// 4. Handle occlusion
pub fn reproject<G: Geometry + SDF>(
    polyline: &Polyline2,
    geometry: &G,
    camera: &Camera,
    _area: ViewBox,
    near: f32,
    far: f32,
) -> Vec<Polyline2> {
    let tracer = Tracer { near, far, steps: 200, newton_raphson: NewtonRaphsonOptions::default() };
    // TODO: When a point is occluded, start a new linesegment
    let points = polyline
        .points
        .iter()
        .map(|uv| {
            let world = geometry.evaluate(uv); // evaluate to 3D point
            let screen = camera.project(world);
            (world, screen)
        });

    let mut polylines = Vec::new();
    let mut current = Polyline2::new();
    for (world, screen) in points {
        if in_front_of_camera(&screen) && visible(&world, &screen, &camera, geometry, &tracer) {
            current.add(screen.xy());
        } else {
            polylines.push(current);
            current = Polyline2::new();
        }
    }
    polylines.push(current);
    polylines
}
