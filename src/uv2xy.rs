use nalgebra_glm::{distance, Vec3, Vec4};

use crate::{
    camera::Camera,
    eq::NewtonRaphsonOptions,
    geometry::Geometry,
    paper::ViewBox,
    polyline::{Polyline2, Polyline4},
    raytracer::{backproject, Tracer},
    sdf::SDF,
};

fn in_front_of_camera(screen: &Vec3) -> bool {
    screen.z > 0.0
}

// handle occlusions
fn visible(
    world: &Vec3,
    screen: &Vec3,
    camera: &Camera,
    geometry: &impl SDF,
    tracer: &Tracer,
) -> bool {
    const HIT_EPSILON: f32 = 0.005;
    // back project and ray trace to find occlusions
    let ray = backproject(&screen.xy(), &camera.model, &camera.projection, camera.viewport);
    if let Some(intersection) = tracer.trace(&ray, geometry) {
        return distance(world, &intersection) < HIT_EPSILON;
    }
    false
}

// Takes uv-coordinates and returns projected screen-space coordinates.
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
) -> Vec<Polyline4> {
    let tracer = Tracer {
        near,
        far,
        steps: 200,
        newton_raphson: NewtonRaphsonOptions::default(),
    };
    // TODO: When a point is occluded, start a new linesegment
    let points = polyline.points.iter().map(|uv| {
        let world = geometry.evaluate(uv); // evaluate to 3D point
        let clip = camera.projection * camera.model * Vec4::new(world.x, world.y, world.z, 1.0);
        let ndc = clip.xyz() / clip.w;
        let screen_x = camera.viewport.x + camera.viewport.z * (ndc.x + 1.0) * 0.5;
        let screen_y = camera.viewport.y + camera.viewport.w * (ndc.y + 1.0) * 0.5;
        let screen_z = ndc.z * 0.5 + 0.5;
        let screen = Vec4::new(screen_x, screen_y, screen_z, clip.w);
        (world, screen)
    });

    let mut polylines = Vec::new();
    let mut current = Polyline4::new();
    for (world, screen) in points {
        let screen3 = screen.xyz();
        if in_front_of_camera(&screen3) && visible(&world, &screen3, &camera, geometry, &tracer) {
            current.add(screen);
        } else {
            polylines.push(current);
            current = Polyline4::new();
        }
    }
    polylines.push(current);
    polylines
}

pub fn keep_xy(polylines: Vec<Polyline4>) -> Vec<Polyline2> {
    polylines
        .into_iter()
        .map(|polyline| polyline.points.into_iter().map(|screen| screen.xy()).collect())
        .collect()
}
