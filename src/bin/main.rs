use std::{io, ops::AddAssign};

use nalgebra_glm::{look_at, perspective, project, Mat4x4, Vec2, Vec3, Vec4};
use plotter::{fields::Spiral, geometries::hole::Hole, geometry::Geometry, gridlines::{bent_grid_line, grid_line}, integrate::verlet, iso_surface::IsoSurface, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, raytracer::{backproject, trace}};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}

struct Camera {
    projection: Mat4x4,
    model: Mat4x4,
    viewport: Vec4,
}
impl Camera {
    fn project(&self, world: Vec3) -> Vec3 {
        project(&world, &self.model, &self.projection, self.viewport)
    }
}

// handle occlusions
fn visible(
    screen: &Vec3,
    camera: &Camera,
    geometry: &impl IsoSurface,
    near: f32,
    far: f32,
) -> bool {
    // back project and ray trace to find occlusions
    let ray = backproject(&screen.xy(), &camera.model, &camera.projection, camera.viewport);
    if let Some(intersection) = trace(&ray, geometry, near, far) {
        let traced_screen = project(&intersection, &camera.model, &camera.projection, camera.viewport);
        // handle occlusions
        if screen.z - traced_screen.z < 0.0001 {
            return true
        }
    }
    false
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

// Takes uv-coordinates and returns xy-cordinates
// 1. Evaluates geometry
// 2. Project using camera
// 3. Clip to viewport 
// 4. Handle occlusion
fn reproject<G: Geometry + IsoSurface>(polyline: &Polyline2, geometry: &G, camera: &Camera, area: ViewBox, near: f32, far: f32) -> Vec<Polyline2> {
    // TODO: When a point is occluded, start a new linesegment
    let points: Vec<_> = polyline.points.iter()
        .map(|uv| geometry.evaluate(uv))  // evaluate to 3D point
        .map(|world| camera.project(world))
        .filter(|&screen| contains(&area, &screen.xy()))
        .filter(|&screen| visible(&screen, &camera, geometry, near, far)) // handle occlusions
        .map(|screen| screen.xy())
        .collect();

    [Polyline2 { points }].into()
}

fn main() -> io::Result<()> {
    // set up paper
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 20);

    // set up pseudo random generator
    let mut rng = rand::thread_rng();
    let distribution = Normal::new(0.0, 1.0).unwrap();

    // set up uv-field
    let field = Spiral::new(Vec2::zeros());

    // set up 3D geometry
    let geometry = Hole::new();

    // set up 3D camera
    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    // top-view
    //let eye = Vec3::new(0.0, 0.0, -2.5);
    //let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0));

    let near = 0.1;
    let far = 10.0;
    let projection = perspective(viewbox_aspect(paper.view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };

    let size = 1.5;  // parameter for square things like gridlines
    let n = 16;
    for i in 0..n {
        // gridlines
        let amplitude = 0.05;
        let p = size * 2.0 * (i as f32 / n as f32 - 0.5);
        // fixed v gridlines
        let start = Vec2::new(-size, p);
        let end = Vec2::new(size, p);
        //let uv_polyline = bent_grid_line(&geometry, start, end, amplitude);
        let uv_polyline = grid_line(start, end, 128);
        for polyline in reproject(&uv_polyline, &geometry, &camera, area, near, far) {
            paper.add(polyline);
        }
        // fixed u gridlines
        let start = Vec2::new(p, -size);
        let end = Vec2::new(p, size);
        //let uv_polyline = bent_grid_line(&geometry, start, end, amplitude);
        let uv_polyline = grid_line(start, end, 128);
        for polyline in reproject(&uv_polyline, &geometry, &camera, area, near, far) {
            paper.add(polyline);
        }


        // random positions
        /*let position = Vec2::new(
            distribution.sample(&mut rng) as f32,
            distribution.sample(&mut rng) as f32,
        );

        let mut particle = Particle {
            position,
            velocity: field.at(&position),
        };*/

        // paralell lines
        /*let mut particle = Particle {
            position: Vec2::new(-size, size * 2.0 * (i as f32 / n as f32 - 0.5)),
            velocity: Vec2::new(1.0, 0.0),
        };*/

        // inward
        /*let theta = TAU * (i as f32 / n as f32);
        let position = 2.0 * Vec2::new(theta.cos(), theta.sin());
        let velocity = -cross2(position) - 2.0 * position;
        let mut particle = Particle {position, velocity};*/

        // integrate
        /*let mut uv_polyline = Polyline2::new();
        for _ in 0..20 {
            uv_polyline.add(particle.position);
            let dt = 0.1;
            (particle.position, particle.velocity) = verlet(&geometry, &particle.position, &particle.velocity, dt);
        }*/

        // project & etc
        //paper.add(reproject(&uv_polyline, &geometry, &camera, area, near, far));
    }
    paper.optimize();
    paper.save("output.svg")?;

    Ok(())
}