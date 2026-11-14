//! Scene camera and derived wall coverage from prism-background/scene/camera.ts.
//! Camera storage rounds at the same Float32 boundaries as scene-camera.ts and
//! vgpu/scene; its ideal orbital basis remains double precision.
use std::cell::Cell;
use std::rc::Rc;

use crate::types::{CAMERA_ORBIT_DEGREES, CAMERA_PITCH_DEGREES, CAMERA_YAW_DEGREES, Vec3};
use crate::{clamp, number_max};

#[derive(Clone, Debug, PartialEq)]
pub struct Lens {
    pub fov: f64,
    pub near: f64,
    pub far: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraState {
    pub position: [f32; 3],
    pub quaternion: [f32; 4],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub lens: Lens,
    pub aspect: f64,
    pub projection: [f32; 16],
    pub view: [f32; 16],
    pub view_projection: Rc<[f32; 16]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraView {
    pub camera: CameraState,
    /// Responsive framing may replace this while retaining the original camera.
    pub view_projection: Rc<[f32; 16]>,
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

/// Slack on the derived wall size; the coverage test holds this margin.
const WALL_SAFETY: f64 = 1.02;

thread_local! {
    // Computing this costs nine cameras. It changes with canvas shape/controls.
    static WALL_SIZE: Cell<Option<(f64, f64, f64, f64)>> = const { Cell::new(None) };
}

/// The camera swings on a sphere and keeps looking at the origin, so the prism
/// stays put and only its parallax against the wall moves.
pub fn camera_view(aspect: f64, orbit_x: f64, orbit_y: f64, distance: f64, fov: f64) -> CameraView {
    let yaw = radians(CAMERA_YAW_DEGREES + clamp(orbit_x, -1.0, 1.0) * CAMERA_ORBIT_DEGREES);
    let pitch = radians(CAMERA_PITCH_DEGREES - clamp(orbit_y, -1.0, 1.0) * CAMERA_ORBIT_DEGREES);
    let cos_pitch = pitch.cos();
    let position = [
        yaw.sin() * cos_pitch * distance,
        pitch.sin() * distance,
        yaw.cos() * cos_pitch * distance,
    ];
    let forward = normalize(position.map(|value| -value));
    let right = normalize(cross(forward, [0.0, 1.0, 0.0]));
    let camera = create_camera_state(position, aspect, fov, 0.05, 4.0 * distance);
    CameraView {
        view_projection: Rc::clone(&camera.view_projection),
        camera,
        position,
        forward,
        right,
        up: cross(right, forward),
    }
}

/// Derived rather than chosen: cover the worst frustum corner over the whole
/// pointer swing. Optical scene dimensions remain fixed when the wall grows.
pub fn wall_half_height(aspect: f64, distance: f64, fov: f64) -> f64 {
    if let Some((previous_aspect, previous_distance, previous_fov, height)) = WALL_SIZE.get()
        && aspect == previous_aspect
        && distance == previous_distance
        && fov == previous_fov
    {
        return height;
    }
    let mut worst = 0.0;
    for orbit_x in [-1.0, 0.0, 1.0] {
        for orbit_y in [-1.0, 0.0, 1.0] {
            worst = number_max(
                worst,
                wall_coverage(aspect, orbit_x, orbit_y, distance, fov),
            );
        }
    }
    let height = worst * WALL_SAFETY;
    WALL_SIZE.set(Some((aspect, distance, fov, height)));
    height
}

/// Walks the four frustum corner rays to the wall plane at z = 0.
pub fn wall_coverage(aspect: f64, orbit_x: f64, orbit_y: f64, distance: f64, fov: f64) -> f64 {
    let view = camera_view(aspect, orbit_x, orbit_y, distance, fov);
    let tan_half_fov = (radians(fov) / 2.0).tan();
    let mut worst = 0.0;
    for horizontal in [-1.0, 1.0] {
        for vertical in [-1.0, 1.0] {
            let direction: Vec3 = std::array::from_fn(|axis| {
                view.forward[axis]
                    + view.right[axis] * horizontal * tan_half_fov * aspect
                    + view.up[axis] * vertical * tan_half_fov
            });
            // Every corner ray should cross z = 0 at a positive distance.
            if direction[2] >= 0.0 {
                return f64::INFINITY;
            }
            let t = -view.position[2] / direction[2];
            let x = view.position[0] + direction[0] * t;
            let y = view.position[1] + direction[1] * t;
            worst = number_max(number_max(worst, x.abs() / aspect), y.abs());
        }
    }
    worst
}

/// Column-major XYZ rotation for the studio environment's orientation.
pub fn rotation_matrix(degrees: Vec3) -> [f32; 16] {
    let [x, y, z] = degrees.map(radians);
    let (sx, cx) = (x.sin(), x.cos());
    let (sy, cy) = (y.sin(), y.cos());
    let (sz, cz) = (z.sin(), z.cos());
    [
        cy * cz,
        cy * sz,
        -sy,
        0.0,
        sx * sy * cz - cx * sz,
        sx * sy * sz + cx * cz,
        sx * cy,
        0.0,
        cx * sy * cz + sx * sz,
        cx * sy * sz - sx * cz,
        cx * cy,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
    .map(|value| value as f32)
}

fn create_camera_state(position: Vec3, aspect: f64, fov: f64, near: f64, far: f64) -> CameraState {
    // Scene controls and canvas sizing supply finite, positive camera values.
    // Round stored vectors before lookAt, and the quaternion before viewMatrices.
    let position = position.map(|value| value as f32);
    let quaternion = look_at_quaternion(position.map(f64::from)).map(|value| value as f32);
    let f = 1.0 / (fov * std::f64::consts::PI / 360.0).tan();
    let range = 1.0 / (near - far);
    let projection = [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        far * range,
        -1.0,
        0.0,
        0.0,
        far * near * range,
        0.0,
    ]
    .map(|value| value as f32);
    let [x, y, z, w] = normalize4(quaternion.map(f64::from));
    let (xx, xy, xz) = (x * x, x * y, x * z);
    let (yy, yz, zz) = (y * y, y * z, z * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);
    let (r00, r10, r20) = (1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy));
    let (r01, r11, r21) = (2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx));
    let (r02, r12, r22) = (2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy));
    let [px, py, pz] = position.map(f64::from);
    let view = [
        r00,
        r01,
        r02,
        0.0,
        r10,
        r11,
        r12,
        0.0,
        r20,
        r21,
        r22,
        0.0,
        -(r00 * px + r10 * py + r20 * pz),
        -(r01 * px + r11 * py + r21 * pz),
        -(r02 * px + r12 * py + r22 * pz),
        1.0,
    ]
    .map(|value| value as f32);
    let view_projection = std::array::from_fn(|index| {
        let row = index % 4;
        let column = index / 4 * 4;
        (f64::from(projection[row]) * f64::from(view[column])
            + f64::from(projection[row + 4]) * f64::from(view[column + 1])
            + f64::from(projection[row + 8]) * f64::from(view[column + 2])
            + f64::from(projection[row + 12]) * f64::from(view[column + 3])) as f32
    });
    CameraState {
        position,
        quaternion,
        target: [0.0; 3],
        up: [0.0, 1.0, 0.0],
        lens: Lens { fov, near, far },
        aspect,
        projection,
        view,
        view_projection: Rc::new(view_projection),
    }
}

fn look_at_quaternion(eye: Vec3) -> [f64; 4] {
    let length = magnitude(eye);
    if length == 0.0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let z = eye.map(|value| value / length);
    let mut x = cross([0.0, 1.0, 0.0], z);
    let mut x_length = magnitude(x);
    if x_length == 0.0 {
        x = [z[2], 0.0, -z[0]];
        x_length = magnitude(x);
        if x_length == 0.0 {
            x = [1.0, 0.0, 0.0];
            x_length = 1.0;
        }
    }
    let [m00, m01, m02] = x.map(|value| value / x_length);
    let [m10, m11, m12] = cross(z, [m00, m01, m02]);
    let [m20, m21, m22] = z;
    let trace = m00 + m11 + m22;
    let quaternion = if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        [(m12 - m21) * s, (m20 - m02) * s, (m01 - m10) * s, 0.25 / s]
    } else if m00 > m11 && m00 > m22 {
        let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
        [0.25 * s, (m10 + m01) / s, (m20 + m02) / s, (m12 - m21) / s]
    } else if m11 > m22 {
        let s = 2.0 * (1.0 + m11 - m00 - m22).sqrt();
        [(m10 + m01) / s, 0.25 * s, (m21 + m12) / s, (m20 - m02) / s]
    } else {
        let s = 2.0 * (1.0 + m22 - m00 - m11).sqrt();
        [(m20 + m02) / s, (m21 + m12) / s, 0.25 * s, (m01 - m10) / s]
    };
    normalize4(quaternion)
}

