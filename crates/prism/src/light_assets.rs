//! Portable recovery textures and mip filtering, from light/assets/generate*.ts and mips.ts.
use crate::light_asset_math::{
    clamp01, fbm, segment_distance, smoothstep, triangle_contains, triangle_distance, write_pixel,
};
use crate::optics::wavelength_to_beam_rgb;
use crate::types::{PRISM_CENTROID, PRISM_SIDE, PRISM_TRIANGLE};
use crate::{number_max, number_min};
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightAssetId {
    WallMaterial,
    WallLighting,
    CausticProfile,
}
impl LightAssetId {
    pub fn size(self) -> [usize; 2] {
        match self {
            Self::WallMaterial | Self::WallLighting => [512, 512],
            Self::CausticProfile => [1024, 256],
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::WallMaterial => "wall-material",
            Self::WallLighting => "wall-lighting",
            Self::CausticProfile => "caustic-profile",
        }
    }
}
pub const LIGHT_ASSET_IDS: [LightAssetId; 3] = [
    LightAssetId::WallMaterial,
    LightAssetId::WallLighting,
    LightAssetId::CausticProfile,
];
#[derive(Clone)]
pub struct GeneratedLightAsset {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}
pub fn generate_light_asset(id: LightAssetId) -> GeneratedLightAsset {
    match id {
        LightAssetId::WallMaterial => generate_wall_material(id.size()),
        LightAssetId::WallLighting => generate_wall_lighting(id.size()),
        LightAssetId::CausticProfile => generate_caustic_profile(id.size()),
    }
}
pub static PRISM_GROUNDING_TRIANGLE: LazyLock<[[f64; 2]; 3]> = LazyLock::new(|| {
    [PRISM_TRIANGLE.a, PRISM_TRIANGLE.b, PRISM_TRIANGLE.c].map(|point| {
        [
            (point[0] - PRISM_CENTROID[0]) / PRISM_SIDE,
            (PRISM_CENTROID[1] - point[1]) / PRISM_SIDE,
        ]
    })
});
pub const GROUNDING_AO_INSIDE_SPREAD: f64 = 0.0015;
pub const GROUNDING_AO_OUTSIDE_SPREAD: f64 = 0.014;
pub const GROUNDING_AO_OPACITY: f64 = 0.075;
fn plaster_height(u: f64, v: f64) -> f64 {
    fbm(u * 34.0, v * 34.0, 5) * 0.65 + fbm(u * 117.0, v * 117.0, 2) * 0.35
}
pub fn generate_wall_material([width, height]: [usize; 2]) -> GeneratedLightAsset {
    let mut pixels = vec![0; width * height * 4];
    let epsilon = 1.0 / width.max(height) as f64;
    for y in 0..height {
        for x in 0..width {
            let u = (x as f64 + 0.5) / width as f64;
            let v = (y as f64 + 0.5) / height as f64;
            let height_x = plaster_height(u + epsilon, v) - plaster_height(u - epsilon, v);
            let height_y = plaster_height(u, v + epsilon) - plaster_height(u, v - epsilon);
            let variation = plaster_height(u, v) - 0.5;
            write_pixel(
                &mut pixels,
                (y * width + x) * 4,
                [
                    0.8 + variation * 0.06,
                    0.5 - height_x * 1.8,
                    0.5 - height_y * 1.8,
                    0.86 + variation * 0.12,
                ],
            );
        }
    }
    GeneratedLightAsset {
        width,
        height,
        pixels,
    }
}
fn soft_blob(u: f64, v: f64, center_x: f64, center_y: f64, radius_x: f64, radius_y: f64) -> f64 {
    let dx = (u - center_x) / radius_x;
    let dy = (v - center_y) / radius_y;
    (-(dx * dx + dy * dy) * 1.8).exp()
}
fn overhead_light(u: f64, v: f64) -> f64 {
    let upper_left = soft_blob(u, v, -0.08, -0.08, 0.58, 0.64) * 0.75;
    let center = soft_blob(u, v, 0.42, 0.22, 0.15, 0.17) * 0.7;
    let right = soft_blob(u, v, 0.83, 0.3, 0.16, 0.18) * 0.6;
    clamp01(upper_left + center + right)
}
fn grounding(x: f64, y: f64) -> [f64; 2] {
    let [_, left, right] = *PRISM_GROUNDING_TRIANGLE;
    let base = segment_distance([x, y], left, right);
    let base_contact = (-(base * base) / 0.00135).exp();
    let edge = triangle_distance([x, y], &*PRISM_GROUNDING_TRIANGLE);
    let spread = if triangle_contains([x, y], &*PRISM_GROUNDING_TRIANGLE) {
        GROUNDING_AO_INSIDE_SPREAD
    } else {
        GROUNDING_AO_OUTSIDE_SPREAD
    };
    let edge_occlusion = (-(edge * edge) / spread).exp();
    [
        clamp01(1.0 - base_contact * 0.15),
        clamp01(1.0 - edge_occlusion * GROUNDING_AO_OPACITY - base_contact * 0.25),
    ]
}
pub fn generate_wall_lighting([width, height]: [usize; 2]) -> GeneratedLightAsset {
    let mut pixels = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let u = (x as f64 + 0.5) / width as f64;
            let v = (y as f64 + 0.5) / height as f64;
            let [prism_shadow, ao] = grounding(u * 2.0 - 1.0, v * 2.0 - 1.0);
            write_pixel(
                &mut pixels,
                (y * width + x) * 4,
                [overhead_light(u, v), prism_shadow, ao, 1.0],
            );
        }
    }
    GeneratedLightAsset {
        width,
        height,
        pixels,
    }
}
pub fn generate_caustic_profile([width, height]: [usize; 2]) -> GeneratedLightAsset {
    let mut pixels = vec![0; width * height * 4];
    for y in 0..height {
        let wavelength = 700.0 - y as f64 / height.saturating_sub(1).max(1) as f64 * 300.0;
        let spectral = wavelength_to_beam_rgb(wavelength);
        for x in 0..width {
            let travel = x as f64 / width.saturating_sub(1).max(1) as f64;
            let coarse = fbm(travel * 18.0, wavelength * 0.018, 4);
            let filament = 0.5 + 0.5 * (travel * 104.0 + wavelength * 0.071).sin();
            let focus = 0.72 + coarse * 0.24 + filament * 0.04;
            let tail = 1.0 - smoothstep(0.58, 1.08, travel) * 0.44;
            let far_neutral = smoothstep(0.2, 0.88, travel) * 0.36;
            let peak = number_max(
                number_max(number_max(spectral[0], spectral[1]), spectral[2]),
                1e-5,
            );
            let rgb = spectral.map(|channel| {
                clamp01((channel / peak * (1.0 - far_neutral) + far_neutral) * focus * tail)
            });
            write_pixel(
                &mut pixels,
                (y * width + x) * 4,
                [rgb[0], rgb[1], rgb[2], focus * tail],
            );
        }
    }
    GeneratedLightAsset {
        width,
        height,
        pixels,
    }
}
pub fn generate_mip_chain(base: GeneratedLightAsset) -> Vec<GeneratedLightAsset> {
    let mut levels = vec![base];
    loop {
        let current = levels.last().expect("base level");
        if current.width <= 1 && current.height <= 1 {
            break;
        }
        let width = (current.width >> 1).max(1);
        let height = (current.height >> 1).max(1);
        let mut pixels = vec![0; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                for channel in 0..4 {
                    let mut sum = 0.0;
                    for oy in 0..2 {
                        for ox in 0..2 {
                            let source_x = current.width.saturating_sub(1).min(x * 2 + ox);
                            let source_y = current.height.saturating_sub(1).min(y * 2 + oy);
                            sum += if current.width == 0 || current.height == 0 {
                                f64::NAN
                            } else {
                                current
                                    .pixels
                                    .get((source_y * current.width + source_x) * 4 + channel)
                                    .copied()
                                    .map(f64::from)
                                    .unwrap_or(f64::NAN)
                            };
                        }
                    }
                    pixels[(y * width + x) * 4 + channel] = (sum / 4.0).round() as u8;
                }
            }
        }
        levels.push(GeneratedLightAsset {
            width,
            height,
            pixels,
        });
    }
    levels
}
pub fn apply_global_light_mask(
    asset: &mut GeneratedLightAsset,
    source: &[u8],
    source_width: usize,
    source_height: usize,
) {
    let sample = |x: usize, y: usize| {
        if source_width == 0 || source_height == 0 {
            return f64::NAN;
        }
        source
            .get((y * source_width + x) * 4)
            .map(|&v| f64::from(v))
            .unwrap_or(f64::NAN)
    };
    for y in 0..asset.height {
        let source_y = (y as f64 + 0.5) / asset.height as f64 * source_height as f64 - 0.5;
        let y0 = number_max(0.0, source_y.floor()) as usize;
        let y1 = source_height.saturating_sub(1).min(y0 + 1);
        let mix_y = source_y - source_y.floor();
        for x in 0..asset.width {
            let source_x = (x as f64 + 0.5) / asset.width as f64 * source_width as f64 - 0.5;
            let x0 = number_max(0.0, source_x.floor()) as usize;
            let x1 = source_width.saturating_sub(1).min(x0 + 1);
            let mix_x = source_x - source_x.floor();
            let top = sample(x0, y0) * (1.0 - mix_x) + sample(x1, y0) * mix_x;
            let bottom = sample(x0, y1) * (1.0 - mix_x) + sample(x1, y1) * mix_x;
            let u = (x as f64 + 0.5) / asset.width as f64;
            let v = (y as f64 + 0.5) / asset.height as f64;
            let distance = number_min(number_min(number_min(u, 1.0 - u), v), 1.0 - v);
            let unit = clamp01(distance / 0.06);
            let fade = unit * unit * (3.0 - 2.0 * unit);
            asset.pixels[(y * asset.width + x) * 4] =
                ((top * (1.0 - mix_y) + bottom * mix_y) * fade).round() as u8;
        }
    }
}
pub fn global_light_mask_edge_max(pixels: &[u8], width: usize, height: usize, inset: f64) -> f64 {
    let mut result = 0.0;
    for y in 0..height {
        for x in 0..width {
            if x as f64 >= inset
                && (x as f64) < width as f64 - inset
                && y as f64 >= inset
                && (y as f64) < height as f64 - inset
            {
                continue;
            }
            result = number_max(
                result,
                pixels
                    .get((y * width + x) * 4)
                    .copied()
                    .map(f64::from)
                    .unwrap_or(f64::NAN),
            );
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_mips_duplicate_single_pixel_axes_and_round_half_up() {
        let asset = GeneratedLightAsset {
            width: 1,
            height: 2,
            pixels: vec![0, 1, 2, 255, 1, 2, 3, 255],
        };
        let levels = generate_mip_chain(asset);
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[1].pixels, vec![1, 2, 3, 255]);
        for id in LIGHT_ASSET_IDS {
            let asset = match id {
                LightAssetId::WallMaterial => generate_wall_material([8, 8]),
                LightAssetId::WallLighting => generate_wall_lighting([8, 8]),
                LightAssetId::CausticProfile => generate_caustic_profile([8, 8]),
            };
            assert_eq!(asset.pixels.len(), 8 * 8 * 4);
            assert_eq!(generate_mip_chain(asset).len(), 4);
        }
    }
}
