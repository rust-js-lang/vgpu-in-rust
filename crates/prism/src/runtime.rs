//! Runtime lamp interpolation and wall sizing from runtime/state.ts.
use crate::camera::wall_half_height;
use crate::controls::BeamMouseYControls;
use crate::framing::{ProjectionFraming, framing_coverage};
use crate::types::{CollimatedLight, PRISM_INCIDENCE_DEGREES, lamp_for_incidence};
use crate::{clamp, number_max};

pub fn incidence_at(position: f64, beam_mouse_y: BeamMouseYControls) -> f64 {
    let position = clamp(position, 0.0, 1.0);
    if position <= 0.5 {
        beam_mouse_y.top + (PRISM_INCIDENCE_DEGREES - beam_mouse_y.top) * position * 2.0
    } else {
        PRISM_INCIDENCE_DEGREES
            + (beam_mouse_y.bottom - PRISM_INCIDENCE_DEGREES) * (position - 0.5) * 2.0
    }
}

pub fn lamp_at(
    position: f64,
    beam_width: f64,
    target_position: f64,
    beam_mouse_y: BeamMouseYControls,
) -> CollimatedLight {
    lamp_for_incidence(
        incidence_at(position, beam_mouse_y),
        beam_width,
        target_position,
    )
}

pub fn wall_extent(
    aspect: f64,
    camera_distance: f64,
    camera_fov: f64,
    framing: ProjectionFraming,
) -> [f64; 2] {
    let height = wall_half_height(aspect, camera_distance, camera_fov);
    let coverage = framing_coverage(framing);
    [height * aspect * coverage[0], height * coverage[1]]
}

/// Keeps the rainbow's dark tail outside narrow portrait canvases. The light
/// mesh is clipped by the real render target, so overscanning its wall boundary
/// changes only where normalized outgoing travel reaches one.
pub fn light_wall_extent(
    aspect: f64,
    camera_distance: f64,
    camera_fov: f64,
    framing: ProjectionFraming,
) -> [f64; 2] {
    let extent = wall_extent(aspect, camera_distance, camera_fov, framing);
    let overscan = clamp(1.0 / number_max(aspect, 1e-3), 1.0, 2.5);
    [extent[0] * overscan, extent[1] * overscan]
}
