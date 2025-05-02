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

pub fn newton_raphson<F: Fn(f32) -> f32>(f: F, x0: f32) -> Option<f32> {
    let epsilon = 0.01; // for numerical diffrentiation
    let tol = 0.001; // for considering roots
    let mut x = x0; 
    
    for _ in 0..20 {
        // compute df/dt using forward diffrentiation
        let dfdt = (f(x + epsilon) - f(x - epsilon)) / (2.0 * epsilon);
        if dfdt.abs() < 0.001 {
            break;
        }
        x = x - f(x) / dfdt;
        // exit early if root found
        if f(x).abs() < tol {
            break;
        }
    }
    // if we're close enough a root was found
    (f(x).abs() < tol).then_some(x)
}