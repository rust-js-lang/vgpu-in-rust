//! Parameters for the post-tone-mapped dust draw from dark/bind.ts.
use crate::camera::CameraView;
use crate::types::{PRISM_FRONT_Z, PRISM_LIGHT_PLANE_Z, PRISM_TRIANGLE};
use std::rc::Rc;

pub const DUST_PARTICLE_COUNT: usize = 2200;
pub struct DustUniforms {
    pub view_projection: Rc<[f32; 16]>,
    pub field_half_extent: [f64; 2],
    pub output_size: [f64; 2],
    pub time: f64,
    pub camera_distance: f64,
    pub light_plane_z: f64,
    pub prism_a: [f64; 2],
    pub prism_b: [f64; 2],
    pub prism_c: [f64; 2],
    pub prism_front_z: f64,
    pub reveal_progress: f64,
}
pub fn dust_uniforms(
    view: &CameraView,
    field_half_extent: [f64; 2],
    output_size: [f64; 2],
    camera_distance: f64,
    time: f64,
    reveal_progress: f64,
) -> DustUniforms {
    DustUniforms {
        view_projection: Rc::clone(&view.view_projection),
        field_half_extent,
        output_size,
        time,
        camera_distance,
        light_plane_z: PRISM_LIGHT_PLANE_Z,
        prism_a: PRISM_TRIANGLE.a,
        prism_b: PRISM_TRIANGLE.b,
        prism_c: PRISM_TRIANGLE.c,
        prism_front_z: PRISM_FRONT_Z,
        reveal_progress,
    }
}
