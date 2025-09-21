use std::collections::HashMap;

use nalgebra_glm::Vec2;

use crate::{field::Field, polyline::Polyline2};

const POINTS: [[i32; 2]; 4] = [[0, 0], [1, 0], [0, 1], [1, 1]];

type Edge = (usize, usize);
const EDGE_CASES: [[Option<(Edge, Edge)>; 2]; 16] = [
    [None, None],
    [Some(((0, 2), (0, 1))), None],
    [Some(((0, 1), (1, 3))), None],
    [Some(((0, 2), (1, 3))), None],
    [Some(((0, 2), (2, 3))), None],
    [Some(((0, 1), (2, 3))), None],
    [Some(((0, 2), (2, 3))), Some(((0, 1), (1, 3)))],
    [Some(((2, 3), (1, 3))), None],
    [Some(((1, 3), (2, 3))), None],
    [Some(((0, 2), (0, 1))), Some(((2, 3), (1, 3)))],
    [Some(((2, 3), (0, 1))), None],
    [Some(((2, 3), (0, 2))), None],
    [Some(((1, 3), (0, 2))), None],
    [Some(((1, 3), (0, 1))), None],
    [Some(((0, 1), (0, 2))), None],
    [None, None],
];

fn interpolate_edge(
    points: &[[i32; 2]],
    (x, y): (f32, f32),
    values: &[f32; 4],
    (i0, i1): Edge,
    level: f32,
) -> Vec2 {
    let (v0, v1) = (values[i0], values[i1]);

    // compute parameter
    let t = (level - v0) / (v1 - v0);
    debug_assert!(t >= 0.0 && t < 1.0);
    // lerp x & y and add offsets
    Vec2::new(
        x + (1.0 - t) * (points[i0][0] as f32) + t * (points[i1][0] as f32),
        y + (1.0 - t) * (points[i0][1] as f32) + t * (points[i1][1] as f32),
    )
}

fn next_non_empty(residual: &Vec<Vec<usize>>) -> Option<usize> {
    residual.iter().enumerate().filter_map(|(i, rs)| (!rs.is_empty()).then_some(i)).next()
}

fn continuation(residual: &Vec<Vec<usize>>, vertex: Option<&usize>) -> Option<usize> {
    match vertex {
        Some(&number) => (!residual[number].is_empty()).then_some(number),
        None => None,
    }
}

// Find chains in a bag of edges. For example [(0,1), (1,2)] will result in
// [[0, 1, 2]]
fn find_chains(edges: &[Edge]) -> Vec<Vec<usize>> {
    // 1. Find top edge value and use a vector instead of hash table
    let Some(&top) = edges.iter().map(|(e0, e1)| e0.max(e1)).max() else { return Vec::new(); };

    // 2. Create lookup table for edges left to process (todo)
    let mut residual: Vec<Vec<usize>> = vec![Vec::new(); top + 1];
    for (a, b) in edges {
        residual[*a].push(*b);
        residual[*b].push(*a);
    }

    let mut chains: Vec<Vec<usize>> = Vec::new();
    // Keep taking vertices until completely empty
    while let Some(vertex) = next_non_empty(&residual) {
        let mut chain = vec![vertex];

        // Search forwards (search for links from chain.last())
        while let Some(a) = continuation(&residual, chain.last()) {
            if let Some(&b) = residual[a].iter().filter(|v| !chain.contains(v)).next() {
                // remove edge from "todo"
                residual[a].retain(|&v| v != b);
                residual[b].retain(|&v| v != a);
                // add at end
                chain.push(b);
            } else {
                break;
            }
        }

        // Search backwards
        while let Some(a) = continuation(&residual, chain.first()) {
            if let Some(&b) = residual[a].iter().filter(|v| !chain.contains(v)).next() {
                // remove edge from "todo"
                residual[a].retain(|&v| v != b);
                residual[b].retain(|&v| v != a);
                // add at start
                chain.insert(0, b);
            } else {
                break;
            }
        }
        chains.push(chain);
    }
    chains
}

type VertexId = (i32, i32, bool);
fn vertex_id(x: usize, y: usize, edge: Edge) -> VertexId {
    let (a, b) = edge;
    let dy = POINTS[b][1] - POINTS[a][1];
    let is_horizontal = dy == 0;
    (x as i32 + POINTS[a][0], y as i32 + POINTS[b][1], is_horizontal)
}

pub fn find_contours(grid: &Field<f32>, level: f32) -> Vec<Polyline2> {
    // keeps track of vertex ids -> vertex number
    let mut lookup = HashMap::new();

    // holds vertex cordinates
    let mut vertices = Vec::new();
    let mut lines = Vec::new();
    for y in 0..grid.height() - 1 {
        for x in 0..grid.width() - 1 {
            // find grid values at the corner of the square
            let values = [
                grid[(x, y)],
                grid[(x + 1, y)],
                grid[(x, y + 1)],
                grid[(x + 1, y + 1)],
            ];

            // find table index
            let table_index: usize =
                values.iter().enumerate().map(|(i, v)| (1 << i) * ((*v < level) as usize)).sum();
            // loop over items that are not None
            for (e0, e1) in EDGE_CASES[table_index].iter().flatten() {
                let id0 = vertex_id(x, y, *e0);
                let id1 = vertex_id(x, y, *e1);

                let position = (x as f32, y as f32);
                let vertex0 = *lookup.entry(id0).or_insert_with(|| {
                    // interpolate
                    let vertex = interpolate_edge(&POINTS, position, &values, *e0, level);
                    // add and return index
                    vertices.push(vertex);
                    vertices.len() - 1
                });
                let vertex1 = *lookup.entry(id1).or_insert_with(|| {
                    // interpolate
                    let vertex = interpolate_edge(&POINTS, position, &values, *e1, level);
                    // add and return index
                    vertices.push(vertex);
                    vertices.len() - 1
                });

                // add edge
                lines.push((vertex0, vertex1));
            }
        }
    }
    // just dump all line segments
    //lines.iter().map(|(i0, i1)| vec![vertices[*i0], vertices[*i1]]).collect()
    // find chains and convert vertex ids to cordinates
    find_chains(&lines).iter().map(|chain| chain.iter().map(|&v| vertices[v]).collect()).collect()
}
