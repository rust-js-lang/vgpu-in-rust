//! Procedural, subtly filleted prism geometry from scene/prism-mesh.ts.
//!
//! The ideal triangular solid still belongs to types. This mesh rounds inward,
//! so the bevel catches the environment without moving its optical planes.
use std::collections::HashSet;
use std::f64::consts::PI;

use crate::types::{Triangle, Vec2, Vec3};
use crate::{clamp, number_max, number_min};

pub const PRISM_VERTEX_STRIDE: usize = 24;
pub const PRISM_BEVEL_RADIUS: f64 = 0.008;
pub const PRISM_CORNER_SEGMENTS: usize = 4;
pub const PRISM_BEVEL_SEGMENTS: usize = 4;
pub const PRISM_EDGE_SEGMENTS: usize = 16;

/// Interleaved position and normal, six floats per vertex.
pub struct PrismMeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Copy)]
struct ContourPoint {
    position: Vec2,
    normal: Vec2,
}

// Keep construction in f64, as the original number[] does; round only at the
// final Float32Array boundary. Rounding on every push changes cap centroids.
#[derive(Default)]
struct MeshBuilder {
    vertices: Vec<f64>,
    indices: Vec<u16>,
}

impl MeshBuilder {
    fn push(&mut self, position: Vec3, normal: Vec3) -> u16 {
        let index = (self.vertices.len() / 6) as u16;
        self.vertices.extend(position);
        self.vertices.extend(normal);
        index
    }

    fn add_cap(&mut self, source_ring: &[u16], normal: Vec3, reverse: bool) {
        let cap: Vec<_> = source_ring
            .iter()
            .map(|source| {
                let base = usize::from(*source) * 6;
                self.push(
                    [
                        self.vertices[base],
                        self.vertices[base + 1],
                        self.vertices[base + 2],
                    ],
                    normal,
                )
            })
            .collect();
        let center = [0, 1, 2].map(|axis| {
            cap.iter()
                .map(|index| self.vertices[usize::from(*index) * 6 + axis])
                .sum::<f64>()
                / cap.len() as f64
        });
        let center_index = self.push(center, normal);
        for (point, index) in cap.iter().enumerate() {
            let following = (point + 1) % cap.len();
            if reverse {
                self.indices.extend([center_index, cap[following], *index]);
            } else {
                self.indices.extend([center_index, *index, cap[following]]);
            }
        }
    }
}

/// Rounded prism, wound counter-clockwise from outside for back-face culling.
pub fn prism_mesh_data(triangle: Triangle, back_z: f64, front_z: f64) -> PrismMeshData {
    let depth = number_max(0.0, front_z - back_z);
    let radius = number_min(PRISM_BEVEL_RADIUS, depth * 0.45);
    let contour = rounded_triangle_contour(triangle, radius);
    let mut mesh = MeshBuilder::default();
    let mut rings = Vec::new();
    let mut add_ring = |theta: f64, z: f64, z_normal: f64| {
        let inset = radius * (1.0 - theta.cos());
        let xy_weight = theta.cos();
        let ring: Vec<_> = contour
            .iter()
            .map(|point| {
                mesh.push(
                    [
                        point.position[0] - point.normal[0] * inset,
                        point.position[1] - point.normal[1] * inset,
                        z,
                    ],
                    [
                        point.normal[0] * xy_weight,
                        point.normal[1] * xy_weight,
                        z_normal,
                    ],
                )
            })
            .collect();
        rings.push(ring);
    };
    // Stop short of pi/2 so cap corner arcs retain positive area.
    let max_theta = PI / 2.0 - 0.06;
    let max_sine = max_theta.sin();
    for step in (0..=PRISM_BEVEL_SEGMENTS).rev() {
        let theta = (max_theta * step as f64) / PRISM_BEVEL_SEGMENTS as f64;
        add_ring(
            theta,
            back_z + radius - (radius * theta.sin()) / max_sine,
            -theta.sin(),
        );
    }
    for step in 0..=PRISM_BEVEL_SEGMENTS {
        let theta = (max_theta * step as f64) / PRISM_BEVEL_SEGMENTS as f64;
        add_ring(
            theta,
            front_z - radius + (radius * theta.sin()) / max_sine,
            theta.sin(),
        );
    }
    for pair in rings.windows(2) {
        let current = &pair[0];
        let next = &pair[1];
        for (point, index) in current.iter().enumerate() {
            let following = (point + 1) % contour.len();
            mesh.indices.extend([
                *index,
                current[following],
                next[following],
                *index,
                next[following],
                next[point],
            ]);
        }
    }
    mesh.add_cap(&rings[0], [0.0, 0.0, -1.0], true);
    mesh.add_cap(&rings[rings.len() - 1], [0.0, 0.0, 1.0], false);
    PrismMeshData {
        vertices: mesh
            .vertices
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        indices: mesh.indices,
    }
}

/// Unique line-list edges, retaining first occurrence order.
pub fn prism_wireframe_indices(triangle_indices: &[u16]) -> Vec<u16> {
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for triangle in triangle_indices.as_chunks::<3>().0 {
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let start = a.min(b);
            let end = a.max(b);
            let key = u32::from(start) * 65_536 + u32::from(end);
            if seen.insert(key) {
                edges.extend([start, end]);
            }
        }
    }
    edges
}

