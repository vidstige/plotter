use std::ops::{Add, Mul};

/// Linearly interpolates between `a` and `b` using `t`, where t ∈ [0, 1].
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Add<Output = T> + Mul<f32, Output = T> + Copy,
{
    a * (1.0 - t) + b * t
}