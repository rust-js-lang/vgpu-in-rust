//! Shared scene definitions from prism-background/types.ts.
//! Constants live here so optics and interaction share one scene definition.
pub type Vec2 = [f64; 2];
pub type Vec3 = [f64; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

/// The neutral 60-degree shot sits at the vertical centre of the viewport.
pub const PRISM_DEFAULT_ARC: f64 = 0.5;
/// Per-frame interpolation towards the pointer's requested lamp position.
pub const LAMP_AIM_LERP: f64 = 0.12;
/// Per-frame interpolation towards the pointer's camera angle.
pub const CAMERA_ORBIT_LERP: f64 = 0.08;
/// Internal reflections a ray may take before the analytic solver gives up.
pub const PRISM_MAX_INTERNAL_BOUNCES: usize = 3;
/// Visible wavelength range, in nanometres.
pub const PRISM_WAVELENGTH_MIN: f64 = 400.0;
pub const PRISM_WAVELENGTH_MAX: f64 = 700.0;

use std::sync::LazyLock;

use crate::{clamp, number_max, number_min};

/// A finite collimated beam emitted from one point and aimed at another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollimatedLight {
    pub center: Vec2,
    pub direction: Vec2,
    pub beam_half_width: f64,
}

/// Side length of the equilateral prism, in scene units.
pub const PRISM_SIDE: f64 = 0.57;
/// Upright from the resting front camera.
pub const PRISM_TILT_DEGREES: f64 = 0.0;
/// The prism stands in the middle of the wall.
pub const PRISM_CENTROID: Vec2 = [0.0, 0.0];
pub const PRISM_DEFAULT_BEAM_WIDTH: f64 = 0.025;
pub const PRISM_BEAM_WIDTH_MIN: f64 = 0.01;
pub const PRISM_BEAM_WIDTH_MAX: f64 = 0.2;
pub const PRISM_LAMP_DISTANCE: f64 = 6.5;
pub const PRISM_INCIDENCE_DEGREES: f64 = 60.0;
pub const PRISM_SPECTRAL_SAMPLES: usize = 64 * 2;
pub const PRISM_BEAM_SLICES: usize = 24;
pub const PRISM_LIGHT_EXPOSURE: f64 = 88.0;
pub const PRISM_DEPTH: f64 = 0.3;
pub const PRISM_WALL_GAP: f64 = 0.015;
pub const PRISM_BACK_Z: f64 = PRISM_WALL_GAP;
pub const PRISM_FRONT_Z: f64 = PRISM_WALL_GAP + PRISM_DEPTH;
pub const PRISM_LIGHT_PLANE_Z: f64 = (PRISM_BACK_Z + PRISM_FRONT_Z) * 0.5;
pub const CAMERA_FOV_DEGREES: f64 = 48.0;
pub const CAMERA_DISTANCE: f64 = 1.25;
pub const CAMERA_YAW_DEGREES: f64 = 0.0;
pub const CAMERA_PITCH_DEGREES: f64 = 0.0;
pub const CAMERA_ORBIT_DEGREES: f64 = 3.5;

/// Equilateral prism, apex up, tilted, wound counter-clockwise.
///
/// The winding matters: optics takes each edge's outward normal to be
/// (edge.y, -edge.x), which only points outward for counter-clockwise vertices.
pub static PRISM_TRIANGLE: LazyLock<Triangle> = LazyLock::new(|| {
    let circumradius = PRISM_SIDE / 3.0_f64.sqrt();
    let vertex = |degrees: f64| {
        let spun = rotate([circumradius, 0.0], radians(degrees + PRISM_TILT_DEGREES));
        [PRISM_CENTROID[0] + spun[0], PRISM_CENTROID[1] + spun[1]]
    };
    Triangle {
        a: vertex(90.0),
        b: vertex(210.0),
        c: vertex(330.0),
    }
});

/// Midpoint of the right face the beam enters, and the default aim point.
pub static PRISM_ENTRY_FACE_MIDPOINT: LazyLock<Vec2> = LazyLock::new(|| {
    [
        (PRISM_TRIANGLE.a[0] + PRISM_TRIANGLE.c[0]) / 2.0,
        (PRISM_TRIANGLE.a[1] + PRISM_TRIANGLE.c[1]) / 2.0,
    ]
});

/// The default lamp, at PRISM_INCIDENCE_DEGREES.
pub static PRISM_LIGHT: LazyLock<CollimatedLight> =
    LazyLock::new(|| lamp_for_incidence(PRISM_INCIDENCE_DEGREES, PRISM_DEFAULT_BEAM_WIDTH, 0.5));

pub fn clamp_beam_width(width: f64) -> f64 {
    if !width.is_finite() {
        PRISM_DEFAULT_BEAM_WIDTH
    } else {
        clamp(width, PRISM_BEAM_WIDTH_MIN, PRISM_BEAM_WIDTH_MAX)
    }
}

