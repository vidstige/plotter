use nalgebra_glm::Vec2;

use crate::{geometries, geometry::compute_gamma};

#[test]
fn test_plane_gamma() {
    let geom = geometries::plane::Plane;
    let p = Vec2::new(1.0, 2.0);

    let gamma = compute_gamma(&geom, &p);

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                assert!(gamma[i][j][k].abs() < 1e-5, "Plane Christoffel not zero at ({},{},{})", i, j, k);
            }
        }
    }
}

#[test]
fn test_sphere_gamma() {
    let geom = geometries::sphere::Sphere;
    let u = std::f32::consts::FRAC_PI_4; // pi/4 = 45 degrees
    let v = 0.0;
    let p = Vec2::new(u, v);

    let gamma = compute_gamma(&geom, &p);

    // Expected values
    let expected_gamma_0_1_1 = -0.5;
    let expected_gamma_1_0_1 = 1.0;
    let expected_gamma_1_1_0 = 1.0;

    assert!((gamma[0][1][1] - expected_gamma_0_1_1).abs() < 1e-5, "Gamma^u_vv wrong");
    assert!((gamma[1][0][1] - expected_gamma_1_0_1).abs() < 1e-5, "Gamma^v_uv wrong");
    assert!((gamma[1][1][0] - expected_gamma_1_1_0).abs() < 1e-5, "Gamma^v_vu wrong");
}
