//! Light-theme look development and shader values from light/config.ts and uniforms.ts.
use crate::color::hex_color;
use crate::controls::{LightToneMapping, PrismTheme};
use crate::presentation::presentation_reveal_uniforms;
use crate::runtime_state::RuntimeState;
use crate::shadow_mesh::ShadowMeshOptions;
use crate::types::{PRISM_CENTROID, PRISM_SIDE};
use std::rc::Rc;

pub const LIGHT_SHADOW_MESH: ShadowMeshOptions = ShadowMeshOptions {
    projection: [PRISM_SIDE * 0.65, -PRISM_SIDE * 0.78],
    near_penumbra: PRISM_SIDE * 0.015,
    far_penumbra: PRISM_SIDE * 0.1,
    mid_ring: 0.48,
    mid_coverage: 0.32,
};
#[derive(Clone, Copy)]
pub struct WallTuning {
    pub material_scale: f64,
    pub normal_strength: f64,
    pub micro_normal_frequency: f64,
    pub micro_normal_strength: f64,
    pub ambient: f64,
    pub prism_shadow_strength: f64,
    pub prism_ao_strength: f64,
    pub grounding_scale: f64,
}
#[derive(Clone, Copy)]
pub struct CausticTuning {
    pub far_desaturation: f64,
    pub far_brightness: f64,
    pub travel_scale: f64,
    pub falloff_rate_scale: f64,
    pub falloff_power_scale: f64,
}
#[derive(Clone, Copy)]
pub struct LightPipelineTuning {
    pub wall: WallTuning,
    pub caustic: CausticTuning,
}
pub const LIGHT_PIPELINE_TUNING: LightPipelineTuning = LightPipelineTuning {
    wall: WallTuning {
        material_scale: 2.4,
        normal_strength: 0.22,
        micro_normal_frequency: 7.0,
        micro_normal_strength: 1.05,
        ambient: 0.5,
        prism_shadow_strength: 1.0,
        prism_ao_strength: 1.0,
        grounding_scale: 2.0,
    },
    caustic: CausticTuning {
        far_desaturation: 0.04,
        far_brightness: 0.02,
        travel_scale: 1.0,
        falloff_rate_scale: 0.12,
        falloff_power_scale: 0.5,
    },
};
#[derive(Clone, Copy)]
pub struct GlassAccentTuning {
    pub band_center: f64,
    pub band_width: f64,
    pub band_strength: f64,
    pub base_reflection: f64,
    pub rim_strength: f64,
    pub base_rim_strength: f64,
    pub environment_lod_bias: f64,
    pub highlight_strength: f64,
}
pub const LIGHT_GLASS_ACCENT_TUNING: GlassAccentTuning = GlassAccentTuning {
    band_center: 0.052,
    band_width: 0.034,
    band_strength: 0.52,
    base_reflection: 0.05,
    rim_strength: 0.45,
    base_rim_strength: 0.28,
    environment_lod_bias: 1.6,
    highlight_strength: 0.95,
};
pub fn light_glass_accent_uniforms() -> GlassAccentTuning {
    LIGHT_GLASS_ACCENT_TUNING
}