/// Point along the entry edge, ordered left-to-right on screen.
pub fn prism_entry_point(position: f64) -> Vec2 {
    let clamped = clamp(position, 0.0, 1.0);
    [
        PRISM_TRIANGLE.a[0] + (PRISM_TRIANGLE.c[0] - PRISM_TRIANGLE.a[0]) * clamped,
        PRISM_TRIANGLE.a[1] + (PRISM_TRIANGLE.c[1] - PRISM_TRIANGLE.a[1]) * clamped,
    ]
}

pub fn collimated_light_between(center: Vec2, target: Vec2, beam_width: f64) -> CollimatedLight {
    let offset = [target[0] - center[0], target[1] - center[1]];
    let distance = offset[0].hypot(offset[1]);
    assert!(
        distance.is_finite() && distance > 1e-8,
        "A collimated light needs distinct finite center and target points."
    );
    CollimatedLight {
        center,
        direction: [offset[0] / distance, offset[1] / distance],
        beam_half_width: clamp_beam_width(beam_width) * 0.5,
    }
}

/// The prism never moves: the lamp swings around it on a fixed radius,
/// always aimed at a point along the entry face. Incidence and impact point
/// can therefore move independently.
pub fn lamp_for_incidence(
    incidence_degrees: f64,
    beam_width: f64,
    entry_position: f64,
) -> CollimatedLight {
    let face = [
        PRISM_TRIANGLE.a[0] - PRISM_TRIANGLE.c[0],
        PRISM_TRIANGLE.a[1] - PRISM_TRIANGLE.c[1],
    ];
    let face_length = face[0].hypot(face[1]);
    // Flip the outward normal to point into the glass: zero incidence.
    let inward = [-face[1] / face_length, face[0] / face_length];
    // Mirror the former left-entry setup: white arrives from the right,
    // and its dispersed output heads left.
    let direction = rotate(inward, radians(-incidence_degrees));
    let clamped_beam_width = clamp_beam_width(beam_width);
    // Keep both finite beam boundaries on the face at oblique incidence.
    let entry_margin = number_min(
        0.45,
        clamped_beam_width
            / (2.0 * face_length * number_max(0.05, radians(incidence_degrees).cos().abs()))
            + 1e-4,
    );
    let entry_point = prism_entry_point(clamp(entry_position, entry_margin, 1.0 - entry_margin));
    collimated_light_between(
        [
            entry_point[0] - direction[0] * PRISM_LAMP_DISTANCE,
            entry_point[1] - direction[1] * PRISM_LAMP_DISTANCE,
        ],
        entry_point,
        clamped_beam_width,
    )
}

fn radians(degrees: f64) -> f64 {
    (degrees * std::f64::consts::PI) / 180.0
}

fn rotate(point: Vec2, angle: f64) -> Vec2 {
    let cosine = angle.cos();
    let sine = angle.sin();
    [
        point[0] * cosine - point[1] * sine,
        point[0] * sine + point[1] * cosine,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scene_geometry_and_lamp_agree() {
        assert!(crate::optics::triangle_winding(*PRISM_TRIANGLE) > 0.0);
        let midpoint = prism_entry_point(0.5);
        for (actual, expected) in midpoint.iter().zip(PRISM_ENTRY_FACE_MIDPOINT.iter()) {
            assert!((actual - expected).abs() < 1e-12);
        }
        assert_eq!(clamp_beam_width(f64::NAN), PRISM_DEFAULT_BEAM_WIDTH);
        assert_eq!(clamp_beam_width(-1.0), PRISM_BEAM_WIDTH_MIN);
        assert_eq!(clamp_beam_width(1.0), PRISM_BEAM_WIDTH_MAX);
        assert!((PRISM_LIGHT.direction[0].hypot(PRISM_LIGHT.direction[1]) - 1.0).abs() < 1e-12);
        assert_eq!(PRISM_LIGHT.beam_half_width, PRISM_DEFAULT_BEAM_WIDTH * 0.5);
    }

    #[test]
    #[should_panic(expected = "A collimated light needs distinct finite center and target points.")]
    fn lamp_rejects_coincident_points() {
        collimated_light_between([0.0, 0.0], [0.0, 0.0], PRISM_DEFAULT_BEAM_WIDTH);
    }
}

/// Cauchy dispersion coefficients. Stylized widens real glass's colored edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DispersionPreset {
    pub base: f64,
    pub strength: f64,
}

pub const STYLIZED_DISPERSION: DispersionPreset = DispersionPreset {
    base: 1.245,
    strength: 0.06,
};
pub const CROWN_DISPERSION: DispersionPreset = DispersionPreset {
    base: 1.5046,
    strength: 0.0042,
};
pub const FLINT_DISPERSION: DispersionPreset = DispersionPreset {
    base: 1.664,
    strength: 0.0105,
};
pub const DEFAULT_LIGHT_EDGE_FALLOFF: f64 = 16.0;