/// Five ideal outward planes, dot(normal, point) <= d inside.
pub fn prism_planes(triangle: Triangle, back_z: f64, front_z: f64) -> Vec<[f64; 4]> {
    let corners = [triangle.a, triangle.b, triangle.c];
    let mut sides: Vec<_> = corners
        .iter()
        .enumerate()
        .map(|(edge, start)| {
            let [nx, ny] = outward_normal(*start, corners[(edge + 1) % 3]);
            [nx, ny, 0.0, nx * start[0] + ny * start[1]]
        })
        .collect();
    sides.extend([[0.0, 0.0, 1.0, front_z], [0.0, 0.0, -1.0, -back_z]]);
    sides
}

fn rounded_triangle_contour(triangle: Triangle, radius: f64) -> Vec<ContourPoint> {
    let corners = [triangle.a, triangle.b, triangle.c];
    let arcs: Vec<Vec<_>> = corners
        .iter()
        .enumerate()
        .map(|(index, corner)| {
            let previous = corners[(index + corners.len() - 1) % corners.len()];
            let next = corners[(index + 1) % corners.len()];
            let toward_previous = normalize2([previous[0] - corner[0], previous[1] - corner[1]]);
            let toward_next = normalize2([next[0] - corner[0], next[1] - corner[1]]);
            let half_angle = clamp(dot2(toward_previous, toward_next), -1.0, 1.0).acos() / 2.0;
            let tangent_distance = radius / number_max(half_angle.tan(), 1e-6);
            let center_distance = radius / number_max(half_angle.sin(), 1e-6);
            let bisector = normalize2([
                toward_previous[0] + toward_next[0],
                toward_previous[1] + toward_next[1],
            ]);
            let center = [
                corner[0] + bisector[0] * center_distance,
                corner[1] + bisector[1] * center_distance,
            ];
            let start = [
                corner[0] + toward_previous[0] * tangent_distance,
                corner[1] + toward_previous[1] * tangent_distance,
            ];
            let end = [
                corner[0] + toward_next[0] * tangent_distance,
                corner[1] + toward_next[1] * tangent_distance,
            ];
            let start_angle = (start[1] - center[1]).atan2(start[0] - center[0]);
            let mut end_angle = (end[1] - center[1]).atan2(end[0] - center[0]);
            while end_angle <= start_angle {
                end_angle += PI * 2.0;
            }
            (0..=PRISM_CORNER_SEGMENTS)
                .map(|step| {
                    let angle = start_angle
                        + ((end_angle - start_angle) * step as f64) / PRISM_CORNER_SEGMENTS as f64;
                    let normal = [angle.cos(), angle.sin()];
                    ContourPoint {
                        position: [
                            center[0] + normal[0] * radius,
                            center[1] + normal[1] * radius,
                        ],
                        normal,
                    }
                })
                .collect()
        })
        .collect();
    arcs.iter()
        .enumerate()
        .flat_map(|(index, arc)| {
            let end = arc[arc.len() - 1];
            let next_start = arcs[(index + 1) % arcs.len()][0];
            let edge_normal = outward_normal(corners[index], corners[(index + 1) % corners.len()]);
            let straight = (0..PRISM_EDGE_SEGMENTS - 1).map(move |step| {
                let amount = (step + 1) as f64 / PRISM_EDGE_SEGMENTS as f64;
                ContourPoint {
                    position: [
                        end.position[0] + (next_start.position[0] - end.position[0]) * amount,
                        end.position[1] + (next_start.position[1] - end.position[1]) * amount,
                    ],
                    normal: edge_normal,
                }
            });
            arc.iter().copied().chain(straight)
        })
        .collect()
}

fn normalize2(value: Vec2) -> Vec2 {
    let length = value[0].hypot(value[1]);
    let length = if length == 0.0 || length.is_nan() {
        1.0
    } else {
        length
    };
    [value[0] / length, value[1] / length]
}

fn dot2(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn outward_normal(start: Vec2, end: Vec2) -> Vec2 {
    let edge = [end[0] - start[0], end[1] - start[1]];
    let length = edge[0].hypot(edge[1]);
    let length = if length == 0.0 || length.is_nan() {
        1.0
    } else {
        length
    };
    [edge[1] / length, -edge[0] / length]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PRISM_BACK_Z, PRISM_FRONT_Z, PRISM_TRIANGLE};

    #[test]
    fn mesh_has_expected_size_and_finite_unit_normals() {
        let mesh = prism_mesh_data(*PRISM_TRIANGLE, PRISM_BACK_Z, PRISM_FRONT_Z);
        assert_eq!(mesh.vertices.len(), 4332);
        assert_eq!(mesh.indices.len(), 3600);
        assert!(
            mesh.indices
                .iter()
                .all(|index| usize::from(*index) < mesh.vertices.len() / 6)
        );
        for vertex in mesh.vertices.as_chunks::<6>().0 {
            assert!(vertex.iter().all(|value| value.is_finite()));
            let length = vertex[3].hypot(vertex[4]).hypot(vertex[5]);
            assert!((length - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn wireframe_keeps_only_first_occurrence_of_undirected_edges() {
        assert_eq!(
            prism_wireframe_indices(&[0, 1, 2, 2, 1, 3]),
            vec![0, 1, 1, 2, 0, 2, 1, 3, 2, 3]
        );
    }
}