fn radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}
// Scale before squaring, then compensate summation error. Normalizing through
// chained two-axis lengths adds rounding before the final square root; after
// Float32 quaternion storage, that changes the view's cancellation residuals.
fn magnitude<const N: usize>(value: [f64; N]) -> f64 {
    if value.iter().any(|component| component.is_infinite()) {
        return f64::INFINITY;
    }
    if value.iter().any(|component| component.is_nan()) {
        return f64::NAN;
    }
    let maximum = value
        .iter()
        .fold(0.0_f64, |maximum, component| maximum.max(component.abs()));
    if maximum == 0.0 {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut compensation = 0.0;
    for component in value {
        let scaled = component / maximum;
        let adjusted = scaled * scaled - compensation;
        let next = sum + adjusted;
        compensation = (next - sum) - adjusted;
        sum = next;
    }
    maximum * sum.sqrt()
}
fn normalize(value: Vec3) -> Vec3 {
    let length = magnitude(value);
    let length = if length == 0.0 || length.is_nan() {
        1.0
    } else {
        length
    };
    value.map(|component| component / length)
}
fn normalize4(value: [f64; 4]) -> [f64; 4] {
    let length = magnitude(value);
    value.map(|component| component / length)
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CAMERA_DISTANCE, CAMERA_FOV_DEGREES};

    #[test]
    fn camera_orbits_without_losing_the_origin_or_matrix_identity() {
        for (x, y) in [(-1.0, -1.0), (0.0, 0.0), (0.7, -0.3), (1.0, 1.0)] {
            let view = camera_view(16.0 / 9.0, x, y, CAMERA_DISTANCE, CAMERA_FOV_DEGREES);
            assert!((magnitude(view.position) - CAMERA_DISTANCE).abs() < 1e-12);
            assert!((magnitude(view.forward) - 1.0).abs() < 1e-12);
            assert!((magnitude(view.right) - 1.0).abs() < 1e-12);
            assert!(Rc::ptr_eq(
                &view.view_projection,
                &view.camera.view_projection
            ));
            let matrix = &view.view_projection;
            assert!(f64::from(matrix[12]).abs() < 1e-7);
            assert!(f64::from(matrix[13]).abs() < 1e-7);
            assert!(matrix[15] > 0.0);
        }
    }

    #[test]
    fn derived_wall_covers_the_full_swing_and_cache_keys_include_camera_controls() {
        for aspect in [0.25, 1.0, 16.0 / 9.0, 3.5] {
            let height = wall_half_height(aspect, CAMERA_DISTANCE, CAMERA_FOV_DEGREES);
            for x in [-1.0, 0.0, 1.0] {
                for y in [-1.0, 0.0, 1.0] {
                    let needed = wall_coverage(aspect, x, y, CAMERA_DISTANCE, CAMERA_FOV_DEGREES);
                    assert!(height >= needed * WALL_SAFETY);
                }
            }
            assert_eq!(
                height,
                wall_half_height(aspect, CAMERA_DISTANCE, CAMERA_FOV_DEGREES)
            );
            assert!(wall_half_height(aspect, CAMERA_DISTANCE * 2.0, CAMERA_FOV_DEGREES) > height);
            assert!(wall_half_height(aspect, CAMERA_DISTANCE, CAMERA_FOV_DEGREES + 10.0) > height);
        }
    }

    #[test]
    fn environment_rotation_preserves_homogeneous_axes() {
        let matrix = rotation_matrix([20.0, 35.0, -15.0]);
        assert_eq!(
            [matrix[3], matrix[7], matrix[11], matrix[15]],
            [0.0, 0.0, 0.0, 1.0]
        );
        for column in 0..3 {
            let offset = column * 4;
            let length = magnitude([
                matrix[offset].into(),
                matrix[offset + 1].into(),
                matrix[offset + 2].into(),
            ]);
            assert!((length - 1.0).abs() < 1e-6);
        }
    }
}
