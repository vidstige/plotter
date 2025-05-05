use nalgebra_glm::{project, Vec2, Vec3};

use crate::{camera::Camera, geometry::Geometry, paper::ViewBox, polyline::Polyline2, raytracer::{backproject, trace}, sdf::SDF};


fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}

// handle occlusions
fn visible(
    screen: &Vec3,
    camera: &Camera,
    geometry: &impl SDF,
    near: f32,
    far: f32,
) -> bool {
    // back project and ray trace to find occlusions
    let ray = backproject(&screen.xy(), &camera.model, &camera.projection, camera.viewport);
    if let Some(intersection) = trace(&ray, geometry, near, far) {
        let traced_screen = project(&intersection, &camera.model, &camera.projection, camera.viewport);
        // handle occlusions
        if screen.z - traced_screen.z < 0.00001 {
            return true
        }
    }
    false
}

// Takes uv-coordinates and returns xy-cordinates
// 1. Evaluates geometry
// 2. Project using camera
// 3. Clip to viewport 
// 4. Handle occlusion
pub fn reproject<G: Geometry + SDF>(polyline: &Polyline2, geometry: &G, camera: &Camera, area: ViewBox, near: f32, far: f32) -> Vec<Polyline2> {
    // TODO: When a point is occluded, start a new linesegment
    let points = polyline.points.iter()
        .map(|uv| geometry.evaluate(uv))  // evaluate to 3D point
        .map(|world| camera.project(world));

    let mut polylines = Vec::new();
    let mut current = Polyline2::new();
    for screen in points {
        //current.add(screen.xy());
        if contains(&area, &screen.xy()) && visible(&screen, &camera, geometry, near, far) {
            current.add(screen.xy());
        } else {
            polylines.push(current);
            current = Polyline2::new();
        }
    }
    polylines.push(current);
    polylines
}
