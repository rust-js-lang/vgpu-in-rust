//! Gaussian bloom kernels and pyramid sizing from the original dark graph.
use crate::{clamp, number_max, number_min};

pub const BLOOM_LEVELS: usize = 4;
pub const BLOOM_VISIBLE_LEVELS: usize = 3;
pub const BLOOM_KERNEL_TAPS: [usize; 4] = [6, 10, 14, 18];
pub const BLOOM_LEVEL_DIVISORS: [f64; 4] = [2.0, 4.0, 8.0, 16.0];
pub const BLOOM_LEVEL_FACTORS: [f64; 3] = [1.0, 0.8, 0.55];
pub const BLOOM_PAIRED_KERNEL_CAPACITY: usize = 12;

/// Symmetric, energy-normalized half-kernel padded for the WGSL layout.
/// The shader samples zero once and every remaining coefficient twice.
pub fn bloom_kernel_weights(tap_count: f64, capacity: usize) -> Vec<f64> {
    let count = number_min(capacity as f64, number_max(1.0, tap_count.floor()));
    let sigma = count / 3.0;
    let weights: Vec<f64> = (0..capacity)
        .map(|index| {
            if (index as f64) < count {
                (-0.5 * index as f64 * index as f64 / (sigma * sigma)).exp()
            } else {
                0.0
            }
        })
        .collect();
    let tail: f64 = weights
        .iter()
        .skip(1)
        .take((count as usize).saturating_sub(1))
        .sum();
    let total = weights.first().copied().unwrap_or(f64::NAN) + 2.0 * tail;
    weights.into_iter().map(|weight| weight / total).collect()
}

pub struct BloomPairedKernel {
    pub center_weight: f64,
    pub pair_count: f64,
    pub weights: Vec<f64>,
    pub offsets: Vec<f64>,
}