pub struct LightWallUniforms {
    pub view_projection: Rc<[f32; 16]>,
    pub wall_half_extent: [f64; 2],
    pub wall_color: [f64; 3],
    pub prism_center: [f64; 2],
    pub light_direction: [f64; 3],
    pub material_world_scale: f64,
    pub normal_strength: f64,
    pub micro_normal_frequency: f64,
    pub micro_normal_strength: f64,
    pub ambient: f64,
    pub ambient_light_strength: f64,
    pub global_light_transfer: f64,
    pub shadow_contrast: f64,
    pub shadow_pivot: f64,
    pub shadow_floor: f64,
    pub highlight_exposure: f64,
    pub prism_shadow_strength: f64,
    pub prism_ao_strength: f64,
    pub grounding_scale: f64,
}
pub fn light_wall_uniforms(state: &RuntimeState) -> LightWallUniforms {
    let tuning = LIGHT_PIPELINE_TUNING.wall;
    let controls = &state.controls.light_mode.wall;
    LightWallUniforms {
        view_projection: Rc::clone(&state.view.view_projection),
        wall_half_extent: state.wall_extent(),
        wall_color: hex_color(&state.controls.wall_color).unwrap_or([0.87; 3]),
        prism_center: PRISM_CENTROID,
        // A more grazing upper-left key makes the plaster normals readable while
        // the baked HDR blobs remain responsible for the white illumination peaks.
        light_direction: [-0.48, 0.56, 0.68],
        material_world_scale: PRISM_SIDE * tuning.material_scale,
        normal_strength: tuning.normal_strength * controls.normal_strength,
        micro_normal_frequency: tuning.micro_normal_frequency,
        micro_normal_strength: tuning.micro_normal_strength * controls.normal_strength,
        ambient: tuning.ambient,
        ambient_light_strength: controls.ambient_fill,
        global_light_transfer: controls.lightmap_gamma,
        shadow_contrast: controls.shadow_contrast,
        shadow_pivot: controls.shadow_pivot,
        shadow_floor: controls.shadow_floor,
        highlight_exposure: controls.highlight_exposure,
        // The broad cast shadow is a geometry draw. Preserve only the separately
        // baked contact/AO channel in the wall material composition.
        prism_shadow_strength: 0.0,
        prism_ao_strength: tuning.prism_ao_strength,
        grounding_scale: PRISM_SIDE * tuning.grounding_scale,
    }
}
pub struct LightCausticUniforms {
    pub strength: f64,
    pub coverage: f64,
    pub far_desaturation: f64,
    pub far_brightness: f64,
    pub travel_scale: f64,
    pub falloff_rate_scale: f64,
    pub falloff_power_scale: f64,
    pub material_world_scale: f64,
    pub normal_strength: f64,
    pub micro_normal_frequency: f64,
    pub micro_normal_strength: f64,
    pub normal_influence: f64,
    pub normal_elevation: f64,
}
pub fn light_caustic_uniforms(state: &RuntimeState) -> LightCausticUniforms {
    let tuning = LIGHT_PIPELINE_TUNING.caustic;
    let wall = LIGHT_PIPELINE_TUNING.wall;
    let controls = &state.controls.light_mode.caustic;
    let wall_controls = &state.controls.light_mode.wall;
    LightCausticUniforms {
        strength: controls.strength,
        coverage: controls.coverage,
        far_desaturation: tuning.far_desaturation,
        far_brightness: tuning.far_brightness,
        // Light-mesh travel is already normalized from the prism to the wall edge.
        travel_scale: tuning.travel_scale,
        falloff_rate_scale: tuning.falloff_rate_scale,
        falloff_power_scale: tuning.falloff_power_scale,
        material_world_scale: PRISM_SIDE * wall.material_scale,
        normal_strength: wall.normal_strength * wall_controls.normal_strength,
        micro_normal_frequency: wall.micro_normal_frequency,
        micro_normal_strength: wall.micro_normal_strength * wall_controls.normal_strength,
        normal_influence: controls.normal_influence,
        normal_elevation: controls.normal_elevation,
    }
}
pub struct LightPresentUniforms {
    pub background_color: [f64; 3],
    pub reveal_progress: f64,
    pub exposure: f64,
    pub tone_mapping: u32,
}
pub fn light_present_uniforms(state: &RuntimeState, reveal_progress: f64) -> LightPresentUniforms {
    let output = &state.controls.light_mode.output;
    let reveal = presentation_reveal_uniforms(PrismTheme::Light, reveal_progress);
    LightPresentUniforms {
        background_color: reveal.background_color,
        reveal_progress: reveal.reveal_progress,
        exposure: output.exposure,
        tone_mapping: match output.tone_mapping {
            LightToneMapping::Aces => 0,
            LightToneMapping::Neutral => 1,
            LightToneMapping::Reinhard => 2,
            LightToneMapping::Clamp => 3,
        },
    }
}
pub struct PrismShadowUniforms {
    pub view_projection: Rc<[f32; 16]>,
    pub color: [f64; 3],
    pub opacity: f64,
    pub far_strength: f64,
}
pub fn prism_shadow_uniforms(view_projection: Rc<[f32; 16]>) -> PrismShadowUniforms {
    PrismShadowUniforms {
        view_projection,
        color: [0.04, 0.037, 0.033],
        opacity: 0.46,
        far_strength: 0.92,
    }
}
