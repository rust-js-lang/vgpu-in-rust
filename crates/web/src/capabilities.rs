//! Optional prism device features and the live bloom storage decision.
use react::webapi::GPUSupportedFeatures;
use vgpu_prism::bloom::BLOOM_VISIBLE_LEVELS;
pub const PACKED_BLOOM_FEATURE: &str = "rg11b10ufloat-renderable";
pub const PACKED_BLOOM_FORMAT: &str = "rg11b10ufloat";
pub const FALLBACK_BLOOM_FORMAT: &str = "rgba16float";
pub fn optional_prism_features(
    supported: &GPUSupportedFeatures,
    performance_sampling: bool,
) -> Vec<String> {
    let mut features = Vec::new();
    if supported.has(PACKED_BLOOM_FEATURE) {
        features.push(PACKED_BLOOM_FEATURE.into());
    }
    if performance_sampling && supported.has("timestamp-query") {
        features.push("timestamp-query".into());
    }
    features
}
pub fn bloom_format_for_level(enabled: &GPUSupportedFeatures, level: usize) -> &'static str {
    if level < BLOOM_VISIBLE_LEVELS && enabled.has(PACKED_BLOOM_FEATURE) {
        PACKED_BLOOM_FORMAT
    } else {
        FALLBACK_BLOOM_FORMAT
    }
}
