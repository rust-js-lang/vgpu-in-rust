//! Scene and glass uniform values from prism-background/runtime/uniforms.ts.
//! This module owns values; the WGSL reflection owns their byte layout.
use crate::camera::{CameraView, rotation_matrix};
use crate::clamp;
use crate::controls::{PrismControls, PrismTheme, PrismView};
use crate::light_mesh::{LIGHT_INTERNAL_SEGMENTS, LightMeshLayout};
use crate::prism_mesh::prism_planes;
use crate::types::{
    CollimatedLight, PRISM_BACK_Z, PRISM_FRONT_Z, PRISM_LIGHT_PLANE_Z, PRISM_TRIANGLE,
};
use std::rc::Rc;
use std::sync::LazyLock;

pub const ENVIRONMENT_SIZE: [f64; 2] = [1024.0, 512.0];
pub const ENVIRONMENT_LEVELS: usize = 8;
pub const ENVIRONMENT_TEXEL_ANGLE: f64 = 2.0 * std::f64::consts::PI / ENVIRONMENT_SIZE[0];
static ENVIRONMENT_ROTATION: LazyLock<[f32; 16]> = LazyLock::new(|| rotation_matrix([0.0; 3]));
static PRISM_PLANES: LazyLock<Vec<[f64; 4]>> =
    LazyLock::new(|| prism_planes(*PRISM_TRIANGLE, PRISM_BACK_Z, PRISM_FRONT_Z));

/// Schlick's normal-incidence reflectance, rounded like the shader's f32 math.
pub fn schlick_fresnel_f0(ior: f64) -> f32 {
    let ior = ior as f32;
    let ratio = (ior - 1.0) / (ior + 1.0);
    ratio * ratio
}

#[derive(Clone, Debug)]
pub struct SceneUniforms {
    pub view_projection: Rc<[f32; 16]>,
    pub wall_half_extent: [f64; 2],
    pub input_beam_direction: [f64; 2],
    pub wall_color: [f64; 3],
    pub caustic_only: u32,
    pub light_plane_z: f64,
    pub light_white_quads: usize,
    pub light_beam_slices: usize,
    pub light_spectral_samples: usize,
    pub light_internal_quads: usize,
    pub light_internal_segments: usize,
    pub light_opacity: f64,
    pub light_edge_falloff: f64,
    pub rainbow_falloff_rate: f64,
    pub rainbow_falloff_power: f64,
    pub beam_width_reveal: f64,
}

/// Shared block used by wall and light draws in either theme. Runtime state
/// supplies its current camera, wall extent and lamp so all passes agree.
pub fn scene_uniforms(
    view: &CameraView,
    wall_half_extent: [f64; 2],
    light: CollimatedLight,
    controls: &PrismControls,
    layout: &LightMeshLayout,
    beam_width_reveal: f64,
) -> SceneUniforms {
    let color = controls
        .wall_color
        .strip_prefix('#')
        .unwrap_or(&controls.wall_color);
    let wall_color = if color.len() == 6 && color.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        std::array::from_fn(|index| {
            u8::from_str_radix(&color[index * 2..index * 2 + 2], 16).expect("hex channel") as f64
                / 255.0
        })
    } else {
        [0.0; 3]
    };
    SceneUniforms {
        view_projection: Rc::clone(&view.view_projection),
        wall_half_extent,
        input_beam_direction: light.direction,
        wall_color,
        caustic_only: u32::from(controls.view == PrismView::Caustic),
        light_plane_z: PRISM_LIGHT_PLANE_Z,
        light_white_quads: layout.white_quads,
        light_beam_slices: layout.beam_slices,
        light_spectral_samples: layout.samples,
        light_internal_quads: layout.internal_quads,
        light_internal_segments: LIGHT_INTERNAL_SEGMENTS,
        light_opacity: controls.light_fade.beam_opacity,
        light_edge_falloff: controls.light_fade.edge_falloff,
        rainbow_falloff_rate: controls.light_fade.rainbow_falloff_rate,
        rainbow_falloff_power: controls.light_fade.rainbow_falloff_power,
        beam_width_reveal: clamp(beam_width_reveal, 0.0, 1.0),
    }
}

#[derive(Clone, Debug)]
pub struct GlassUniforms {
    pub view_projection: Rc<[f32; 16]>,
    pub environment_rotation: &'static [f32; 16],
    pub camera_position: [f64; 3],
    pub absorption: [f64; 3],
    pub prism_a: [f64; 2],
    pub prism_b: [f64; 2],
    pub prism_c: [f64; 2],
    pub environment_size: [f64; 2],
    pub front_z: f64,
    pub back_z: f64,
    pub ior: f64,
    pub reflection_strength: f64,
    pub environment_exposure: f64,
    pub environment_debug: u32,
    pub environment_texel_angle: f64,
    pub fresnel_f0: f32,
    pub prism_planes: &'static [[f64; 4]],
}

pub fn glass_uniforms(
    view: &CameraView,
    controls: &PrismControls,
    mode: PrismTheme,
    debug_environment_enabled: bool,
) -> GlassUniforms {
    let transmission = controls.glass.transmission.get(mode);
    let reflection = controls.glass.reflection.get(mode);
    GlassUniforms {
        view_projection: Rc::clone(&view.view_projection),
        environment_rotation: &ENVIRONMENT_ROTATION,
        camera_position: view.position,
        absorption: transmission.absorption,
        prism_a: PRISM_TRIANGLE.a,
        prism_b: PRISM_TRIANGLE.b,
        prism_c: PRISM_TRIANGLE.c,
        environment_size: ENVIRONMENT_SIZE,
        front_z: PRISM_FRONT_Z,
        back_z: PRISM_BACK_Z,
        ior: transmission.ior,
        reflection_strength: reflection.reflection_strength,
        environment_exposure: reflection.environment_exposure,
        environment_debug: u32::from(debug_environment_enabled && controls.environment_debug),
        environment_texel_angle: ENVIRONMENT_TEXEL_ANGLE,
        fresnel_f0: schlick_fresnel_f0(transmission.ior),
        prism_planes: &PRISM_PLANES,
    }
}
