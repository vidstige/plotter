use std::{ops::{Sub, Add}, io::{self, Write}, fs::File, collections::VecDeque, f64::consts::TAU};

use plotter::eq::{linesearch, newton_raphson};
use plotter::geometries::{sphere::Sphere, hole::Hole};
use plotter::geometry::Geometry;
use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, perspective, unproject, Mat4};

use plotter::polyline::Polyline2;

use rand::{distributions::Distribution, rngs::ThreadRng};
use plotter::resolution::Resolution;
use rand_distr::StandardNormal;
use tiny_skia::{Pixmap, PathBuilder, Paint, Stroke, Transform, Color};

fn cross2(vector: Vec2) -> Vec2 {
    Vec2::new(-vector.y, vector.x)
}

struct Spiral {
    center: Vec2,
}
impl Spiral {
    fn new(center: Vec2) -> Spiral {
        Spiral { center }
    }
    fn at(&self, p: &Vec2) -> Vec2 {
        cross2(p.sub(&self.center))
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f32) -> Vec3 {
        self.origin.add(self.direction.scale(t))
    }
}

fn backproject(screen: &Vec2, model: &Mat4, projection: &Mat4, viewport: Vec4) -> Ray {
    let world = unproject(&Vec3::new(screen.x, screen.y, 1.0), &model, &projection, viewport);
    // recover eye position
    let model_inverse = model.try_inverse().unwrap();
    let eye = model_inverse.column(3).xyz();

    Ray{ origin: eye, direction: world.sub(eye).normalize() }
}

trait IsoSurface {
    fn iso_level(&self, position: &Vec3) -> f32;
}

