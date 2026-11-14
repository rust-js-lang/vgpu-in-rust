//! Complete typed runtime normalization from runtime/normalize-controls.ts.
//! Rust callers already provide the schema; numeric fallback/clamping remains identical.
use crate::control_ranges::{
    beam_mouse_y, camera, light_caustic, light_fade, light_output, light_wall, spectral_dispersion,
};
use crate::controls::*;
use crate::types::{
    CROWN_DISPERSION, DispersionPreset, FLINT_DISPERSION, STYLIZED_DISPERSION, clamp_beam_width,
};
fn finite(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}
fn normalize_transmission(
    input: GlassTransmissionControls,
    defaults: &GlassTransmissionControls,
) -> GlassTransmissionControls {
    GlassTransmissionControls {
        ior: finite(input.ior, defaults.ior),
        absorption: std::array::from_fn(|index| {
            finite(input.absorption[index], defaults.absorption[index])
        }),
    }
}
fn normalize_reflection(
    input: GlassReflectionControls,
    defaults: &GlassReflectionControls,
) -> GlassReflectionControls {
    GlassReflectionControls {
        reflection_strength: finite(input.reflection_strength, defaults.reflection_strength),
        environment_exposure: finite(input.environment_exposure, defaults.environment_exposure),
    }
}
pub fn dispersion_preset(dispersion: PrismDispersion) -> DispersionPreset {
    match dispersion {
        PrismDispersion::Stylized => STYLIZED_DISPERSION,
        PrismDispersion::Crown => CROWN_DISPERSION,
        PrismDispersion::Flint => FLINT_DISPERSION,
    }
}
pub fn normalize_controls(controls: &PrismControls) -> PrismControls {
    let mut controls = controls.clone();
    let defaults = PrismControls::default();
    controls.camera_fov = camera::FOV.normalize(controls.camera_fov, defaults.camera_fov);
    controls.beam_width = clamp_beam_width(controls.beam_width);
    let dispersion = controls
        .spectral_dispersion
        .unwrap_or_else(|| dispersion_preset(controls.dispersion));
    controls.spectral_dispersion = Some(DispersionPreset {
        base: spectral_dispersion::BASE.normalize(dispersion.base, STYLIZED_DISPERSION.base),
        strength: spectral_dispersion::STRENGTH
            .normalize(dispersion.strength, STYLIZED_DISPERSION.strength),
    });
    controls.beam_mouse_y.top =
        beam_mouse_y::TOP.normalize(controls.beam_mouse_y.top, defaults.beam_mouse_y.top);
    controls.beam_mouse_y.bottom =
        beam_mouse_y::BOTTOM.normalize(controls.beam_mouse_y.bottom, defaults.beam_mouse_y.bottom);
    controls.light_fade.beam_opacity = light_fade::BEAM_OPACITY.normalize(
        controls.light_fade.beam_opacity,
        defaults.light_fade.beam_opacity,
    );
    controls.light_fade.edge_falloff = light_fade::EDGE_FALLOFF.normalize(
        controls.light_fade.edge_falloff,
        defaults.light_fade.edge_falloff,
    );
    controls.light_fade.rainbow_falloff_rate = light_fade::RAINBOW_FALLOFF_RATE.normalize(
        controls.light_fade.rainbow_falloff_rate,
        defaults.light_fade.rainbow_falloff_rate,
    );
    controls.light_fade.rainbow_falloff_power = light_fade::RAINBOW_FALLOFF_POWER.normalize(
        controls.light_fade.rainbow_falloff_power,
        defaults.light_fade.rainbow_falloff_power,
    );
    controls.light_mode.wall.normal_strength = light_wall::NORMAL_STRENGTH.normalize(
        controls.light_mode.wall.normal_strength,
        defaults.light_mode.wall.normal_strength,
    );
    controls.light_mode.wall.lightmap_gamma = light_wall::LIGHTMAP_GAMMA.normalize(
        controls.light_mode.wall.lightmap_gamma,
        defaults.light_mode.wall.lightmap_gamma,
    );
    controls.light_mode.wall.shadow_contrast = light_wall::SHADOW_CONTRAST.normalize(
        controls.light_mode.wall.shadow_contrast,
        defaults.light_mode.wall.shadow_contrast,
    );
    controls.light_mode.wall.shadow_pivot = light_wall::SHADOW_PIVOT.normalize(
        controls.light_mode.wall.shadow_pivot,
        defaults.light_mode.wall.shadow_pivot,
    );
    controls.light_mode.wall.shadow_floor = light_wall::SHADOW_FLOOR.normalize(
        controls.light_mode.wall.shadow_floor,
        defaults.light_mode.wall.shadow_floor,
    );
    controls.light_mode.wall.highlight_exposure = light_wall::HIGHLIGHT_EXPOSURE.normalize(
        controls.light_mode.wall.highlight_exposure,
        defaults.light_mode.wall.highlight_exposure,
    );
    controls.light_mode.wall.ambient_fill = light_wall::AMBIENT_FILL.normalize(
        controls.light_mode.wall.ambient_fill,
        defaults.light_mode.wall.ambient_fill,
    );
    controls.light_mode.caustic.strength = light_caustic::STRENGTH.normalize(
        controls.light_mode.caustic.strength,
        defaults.light_mode.caustic.strength,
    );
    controls.light_mode.caustic.coverage = light_caustic::COVERAGE.normalize(
        controls.light_mode.caustic.coverage,
        defaults.light_mode.caustic.coverage,
    );
    controls.light_mode.caustic.normal_influence = light_caustic::NORMAL_INFLUENCE.normalize(
        controls.light_mode.caustic.normal_influence,
        defaults.light_mode.caustic.normal_influence,
    );
    controls.light_mode.caustic.normal_elevation = light_caustic::NORMAL_ELEVATION.normalize(
        controls.light_mode.caustic.normal_elevation,
        defaults.light_mode.caustic.normal_elevation,
    );
    controls.light_mode.output.exposure = light_output::EXPOSURE.normalize(
        controls.light_mode.output.exposure,
        defaults.light_mode.output.exposure,
    );
    controls.glass.transmission = ByTheme {
        dark: normalize_transmission(
            controls.glass.transmission.dark,
            &defaults.glass.transmission.dark,
        ),
        light: normalize_transmission(
            controls.glass.transmission.light,
            &defaults.glass.transmission.light,
        ),
    };
    controls.glass.reflection = ByTheme {
        dark: normalize_reflection(
            controls.glass.reflection.dark,
            &defaults.glass.reflection.dark,
        ),
        light: normalize_reflection(
            controls.glass.reflection.light,
            &defaults.glass.reflection.light,
        ),
    };
    controls.postprocess.bloom_strength = finite(
        controls.postprocess.bloom_strength,
        defaults.postprocess.bloom_strength,
    );
    controls.postprocess.bloom_threshold = finite(
        controls.postprocess.bloom_threshold,
        defaults.postprocess.bloom_threshold,
    );
    controls.postprocess.bloom_radius = finite(
        controls.postprocess.bloom_radius,
        defaults.postprocess.bloom_radius,
    );
    controls
}
