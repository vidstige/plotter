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

/// Optimizes the position field by aligning neighbor offsets to ρ-spaced grid vectors
pub fn optimize_position_field(
    surface: &impl DifferentiableGeometry,
    mesh: &Mesh2,
    orientations: &[Vec3],
    position_field: &mut [Vec3],
    rho: f32,
    iterations: usize,
) {
    let neighbors = build_neighbor_lookup(mesh);
    let uv_vertices = &mesh.vertices;
    for _ in 0..iterations {
        let old_positions = position_field.to_vec();

        for (i, uv) in uv_vertices.iter().enumerate() {
            let p_i = old_positions[i];
            let o_i = orientations[i];
            let n_i = surface.normal(uv);

            let u_dir = o_i;
            let v_dir = normalize(&cross(&n_i, &u_dir));

            let mut avg = Vec3::new(0.0, 0.0, 0.0);
            let mut count = 0;

            for &j in &neighbors[i] {
                let p_j = old_positions[j];

                // Try small integer jumps (a, b) ∈ [-1, 0, 1]^2
                let mut best_t = Vec3::new(0.0, 0.0, 0.0);
                let mut best_dist2 = f32::INFINITY;

                for a in -1..=1 {
                    for b in -1..=1 {
                        let t_ij = (a as f32) * rho * u_dir + (b as f32) * rho * v_dir;
                        let candidate = p_j + t_ij;
                        let dist2 = (p_i - candidate).magnitude_squared();
                        if dist2 < best_dist2 {
                            best_dist2 = dist2;
                            best_t = t_ij;
                        }
                    }
                }

                avg += p_j + best_t;
                count += 1;
            }

            if count > 0 {
                let new_p_i = avg / count as f32;

                // Project back onto tangent plane (optional)
                let offset = new_p_i - surface.evaluate(uv);
                let projected = offset
                    - n_i * dot(&n_i, &offset);
                position_field[i] = surface.evaluate(uv) + projected;
            }
        }
    }
}

pub struct QuadMesh {
    pub vertices: Vec<Vec3>,
    pub quads: Vec<[usize; 4]>,
}

/// Constructs a quad mesh from a position field and orientation field using local frame matching.
pub fn extract_quad_mesh<D: DifferentiableGeometry>(
    surface: &D,
    mesh: &Mesh2,
    position_field: &[Vec3],
    orientations: &[Vec3],
    rho: f32,
    threshold: f32,
) -> QuadMesh {
    let neighbors = build_neighbor_lookup(mesh);
    let uv_vertices = &mesh.vertices;
    assert_eq!(uv_vertices.len(), position_field.len(), "UV vertices and position field must match in length");

    let n = uv_vertices.len();
    let mut edges_u = vec![None; n];
    let mut edges_v = vec![None; n];

    // Step 1: Identify +U and +V neighbors
    for i in 0..n {
        let p_i = position_field[i];
        let o_i = orientations[i];
        let n_i = surface.normal(&uv_vertices[i]);

        let u_dir = o_i;
        let v_dir = normalize(&cross(&n_i, &u_dir));

        for &j in &neighbors[i] {
            let delta = position_field[j] - p_i;
            let du = dot(&delta, &u_dir);
            let dv = dot(&delta, &v_dir);

            if dv.abs() < threshold && (du - rho).abs() < threshold {
                edges_u[i] = Some(j); // +U neighbor
            } else if du.abs() < threshold && (dv - rho).abs() < threshold {
                edges_v[i] = Some(j); // +V neighbor
            }
        }
    }

    // Step 2: Form quads from U/V neighbors
    let mut quads = Vec::new();
    for i in 0..n {
        let j = match edges_u[i] { Some(idx) => idx, None => continue };
        let k = match edges_v[i] { Some(idx) => idx, None => continue };
        let m = match edges_u[k] {
            Some(idx) if Some(idx) == edges_v[j] => idx,
            _ => continue,
        };

        // Optional: enforce ordering to avoid duplicate quads
        if i < j && i < k && i < m {
            quads.push([i, j, m, k]);
        }
    }

    QuadMesh {
        vertices: position_field.to_vec(), // or surface.evaluate(&uv[i]) if you want to reproject
        quads,
    }
}