impl IsoSurface for Hole {
    fn iso_level(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

impl IsoSurface for Sphere {
    fn iso_level(&self, position: &Vec3) -> f32 {
        position.norm() - 1.0
    }
}

fn trace<S: IsoSurface>(ray: &Ray, surface: &S, lo: f32, hi: f32) -> Option<Vec3> {
    // first linesearch to find rough estimate
    let f = |t| surface.iso_level(&ray.at(t));
    if let Some((lo, hi)) = linesearch(f, lo, hi, 10) {
        // fine tune with newton_raphson
        if let Some(t) = newton_raphson(f, 0.5 * (hi + lo)) {
            return Some(ray.at(t));
        }
    }
    None
}

fn contains(resolution: &Resolution, point: &Vec2) -> bool {
    point.x >= 0.0 && point.x < resolution.width as f32 && point.y >= 0.0 && point.y < resolution.height as f32
}

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut ThreadRng) -> Vec2 {
    Vec2::new(
        distribution.sample(rng),
        distribution.sample(rng),
    )
}

// return Christoffel symbols with index k, i, j
fn compute_gamma(geometry: &impl Geometry, p: &Vec2) -> [[[f32; 2]; 2]; 2] {
    let metric = geometry.metric(p);
    let maybe_inverse_metric = metric.try_inverse();
    if maybe_inverse_metric.is_none() {
        println!("could not invert {:?}", metric);
    }
    let inverse_metric = maybe_inverse_metric.unwrap();
    // compute all second order partial derivatives
    let d2: [[Vec3; 2]; 2] = [ 
        [geometry.du().du().evaluate(p), geometry.du().dv().evaluate(p)],
        [geometry.dv().du().evaluate(p), geometry.dv().dv().evaluate(p)],
    ];
    // compute first order partial derivatives
    let d: [Vec3; 2] = [geometry.du().evaluate(p), geometry.dv().evaluate(p)];

    // compute tensor product gamma^k_ij = (d²R/du^i du^j) * (dR/du^l) * (g^-1)^lk
    // the index l is thus summed over
    let mut tmp = [[[0.0; 2]; 2]; 2];
    for k in 0..2 {
        for i in 0..2 {
            for j in 0..2 {
                for l in 0..2 {
                    tmp[k][i][j] += d2[i][j].dot(&d[l]) * inverse_metric[(l, k)];
                }
            }
        }
    }
    tmp
}

fn acceleration(geometry: &impl Geometry, position: &Vec2, velocity: &Vec2) -> Vec2 {
    let gamma = compute_gamma(geometry, position);
    let mut a = Vec2::zeros();
    let u = velocity.as_slice();
    // tensor sum
    for k in 0..2 {
        for i in 0..2 {
            for j in 0..2 {
                a.as_mut_slice()[k] += -gamma[k][i][j] * u[i] * u[j];
            }
        }
    }
    a
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

fn main() -> io::Result<()> {
    let resolution = Resolution::new(506, 253);

    let mut rng = rand::thread_rng();
    let distribution = StandardNormal {};
    let field = Spiral::new(Vec2::zeros());

    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);

    let geometry = Hole::new();
    //let geometry = Sphere::new();

    let mut output = File::create(std::path::Path::new("output.raw"))?;
    let positions: Vec<_> = (0..256).map(|_| sample_vec2(&distribution, &mut rng)).collect();
    //let semicircle = Uniform::new(0.0, 0.5*TAU).unwrap();
    //let positions: Vec<_> = (0..256).map(|_| sample_vec2(&semicircle, &mut rng)).collect();
    let mut particles: Vec<_> = positions.iter().map(|p| Particle {
        position: Vec2::new(p.x, p.y),
        //velocity: 0.2 * field.at(p),
        velocity: Vec2::new(0.0, 0.5),
    }).collect();
    let mut traces: Vec<VecDeque<Vec3>> = particles.iter().map(|_| VecDeque::new()).collect();
    let fps = 25.0;
    let dt = 1.0 / fps;
    for frame in 0..256 {
        let mut polylines = Vec::new();
        for (index, particle) in particles.iter_mut().enumerate() {
            // move according to field
            /*let nabla = field.at(p);
            let norm = nabla.norm();
            let step = 0.01;
            p.add_assign(nabla.scale(step / norm));*/

            // integrate geodesic equation (d²u/dt²) + gamma^k_ij * (du^i/dt) * (du^j/dt) = 0
            
            // verlet
            /*let a = acceleration(&geometry, &particle.position, &particle.velocity);
            let new_position = particle.position + particle.velocity * dt + a * (dt * dt * 0.5);
            let new_a = acceleration(&geometry, &new_position, &particle.velocity);
            let new_velocity = particle.velocity + (a + new_a) * (dt * 0.5);
            particle.position = new_position;
            particle.velocity = new_velocity;
            // TODO: acceleration could be stored to save time
            */
            
            // euler
            let a = acceleration(&geometry, &particle.position, &particle.velocity);
            particle.position += particle.velocity * dt;
            particle.velocity += a * dt;

            // evaluate surface at x, y
            //let z = hole.z(&p);
            //let world = Vec3::new(p.x, p.y, z);
            
            // project world cordinate into screen cordinate
            let world = geometry.evaluate(&particle.position);
            let screen = project(&world, &model, &projection, viewport);

            traces[index].push_back(screen);
            if traces[index].len() > 10 {
                traces[index].pop_front();
            }
        }

        // draw traces
        if frame > 10 {
            for particle_trace in &traces {
                let mut polyline = Polyline2::new();
                for screen in particle_trace {
                    if contains(&resolution, &screen.xy()) {
                        // back project and ray trace to find occlusions
                        let ray = backproject(&screen.xy(), &model, &projection, viewport);
                        if let Some(intersection) = trace(&ray, &geometry, near, far) {
                            let traced_screen = project(&intersection, &model, &projection, viewport);
                            // handle occlusions
                            if screen.z - traced_screen.z < 0.0001 {
                                polyline.add(screen.xy());
                            }
                        }
                    }
                }
                polylines.push(polyline);
            }
        }

        // render to pixmap
        let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
        
        let color = Color::from_rgba8(255, 180, 220, 0xff);
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = 1.0;

        for polyline in polylines {
            let mut pb = PathBuilder::new();
            for (index, point) in polyline.points.iter().enumerate() {
                if index == 0 {
                    pb.move_to(point.x, point.y);
                } else {
                    pb.line_to(point.x, point.y);
                }
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        //pixmap.save_png("image.png")?;
        output.write_all(pixmap.data())?;
    }

    Ok(())
}