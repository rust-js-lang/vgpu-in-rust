//! Swept convex prism shadow with two soft penumbra rings, from light/passes/shadow/mesh.ts.
use crate::{
    number_max,
    types::{Triangle, Vec2},
};
#[derive(Clone, Copy, Debug)]
pub struct ShadowMeshOptions {
    pub projection: Vec2,
    pub near_penumbra: f64,
    pub far_penumbra: f64,
    pub mid_ring: f64,
    pub mid_coverage: f64,
}
pub struct ShadowMeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub hull: Vec<Vec2>,
    pub hull_travel: Vec<f64>,
}
#[derive(Clone, Copy)]
struct CastPoint {
    position: Vec2,
    travel: f64,
}
pub fn build_shadow_mesh(
    triangle: Triangle,
    options: ShadowMeshOptions,
) -> Result<ShadowMeshData, String> {
    let points: Vec<CastPoint> = [triangle.a, triangle.b, triangle.c]
        .into_iter()
        .flat_map(|position| {
            [
                CastPoint {
                    position,
                    travel: 0.0,
                },
                CastPoint {
                    position: [
                        position[0] + options.projection[0],
                        position[1] + options.projection[1],
                    ],
                    travel: 1.0,
                },
            ]
        })
        .collect();
    let hull = convex_hull(&points);
    if hull.len() < 3 {
        return Err("A cast-shadow hull needs three points.".into());
    }
    let positions: Vec<Vec2> = hull.iter().map(|point| point.position).collect();
    let travel: Vec<f64> = hull.iter().map(|point| point.travel).collect();
    let widths: Vec<f64> = travel
        .iter()
        .map(|value| options.near_penumbra + (options.far_penumbra - options.near_penumbra) * value)
        .collect();
    let middle = offset_convex_polygon(
        &positions,
        &widths
            .iter()
            .map(|width| width * options.mid_ring)
            .collect::<Vec<_>>(),
    );
    let outer = offset_convex_polygon(&positions, &widths);
    let center = polygon_centroid(&positions);
    let center_travel = travel.iter().sum::<f64>() / travel.len() as f64;
    let mut vertices = vec![
        center[0] as f32,
        center[1] as f32,
        1.0,
        center_travel as f32,
    ];
    append_ring(&mut vertices, &positions, 1.0, &travel);
    append_ring(&mut vertices, &middle, options.mid_coverage, &travel);
    append_ring(&mut vertices, &outer, 0.0, &travel);
    let count = positions.len();
    let core_start = 1;
    let middle_start = core_start + count;
    let outer_start = middle_start + count;
    let mut indices = Vec::new();
    for index in 0..count {
        let next = (index + 1) % count;
        indices.extend([0, (core_start + index) as u32, (core_start + next) as u32]);
        append_quad(
            &mut indices,
            core_start + index,
            core_start + next,
            middle_start + index,
            middle_start + next,
        );
        append_quad(
            &mut indices,
            middle_start + index,
            middle_start + next,
            outer_start + index,
            outer_start + next,
        );
    }
    // u32 keeps every valid polygon size aligned for queue.writeBuffer; a
    // five-vertex hull produces 75 indices, whose u16 byte length is not / 4.
    Ok(ShadowMeshData {
        vertices,
        indices,
        hull: positions,
        hull_travel: travel,
    })
}
fn append_ring(target: &mut Vec<f32>, positions: &[Vec2], coverage: f64, travel: &[f64]) {
    for (index, position) in positions.iter().enumerate() {
        target.extend([
            position[0] as f32,
            position[1] as f32,
            coverage as f32,
            travel[index] as f32,
        ]);
    }
}
fn append_quad(
    target: &mut Vec<u32>,
    inner: usize,
    inner_next: usize,
    outer: usize,
    outer_next: usize,
) {
    target.extend(
        [inner, outer, outer_next, inner, outer_next, inner_next].map(|index| index as u32),
    );
}
fn convex_hull(points: &[CastPoint]) -> Vec<CastPoint> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|left, right| {
        let x = left.position[0] - right.position[0];
        let difference = if x == 0.0 || x.is_nan() {
            left.position[1] - right.position[1]
        } else {
            x
        };
        difference
            .partial_cmp(&0.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let half = |entries: &[CastPoint]| {
        let mut result: Vec<CastPoint> = Vec::new();
        for &point in entries {
            while result.len() >= 2
                && cross(
                    result[result.len() - 2].position,
                    result[result.len() - 1].position,
                    point.position,
                ) <= 1e-9
            {
                result.pop();
            }
            result.push(point);
        }
        result
    };
    let mut lower = half(&sorted);
    sorted.reverse();
    let mut upper = half(&sorted);
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}
fn offset_convex_polygon(polygon: &[Vec2], widths: &[f64]) -> Vec<Vec2> {
    polygon
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let previous = polygon[(index + polygon.len() - 1) % polygon.len()];
            let next = polygon[(index + 1) % polygon.len()];
            let incoming = normalize([point[0] - previous[0], point[1] - previous[1]]);
            let outgoing = normalize([next[0] - point[0], next[1] - point[1]]);
            let previous_normal = [incoming[1], -incoming[0]];
            let next_normal = [outgoing[1], -outgoing[0]];
            let miter = normalize([
                previous_normal[0] + next_normal[0],
                previous_normal[1] + next_normal[1],
            ]);
            let scale = widths[index]
                / number_max(miter[0] * next_normal[0] + miter[1] * next_normal[1], 0.25);
            [point[0] + miter[0] * scale, point[1] + miter[1] * scale]
        })
        .collect()
}
fn polygon_centroid(polygon: &[Vec2]) -> Vec2 {
    let mut weighted_x = 0.0;
    let mut weighted_y = 0.0;
    let mut signed_area = 0.0;
    for (index, point) in polygon.iter().enumerate() {
        let next = polygon[(index + 1) % polygon.len()];
        let area = point[0] * next[1] - next[0] * point[1];
        weighted_x += (point[0] + next[0]) * area;
        weighted_y += (point[1] + next[1]) * area;
        signed_area += area;
    }
    let divisor = 3.0 * signed_area;
    [weighted_x / divisor, weighted_y / divisor]
}
fn normalize(value: Vec2) -> Vec2 {
    let length = value[0].hypot(value[1]);
    if length > 1e-9 {
        [value[0] / length, value[1] / length]
    } else {
        [0.0; 2]
    }
}
fn cross(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swept_mesh_is_indexed_aligned_and_rejects_degenerate_hulls() {
        let triangle = Triangle {
            a: [0.0, 1.0],
            b: [-1.0, 0.0],
            c: [1.0, 0.0],
        };
        let options = ShadowMeshOptions {
            projection: [0.5, -0.7],
            near_penumbra: 0.01,
            far_penumbra: 0.1,
            mid_ring: 0.48,
            mid_coverage: 0.32,
        };
        let mesh = build_shadow_mesh(triangle, options).unwrap();
        assert!(mesh.vertices.iter().all(|value| value.is_finite()));
        assert!(
            mesh.indices
                .iter()
                .all(|index| (*index as usize) < mesh.vertices.len() / 4)
        );
        assert_eq!(mesh.indices.len(), mesh.hull.len() * 15);
        assert_eq!(mesh.vertices.len(), (1 + 3 * mesh.hull.len()) * 4);
        assert!(
            build_shadow_mesh(
                Triangle {
                    a: [0.0; 2],
                    b: [0.0; 2],
                    c: [0.0; 2]
                },
                ShadowMeshOptions {
                    projection: [0.0; 2],
                    ..options
                }
            )
            .is_err()
        );
    }
}
