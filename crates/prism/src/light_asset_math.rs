//! Procedural light material mathematics, from light/assets/math.ts.
use crate::{clamp, number_max, number_min};
pub fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}
pub fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let unit = clamp01((value - edge0) / number_max(edge1 - edge0, 1e-8));
    unit * unit * (3.0 - 2.0 * unit)
}
fn fract(value: f64) -> f64 {
    value - value.floor()
}
fn hash(x: f64, y: f64) -> f64 {
    fract((x * 127.1 + y * 311.7).sin() * 43_758.545_312_3)
}
pub fn noise(x: f64, y: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = smoothstep(0.0, 1.0, fract(x));
    let fy = smoothstep(0.0, 1.0, fract(y));
    let top = hash(ix, iy) * (1.0 - fx) + hash(ix + 1.0, iy) * fx;
    let bottom = hash(ix, iy + 1.0) * (1.0 - fx) + hash(ix + 1.0, iy + 1.0) * fx;
    top * (1.0 - fy) + bottom * fy
}
pub fn fbm(x: f64, y: f64, octaves: usize) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    let mut weight = 0.0;
    for _ in 0..octaves {
        value += noise(x * frequency, y * frequency) * amplitude;
        weight += amplitude;
        amplitude *= 0.5;
        frequency *= 2.07;
    }
    value / weight
}
pub fn segment_distance(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let abx = b[0] - a[0];
    let aby = b[1] - a[1];
    let squared = abx * abx + aby * aby;
    let length_squared = if squared == 0.0 || squared.is_nan() {
        1.0
    } else {
        squared
    };
    let t = clamp01(((point[0] - a[0]) * abx + (point[1] - a[1]) * aby) / length_squared);
    (point[0] - (a[0] + abx * t)).hypot(point[1] - (a[1] + aby * t))
}
pub fn triangle_distance(point: [f64; 2], vertices: &[[f64; 2]]) -> f64 {
    vertices
        .iter()
        .enumerate()
        .map(|(index, &start)| {
            segment_distance(point, start, vertices[(index + 1) % vertices.len()])
        })
        .fold(f64::INFINITY, number_min)
}
pub fn triangle_contains(point: [f64; 2], vertices: &[[f64; 2]]) -> bool {
    let mut orientation = 0.0;
    for (index, start) in vertices.iter().enumerate() {
        let end = vertices[(index + 1) % vertices.len()];
        let cross = (end[0] - start[0]) * (point[1] - start[1])
            - (end[1] - start[1]) * (point[0] - start[0]);
        if cross.abs() < 1e-9 {
            continue;
        }
        let side = cross.signum();
        if orientation != 0.0 && side != orientation {
            return false;
        }
        orientation = side;
    }
    true
}
pub fn write_pixel(pixels: &mut [u8], index: usize, rgba: [f64; 4]) {
    for (channel, value) in rgba.into_iter().enumerate() {
        pixels[index + channel] = (clamp01(value) * 255.0).round() as u8;
    }
}
