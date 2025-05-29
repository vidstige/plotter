use std::collections::HashSet;

use crate::{geometries, geometry::{DifferentiableGeometry, Geometry}, mesh2::Mesh2};

/// Builds a neighbor lookup table from the quads in `Mesh2`.
/// Each vertex will be assigned the set of directly adjacent vertices (shared in a quad).
pub fn build_neighbor_lookup(mesh: &Mesh2) -> Vec<Vec<usize>> {
    let mut neighbor_sets: Vec<HashSet<usize>> = vec![HashSet::new(); mesh.vertices.len()];

    for quad in &mesh.quads {
        let [i0, i1, i3, i2] = *quad;

        // Horizontal edges
        neighbor_sets[i0].insert(i1);
        neighbor_sets[i1].insert(i0);

        neighbor_sets[i2].insert(i3);
        neighbor_sets[i3].insert(i2);

        // Vertical edges
        neighbor_sets[i0].insert(i2);
        neighbor_sets[i2].insert(i0);

        neighbor_sets[i1].insert(i3);
        neighbor_sets[i3].insert(i1);
    }

    // Convert sets to Vecs
    neighbor_sets
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect()
}

use nalgebra_glm::{cross, dot, length, normalize, vec3, Vec2, Vec3};

/// Computes initial orientation field for each vertex as a tangent unit vector in 3D.
/// Returns a vector of unit Vec3s (one per vertex), representing 4-RoSy directions.
pub fn initialize_orientation_field(
    surface: &impl DifferentiableGeometry,
    uv_vertices: &[Vec2],
) -> Vec<Vec3> {
    let du_surface = surface.du();
    let dv_surface = surface.dv();

    let mut orientations = Vec::with_capacity(uv_vertices.len());

    for uv in uv_vertices {
        // Compute normal
        let du = du_surface.evaluate(uv);
        let dv = dv_surface.evaluate(uv);
        let normal = normalize(&cross(&du, &dv));

        // Use fixed reference direction
        let mut ref_dir = vec3(1.0, 0.0, 0.0);
        if dot(&ref_dir, &normal).abs() > 0.99 {
            ref_dir = vec3(0.0, 1.0, 0.0);
        }

        // Project ref_dir onto tangent plane
        let projected = ref_dir - normal * dot(&ref_dir, &normal);
        let tangent = if projected.norm() < 1e-8 {
            // fallback: pick orthogonal direction in tangent plane
            normalize(&cross(&normal, &vec3(0.0, 0.0, 1.0)))
        } else {
            normalize(&projected)
        };

        orientations.push(tangent);
    }

    orientations
}

/// Optimizes the orientation field to enforce 4-RoSy smoothness across the surface.
pub fn optimize_orientation_field(
    geometry: &impl DifferentiableGeometry,
    mesh: &Mesh2,
    orientations: &mut Vec<Vec3>,
    iterations: usize,
) {
    let neighbors = build_neighbor_lookup(mesh);
    let uv_vertices = &mesh.vertices;
    assert_eq!(uv_vertices.len(), orientations.len(), "UV vertices and orientations must match in length");

    for _ in 0..iterations {
        let old_orientations = orientations.clone();

        for (i, uv) in uv_vertices.iter().enumerate() {
            let o_i = old_orientations[i];
            let n_i = geometry.normal(uv);

            let mut avg = Vec3::new(0.0, 0.0, 0.0);

            for &j in &neighbors[i] {
                let o_j: Vec3 = old_orientations[j];

                // Try all 4 90° rotations of o_j in i's tangent plane
                let candidates = [
                    o_j,
                    cross(&n_i, &o_j),     // 90°
                    -o_j,                  // 180°
                    -cross(&n_i, &o_j),    // 270°
                ];

                // Pick rotation with max alignment to o_i
                let best = *candidates
                    .iter()
                    .max_by(|a, b| {
                        dot(&o_i, a)
                            .partial_cmp(&dot(&o_i, b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap();

                avg += best;
            }

            if length(&avg) > 1e-6 {
                // Project onto tangent plane
                let projected = avg - n_i * dot(&n_i, &avg);

                if length(&projected) > 1e-6 {
                    orientations[i] = normalize(&projected);
                }
            }
        }
    }
}

/// Initializes the position field aligned to the orientation field and snapped to the nearest grid point.
///
/// Returns a Vec<Vec3> of the same length as `uv_vertices`, representing position field p_i.
pub fn initialize_position_field(
    surface: &impl DifferentiableGeometry,
    uv_vertices: &[Vec2],
    orientations: &[Vec3],
    rho: f32,
) -> Vec<Vec3> {
    uv_vertices
        .iter()
        .zip(orientations.iter())
        .map(|(uv, o_i)| {
            let pos_i = surface.evaluate(uv);
            let n_i = surface.normal(uv);

            let u_dir = *o_i;
            let v_dir = normalize(&cross(&n_i, &u_dir));

            // Project pos_i onto local tangent frame to get (du, dv)
            let du = dot(&pos_i, &u_dir);
            let dv = dot(&pos_i, &v_dir);

            // Snap (du, dv) to nearest integer multiple of ρ
            let snapped_u = (du / rho).round() * rho;
            let snapped_v = (dv / rho).round() * rho;

            // Convert back to 3D position field p_i
            snapped_u * u_dir + snapped_v * v_dir
        })
        .collect()
}