/// Adjacent positive taps reconstruct the discrete kernel with linear sampling.
pub fn bloom_paired_kernel(
    weights: &[f64],
    tap_count: f64,
    capacity: usize,
) -> Result<BloomPairedKernel, String> {
    let count = number_min(weights.len() as f64, number_max(1.0, tap_count.floor()));
    let pair_count = ((count - 1.0) / 2.0).ceil();
    if pair_count > capacity as f64 {
        return Err(format!("Bloom paired kernel needs {pair_count} slots."));
    }
    let mut paired_weights = vec![0.0; capacity];
    let mut paired_offsets = vec![0.0; capacity];
    for pair in 0..pair_count as usize {
        let first = 1 + pair * 2;
        let second = first + 1;
        let first_weight = weights.get(first).copied().unwrap_or(0.0);
        let second_weight = if (second as f64) < count {
            weights.get(second).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let combined = first_weight + second_weight;
        paired_weights[pair] = combined;
        paired_offsets[pair] = if combined > 0.0 {
            (first as f64 * first_weight + second as f64 * second_weight) / combined
        } else {
            first as f64
        };
    }
    Ok(BloomPairedKernel {
        center_weight: weights.first().copied().unwrap_or(0.0),
        pair_count,
        weights: paired_weights,
        offsets: paired_offsets,
    })
}

pub fn bloom_spread(radius: f64, minimum: f64, maximum: f64) -> f64 {
    if !radius.is_finite() || maximum.partial_cmp(&minimum) != Some(std::cmp::Ordering::Greater) {
        return 0.0;
    }
    clamp((radius - minimum) / (maximum - minimum), 0.0, 1.0)
}

pub fn bloom_level_size(size: [f64; 2], level: usize) -> [f64; 2] {
    let divisor = BLOOM_LEVEL_DIVISORS.get(level).copied().unwrap_or(16.0);
    [
        number_max(1.0, (size[0] / divisor).ceil()),
        number_max(1.0, (size[1] / divisor).ceil()),
    ]
}

pub fn bloom_uses_pairs(level: usize, horizontal: bool) -> bool {
    level < BLOOM_LEVELS && (!horizontal || level == 0 || level == 3)
}

pub struct RawBlurUniforms {
    pub direction: [f64; 2],
    pub texel_size: [f64; 2],
    pub tap_count: f64,
    pub coefficients0: [f64; 4],
    pub coefficients1: [f64; 4],
    pub coefficients2: [f64; 4],
    pub coefficients3: [f64; 4],
    pub coefficients4: [f64; 4],
    pub coefficients5: [f64; 4],
}
pub struct PairedBlurUniforms {
    pub direction: [f64; 2],
    pub texel_size: [f64; 2],
    pub kernel: [f64; 4],
    pub weights0: [f64; 4],
    pub weights1: [f64; 4],
    pub weights2: [f64; 4],
    pub offsets0: [f64; 4],
    pub offsets1: [f64; 4],
    pub offsets2: [f64; 4],
}
#[cfg_attr(rust_js, rust_js::untagged)]
pub enum BlurUniforms {
    Raw(RawBlurUniforms),
    Paired(PairedBlurUniforms),
}

pub fn bloom_blur_uniforms(level: usize, horizontal: bool, size: [f64; 2]) -> BlurUniforms {
    let direction = if horizontal { [1.0, 0.0] } else { [0.0, 1.0] };
    let texel_size = [1.0 / size[0], 1.0 / size[1]];
    let taps = BLOOM_KERNEL_TAPS[level] as f64;
    let weights = bloom_kernel_weights(taps, 24);
    if bloom_uses_pairs(level, horizontal) {
        let paired = bloom_paired_kernel(&weights, taps, BLOOM_PAIRED_KERNEL_CAPACITY)
            .expect("fixed bloom kernel fits");
        BlurUniforms::Paired(PairedBlurUniforms {
            direction,
            texel_size,
            kernel: [paired.center_weight, paired.pair_count, 0.0, 0.0],
            weights0: paired.weights[0..4].try_into().unwrap(),
            weights1: paired.weights[4..8].try_into().unwrap(),
            weights2: paired.weights[8..12].try_into().unwrap(),
            offsets0: paired.offsets[0..4].try_into().unwrap(),
            offsets1: paired.offsets[4..8].try_into().unwrap(),
            offsets2: paired.offsets[8..12].try_into().unwrap(),
        })
    } else {
        BlurUniforms::Raw(RawBlurUniforms {
            direction,
            texel_size,
            tap_count: taps,
            coefficients0: weights[0..4].try_into().unwrap(),
            coefficients1: weights[4..8].try_into().unwrap(),
            coefficients2: weights[8..12].try_into().unwrap(),
            coefficients3: weights[12..16].try_into().unwrap(),
            coefficients4: weights[16..20].try_into().unwrap(),
            coefficients5: weights[20..24].try_into().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kernels_preserve_energy_and_pairs_reconstruct_discrete_moments() {
        for taps in BLOOM_KERNEL_TAPS {
            let weights = bloom_kernel_weights(taps as f64, 24);
            assert!((weights[0] + 2.0 * weights[1..].iter().sum::<f64>() - 1.0).abs() < 1e-15);
            let pairs = bloom_paired_kernel(&weights, taps as f64, 12).unwrap();
            for (pair, (&weight, &offset)) in pairs
                .weights
                .iter()
                .zip(&pairs.offsets)
                .enumerate()
                .take(pairs.pair_count as usize)
            {
                let first = 1 + 2 * pair;
                let second = first + 1;
                let second_weight = if second < taps { weights[second] } else { 0.0 };
                assert_eq!(weight, weights[first] + second_weight);
                assert!(
                    (weight * offset
                        - (first as f64 * weights[first] + second as f64 * second_weight))
                        .abs()
                        < 1e-15
                );
            }
        }
    }
    #[test]
    fn pyramid_clamps_and_capacity_is_checked() {
        assert_eq!(bloom_level_size([101.0, 31.0], 0), [51.0, 16.0]);
        assert_eq!(bloom_level_size([1.0, 1.0], 9), [1.0, 1.0]);
        assert!(bloom_paired_kernel(&[1.0; 24], 24.0, 11).is_err());
        assert_eq!(bloom_spread(f64::NAN, 0.25, 3.0), 0.0);
        assert_eq!(bloom_spread(3.0, 0.25, 3.0), 1.0);
    }
}
