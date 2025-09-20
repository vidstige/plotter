pub fn linesearch<F: Fn(f32) -> f32>(f: F, lo: f32, hi: f32, steps: usize) -> Option<(f32, f32)> {
    let step_length = (hi - lo) / steps as f32;
    let mut x0 = lo;
    let mut f0 = f(x0);
    for step in 0..steps {
        let x = step as f32 * step_length;
        let fx = f(x);
        if (f0 < 0.0) != (fx < 0.0) {
            // root range found!
            return Some((x0, x));
        }
        x0 = x;
        f0 = fx;
    }
    None
}

pub struct NewtonRaphsonOptions {
    epsilon: f32, // for numerical diffrentiation
    atol: f32, // for considering roots
    dtol: f32, // for bailing out on vanishing differentials
    max_steps: usize,
}

impl Default for NewtonRaphsonOptions {
    fn default() -> Self {
        Self { epsilon: 0.001, atol: 0.0001, dtol: 0.001, max_steps: 20 }
    }
}

pub fn newton_raphson<F: Fn(f32) -> f32>(f: F, x0: f32, options: NewtonRaphsonOptions) -> Option<f32> {
    let epsilon = options.epsilon;
    let mut x = x0; 
    
    for _ in 0..options.max_steps {
        // compute df/dt using forward diffrentiation
        let dfdt = (f(x + epsilon) - f(x - epsilon)) / (2.0 * epsilon);
        if dfdt.abs() < options.dtol {
            break;
        }
        x = x - f(x) / dfdt;
        // exit early if root found
        if f(x).abs() < options.atol {
            break;
        }
    }
    // if we're close enough a root was found
    (f(x).abs() < options.atol).then_some(x)
}