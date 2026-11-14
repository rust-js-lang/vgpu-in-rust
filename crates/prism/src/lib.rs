//! Prism scene mathematics, ported from vgpu at the revision in UPSTREAM.md.
pub mod framing;
pub mod interaction;
pub mod optics;
pub mod prism_mesh;
pub mod types;

/// JavaScript's Math.min and Math.max propagate NaN; Rust's f64 methods do not.
/// These helpers preserve the original numeric policy at the port boundary.
fn number_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}

fn number_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

fn clamp(value: f64, low: f64, high: f64) -> f64 {
    number_min(high, number_max(low, value))
}
pub mod camera;
