//! Control ranges from prism-background/types.ts. One range owner for validation and GUI.
use crate::clamp;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRange {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}
impl ControlRange {
    pub fn normalize(self, value: f64, fallback: f64) -> f64 {
        clamp(
            if value.is_finite() { value } else { fallback },
            self.min,
            self.max,
        )
    }
}
pub mod beam_mouse_y {
    use super::ControlRange;
    pub const TOP: ControlRange = ControlRange {
        min: -85.0,
        max: 85.0,
        step: 1.0,
    };
    pub const BOTTOM: ControlRange = ControlRange {
        min: -85.0,
        max: 85.0,
        step: 1.0,
    };
}
pub mod light_fade {
    use super::ControlRange;
    pub const BEAM_OPACITY: ControlRange = ControlRange {
        min: 0.0,
        max: 1.0,
        step: 0.01,
    };
    pub const EDGE_FALLOFF: ControlRange = ControlRange {
        min: 0.0,
        max: 16.0,
        step: 0.1,
    };
    pub const RAINBOW_FALLOFF_RATE: ControlRange = ControlRange {
        min: 0.0,
        max: 40.0,
        step: 0.1,
    };
    pub const RAINBOW_FALLOFF_POWER: ControlRange = ControlRange {
        min: 0.25,
        max: 8.0,
        step: 0.05,
    };
}
pub mod light_wall {
    use super::ControlRange;
    pub const NORMAL_STRENGTH: ControlRange = ControlRange {
        min: 0.0,
        max: 3.0,
        step: 0.05,
    };
    pub const LIGHTMAP_GAMMA: ControlRange = ControlRange {
        min: 0.5,
        max: 4.0,
        step: 0.05,
    };
    pub const SHADOW_CONTRAST: ControlRange = ControlRange {
        min: 0.25,
        max: 8.0,
        step: 0.05,
    };
    pub const SHADOW_PIVOT: ControlRange = ControlRange {
        min: 0.05,
        max: 0.95,
        step: 0.01,
    };
    pub const SHADOW_FLOOR: ControlRange = ControlRange {
        min: 0.0,
        max: 1.2,
        step: 0.01,
    };
    pub const HIGHLIGHT_EXPOSURE: ControlRange = ControlRange {
        min: 0.25,
        max: 8.0,
        step: 0.01,
    };
    pub const AMBIENT_FILL: ControlRange = ControlRange {
        min: 0.0,
        max: 2.5,
        step: 0.01,
    };
}
pub mod light_caustic {
    use super::ControlRange;
    pub const STRENGTH: ControlRange = ControlRange {
        min: 0.0,
        max: 4.0,
        step: 0.01,
    };
    pub const COVERAGE: ControlRange = ControlRange {
        min: 0.0,
        max: 1.0,
        step: 0.01,
    };
    pub const NORMAL_INFLUENCE: ControlRange = ControlRange {
        min: 0.0,
        max: 1.0,
        step: 0.01,
    };
    pub const NORMAL_ELEVATION: ControlRange = ControlRange {
        min: 5.0,
        max: 85.0,
        step: 1.0,
    };
}
pub mod light_output {
    use super::ControlRange;
    pub const EXPOSURE: ControlRange = ControlRange {
        min: 0.25,
        max: 2.0,
        step: 0.01,
    };
}
pub mod spectral_dispersion {
    use super::ControlRange;
    pub const BASE: ControlRange = ControlRange {
        min: 1.2,
        max: 2.1,
        step: 0.001,
    };
    pub const STRENGTH: ControlRange = ControlRange {
        min: 0.0,
        max: 0.2,
        step: 0.0005,
    };
}

pub mod camera {
    use super::ControlRange;
    pub const FOV: ControlRange = ControlRange {
        min: 20.0,
        max: 70.0,
        step: 1.0,
    };
}
