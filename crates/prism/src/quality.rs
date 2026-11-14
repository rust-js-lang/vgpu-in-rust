//! Scene mesh quality shared by runtime capacity and pipeline activation.
use crate::light_mesh::{LightMeshLayout, light_mesh_layout};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PipelineQuality {
    #[cfg_attr(rust_js, rust_js::name = "high")]
    High,
    #[cfg_attr(rust_js, rust_js::name = "low")]
    Low,
}
impl PipelineQuality {
    pub fn mesh_layout(self) -> LightMeshLayout {
        match self {
            Self::High => light_mesh_layout(128, 24),
            Self::Low => light_mesh_layout(64, 12),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QualityPreference {
    #[cfg_attr(rust_js, rust_js::name = "auto")]
    Auto,
    #[cfg_attr(rust_js, rust_js::name = "high")]
    High,
    #[cfg_attr(rust_js, rust_js::name = "low")]
    Low,
}
impl QualityPreference {
    pub fn initial_quality(self) -> PipelineQuality {
        match self {
            Self::Low => PipelineQuality::Low,
            Self::Auto | Self::High => PipelineQuality::High,
        }
    }
}
