//! Runtime control schema and defaults from prism-background/types.ts.
use crate::types::{CAMERA_FOV_DEGREES, DispersionPreset, PRISM_DEFAULT_BEAM_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrismTheme {
    #[cfg_attr(rust_js, rust_js::name = "dark")]
    Dark,
    #[cfg_attr(rust_js, rust_js::name = "light")]
    Light,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrismView {
    #[cfg_attr(rust_js, rust_js::name = "glass")]
    Glass,
    #[cfg_attr(rust_js, rust_js::name = "back")]
    Back,
    #[cfg_attr(rust_js, rust_js::name = "wall")]
    Wall,
    #[cfg_attr(rust_js, rust_js::name = "caustic")]
    Caustic,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrismDispersion {
    #[cfg_attr(rust_js, rust_js::name = "stylized")]
    Stylized,
    #[cfg_attr(rust_js, rust_js::name = "crown")]
    Crown,
    #[cfg_attr(rust_js, rust_js::name = "flint")]
    Flint,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightToneMapping {
    #[cfg_attr(rust_js, rust_js::name = "aces")]
    Aces,
    #[cfg_attr(rust_js, rust_js::name = "neutral")]
    Neutral,
    #[cfg_attr(rust_js, rust_js::name = "reinhard")]
    Reinhard,
    #[cfg_attr(rust_js, rust_js::name = "clamp")]
    Clamp,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ByTheme<T> {
    pub dark: T,
    pub light: T,
}
impl<T> ByTheme<T> {
    pub fn get(&self, theme: PrismTheme) -> &T {
        match theme {
            PrismTheme::Dark => &self.dark,
            PrismTheme::Light => &self.light,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct GlassTransmissionControls {
    /// Index of refraction used by both rasterized glass interfaces.
    pub ior: f64,
    /// Beer-Lambert absorption per scene unit, in linear RGB.
    pub absorption: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct GlassReflectionControls {
    /// Multiplier on the studio environment before it is reflected.
    pub reflection_strength: f64,
    /// Exposure applied to the studio environment before material response.
    pub environment_exposure: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GlassControls {
    pub transmission: ByTheme<GlassTransmissionControls>,
    pub reflection: ByTheme<GlassReflectionControls>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PostprocessControls {
    pub bloom_strength: f64,
    pub bloom_threshold: f64,
    pub bloom_radius: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LightFadeControls {
    pub beam_opacity: f64,
    pub edge_falloff: f64,
    pub rainbow_falloff_rate: f64,
    pub rainbow_falloff_power: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LightWallControls {
    pub normal_strength: f64,
    pub lightmap_gamma: f64,
    pub shadow_contrast: f64,
    pub shadow_pivot: f64,
    pub shadow_floor: f64,
    pub highlight_exposure: f64,
    pub ambient_fill: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LightCausticControls {
    pub strength: f64,
    pub coverage: f64,
    pub normal_influence: f64,
    pub normal_elevation: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LightOutputControls {
    pub exposure: f64,
    pub tone_mapping: LightToneMapping,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LightModeControls {
    pub wall: LightWallControls,
    pub caustic: LightCausticControls,
    pub output: LightOutputControls,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeamMouseYControls {
    pub top: f64,
    pub bottom: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PrismControls {
    pub dispersion: PrismDispersion,
    pub spectral_dispersion: Option<DispersionPreset>,
    pub view: PrismView,
    pub camera_fov: f64,
    pub beam_width: f64,
    pub beam_mouse_y: BeamMouseYControls,
    pub light_fade: LightFadeControls,
    pub light_mode: LightModeControls,
    pub wall_color: String,
    pub wireframe: bool,
    pub light_wireframe: bool,
    pub environment_debug: bool,
    pub glass: GlassControls,
    pub postprocess: PostprocessControls,
}
impl Default for PrismControls {
    fn default() -> Self {
        Self {
            dispersion: PrismDispersion::Stylized,
            spectral_dispersion: Some(DispersionPreset {
                base: 1.2,
                strength: 0.1,
            }),
            view: PrismView::Glass,
            camera_fov: CAMERA_FOV_DEGREES,
            beam_width: PRISM_DEFAULT_BEAM_WIDTH,
            beam_mouse_y: BeamMouseYControls {
                top: -35.0,
                bottom: 75.0,
            },
            light_fade: LightFadeControls {
                beam_opacity: 1.0,
                edge_falloff: 16.0,
                rainbow_falloff_rate: 3.8,
                rainbow_falloff_power: 3.7,
            },
            light_mode: LightModeControls {
                wall: LightWallControls {
                    normal_strength: 0.6,
                    lightmap_gamma: 0.65,
                    shadow_contrast: 6.85,
                    shadow_pivot: 0.9,
                    shadow_floor: 0.87,
                    highlight_exposure: 3.31,
                    ambient_fill: 0.42,
                },
                caustic: LightCausticControls {
                    strength: 1.9,
                    coverage: 0.86,
                    normal_influence: 1.0,
                    normal_elevation: 35.0,
                },
                output: LightOutputControls {
                    exposure: 1.0,
                    tone_mapping: LightToneMapping::Aces,
                },
            },
            wall_color: "#000000".into(),
            wireframe: false,
            light_wireframe: false,
            environment_debug: false,
            glass: GlassControls {
                transmission: ByTheme {
                    dark: GlassTransmissionControls {
                        ior: 1.645,
                        absorption: [1.0, 1.0, 0.54],
                    },
                    light: GlassTransmissionControls {
                        ior: 1.645,
                        absorption: [0.0; 3],
                    },
                },
                reflection: ByTheme {
                    dark: GlassReflectionControls {
                        reflection_strength: 2.14,
                        environment_exposure: 2.3,
                    },
                    light: GlassReflectionControls {
                        reflection_strength: 3.0,
                        environment_exposure: 4.0,
                    },
                },
            },
            postprocess: PostprocessControls {
                bloom_strength: 0.7,
                bloom_threshold: 0.1,
                bloom_radius: 0.25,
            },
        }
    }
}

pub fn default_prism_controls() -> PrismControls {
    PrismControls::default()
}
