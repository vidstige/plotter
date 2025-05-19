use std::time::Duration;

use crate::paper::Paper;

/// A tuple for one measurement:
/// (draw_length, move_length, draw_speed, move_speed, duration)
pub type Measurement = (f32, f32, f32, f32, Duration);

pub struct Estimator {
    c_draw: f32,
    c_move: f32,
}

impl Estimator {
    pub fn estimate(&self, paper: &Paper, draw_speed: f32, move_speed: f32) -> Duration {
        let (draw_length, move_length) = paper.length();
        self.estimate_time(draw_length, move_length, draw_speed, move_speed)
    }
    pub fn estimate_time(&self, draw_length: f32, move_length: f32, draw_speed: f32, move_speed: f32) -> Duration {
        let draw_time = (draw_length / draw_speed) * self.c_draw;
        let move_time = (move_length / move_speed) * self.c_move;
        Duration::from_secs_f32(draw_time + move_time)
    }
}


pub fn fit_to(measurements: &[Measurement]) -> Estimator {
    let (mut sum_d2, mut sum_dm, mut sum_m2, mut sum_dt, mut sum_mt) = (0.0, 0.0, 0.0, 0.0, 0.0);

    for &(draw_len, move_len, draw_spd, move_spd, duration) in measurements {
        let d = draw_len / draw_spd;
        let m = move_len / move_spd;
        let t = duration.as_secs_f32();

        sum_d2 += d * d;
        sum_dm += d * m;
        sum_m2 += m * m;
        sum_dt += d * t;
        sum_mt += m * t;
    }

    let det = sum_d2 * sum_m2 - sum_dm * sum_dm;
    let c_draw = (sum_m2 * sum_dt - sum_dm * sum_mt) / det;
    let c_move = (sum_d2 * sum_mt - sum_dm * sum_dt) / det;

    Estimator { c_draw, c_move }
}
