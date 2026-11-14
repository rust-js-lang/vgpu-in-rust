//! Display-space reveal and independent opacity/beam timing from the hero.
use crate::{clamp, controls::PrismTheme};
pub struct RevealUniforms {
    pub background_color: [f64; 3],
    pub reveal_progress: f64,
}
pub struct HeroReveal {
    pub opacity: f64,
    pub beam_width: f64,
}
pub fn presentation_reveal_uniforms(mode: PrismTheme, progress: f64) -> RevealUniforms {
    RevealUniforms {
        background_color: match mode {
            PrismTheme::Dark => [0.0; 3],
            PrismTheme::Light => [250.0 / 255.0; 3],
        },
        reveal_progress: clamp(progress, 0.0, 1.0),
    }
}
pub fn hero_reveal_progress(elapsed_seconds: f64) -> HeroReveal {
    let beam_start = 1.0 - (1.0_f64 - 0.25).cbrt();
    let opacity_linear = clamp(elapsed_seconds, 0.0, 1.0);
    let beam_linear = clamp(
        (elapsed_seconds - beam_start) / (2.5 - beam_start),
        0.0,
        1.0,
    );
    HeroReveal {
        opacity: 1.0 - (1.0 - opacity_linear).powi(3),
        beam_width: 1.0 - (1.0 - beam_linear).powi(3),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opacity_finishes_before_the_beam() {
        let start = hero_reveal_progress(0.0);
        assert_eq!(start.opacity, 0.0);
        assert_eq!(start.beam_width, 0.0);
        let middle = hero_reveal_progress(1.0);
        assert_eq!(middle.opacity, 1.0);
        assert!(middle.beam_width > 0.0 && middle.beam_width < 1.0);
        assert_eq!(hero_reveal_progress(2.5).beam_width, 1.0);
        assert!(
            presentation_reveal_uniforms(PrismTheme::Dark, f64::NAN)
                .reveal_progress
                .is_nan()
        );
    }
}
