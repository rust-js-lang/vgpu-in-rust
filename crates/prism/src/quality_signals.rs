//! Advisory Auto-quality decisions from performance/auto-quality.ts.
#[derive(Clone, Copy, PartialEq)]
pub enum GpuTierType {
    #[cfg_attr(rust_js, rust_js::name = "SSR")]
    Ssr,
    #[cfg_attr(rust_js, rust_js::name = "BENCHMARK_FETCH_FAILED")]
    BenchmarkFetchFailed,
    #[cfg_attr(rust_js, rust_js::name = "BLOCKLISTED")]
    Blocklisted,
    #[cfg_attr(rust_js, rust_js::name = "BENCHMARK")]
    Benchmark,
    #[cfg_attr(rust_js, rust_js::name = "FALLBACK")]
    Fallback,
    #[cfg_attr(rust_js, rust_js::name = "WEBGL_UNSUPPORTED")]
    WebglUnsupported,
}
#[derive(Clone, Copy)]
pub struct GpuTierSignal {
    pub tier: f64,
    pub r#type: GpuTierType,
    pub is_mobile: Option<bool>,
}
#[derive(Clone, Copy)]
pub struct BatterySignal {
    pub charging: bool,
    pub level: f64,
}
/// Mobile devices, benchmark-backed low tiers, and the blocklist use Low.
pub fn gpu_tier_requests_low(result: GpuTierSignal) -> bool {
    result.is_mobile == Some(true)
        || result.r#type == GpuTierType::Blocklisted
        || (result.r#type == GpuTierType::Benchmark && result.tier <= 1.0)
}
/// Inclusive 30% threshold, ignored while the device is plugged in.
pub fn battery_requests_low(battery: Option<BatterySignal>) -> bool {
    battery.is_some_and(|battery| {
        !battery.charging && battery.level.is_finite() && battery.level <= 0.3
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn advisory_signals_preserve_thresholds_and_optional_values() {
        assert!(!battery_requests_low(None));
        for level in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.300001, 1.0] {
            assert!(!battery_requests_low(Some(BatterySignal {
                charging: false,
                level
            })));
        }
        assert!(battery_requests_low(Some(BatterySignal {
            charging: false,
            level: 0.3
        })));
        assert!(!battery_requests_low(Some(BatterySignal {
            charging: true,
            level: 0.1
        })));
        assert!(gpu_tier_requests_low(GpuTierSignal {
            tier: 1.0,
            r#type: GpuTierType::Benchmark,
            is_mobile: None
        }));
        assert!(!gpu_tier_requests_low(GpuTierSignal {
            tier: 0.0,
            r#type: GpuTierType::Fallback,
            is_mobile: None
        }));
        assert!(gpu_tier_requests_low(GpuTierSignal {
            tier: 3.0,
            r#type: GpuTierType::Fallback,
            is_mobile: Some(true)
        }));
    }
}
