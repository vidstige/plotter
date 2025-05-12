use nalgebra_glm::Vec2;

use crate::geometry::{acceleration, DifferentiableGeometry};


// step functions
pub fn euler(geometry: &impl DifferentiableGeometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    let a = acceleration(geometry, position, &velocity);
    (position + velocity * dt, velocity + a * dt)
}

pub fn verlet(geometry: &impl DifferentiableGeometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    let a = acceleration(geometry, position, velocity);
    let new_position = position + velocity * dt + a * (dt * dt * 0.5);
    let new_a = acceleration(geometry, &new_position, &velocity);
    // TODO: acceleration could be stored to save time next frame
    let new_velocity = velocity + (a + new_a) * (dt * 0.5);
    (new_position, new_velocity)
}

// Fixed-point iteration (from ChatGPT)
pub fn implicit_euler(geometry: &impl DifferentiableGeometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    // Initial guess using explicit Euler
    let x = *position;
    let v = *velocity;
    let mut x_next = x + dt * v;
    let mut v_next = v;
    for _ in 0..10 {
        let acc = acceleration(geometry, position, &velocity);

        let v_new = v - dt * acc;
        let x_new = x + dt * v_new;

        let dx = x_new - x_next;
        let dv = v_new - v_next;

        x_next = x_new;
        v_next = v_new;
        if dx.norm_squared() + dv.norm_squared() < 1e-9 {
            break;
        }
    }
    (x_next, v_next)
}
