//! Active-frame quality policy, ported from performance/frame-health.ts.
const DEFAULT_REFRESH_FPS: f64 = 60.0;
const MAX_INTERACTIVE_FPS: f64 = 90.0;
const AMBIENT_FPS: f64 = 30.0;
const HEALTH_WINDOW_MS: f64 = 2_000.0;
const HEALTH_RATIO: f64 = 0.8;
const REFRESH_SAMPLE_COUNT: usize = 20;
const MAX_REFRESH_SAMPLES: usize = 60;
const INACTIVE_GAP_MS: f64 = 250.0;

#[derive(Clone, Copy, PartialEq)]
pub enum FrameWorkload {
    #[cfg_attr(rust_js, rust_js::name = "interactive")]
    Interactive,
    #[cfg_attr(rust_js, rust_js::name = "dust")]
    Dust,
}
#[derive(Clone, Copy)]
pub struct FrameHealthSample {
    /// Raw requestAnimationFrame interval, including frames skipped by our cap.
    pub delta_ms: f64,
    pub active: bool,
    pub rendered: bool,
    pub mobile: bool,
    pub workload: FrameWorkload,
}
#[derive(Clone, Copy)]
pub struct FrameHealthStatus {
    pub downgrade: bool,
    pub estimated_refresh_fps: f64,
    pub target_fps: f64,
    pub threshold_fps: f64,
    pub observed_fps: Option<f64>,
    pub active_window_ms: f64,
}
struct TimedFrame {
    duration_ms: f64,
    rendered: bool,
}
/// Measures presented production frames over active rAF time. Intentional caps
/// are evaluated against their workload target rather than the callback rate.
pub struct FrameHealthMonitor {
    refresh_fps: f64,
    refresh_samples: Vec<f64>,
    active_frames: Vec<TimedFrame>,
    active_duration_ms: f64,
    active_rendered_frames: u32,
    active_target_fps: Option<f64>,
    downgrade: bool,
}
impl Default for FrameHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
impl FrameHealthMonitor {
    pub fn new() -> Self {
        Self {
            refresh_fps: DEFAULT_REFRESH_FPS,
            refresh_samples: Vec::new(),
            active_frames: Vec::new(),
            active_duration_ms: 0.0,
            active_rendered_frames: 0,
            active_target_fps: None,
            downgrade: false,
        }
    }
    fn reset_active_window(&mut self) {
        self.active_frames.clear();
        self.active_duration_ms = 0.0;
        self.active_rendered_frames = 0;
        self.active_target_fps = None;
    }
    pub fn reset(&mut self) {
        self.refresh_fps = DEFAULT_REFRESH_FPS;
        self.refresh_samples.clear();
        self.reset_active_window();
    }
    pub fn record(&mut self, sample: FrameHealthSample) -> FrameHealthStatus {
        if self.downgrade {
            return status(
                true,
                self.refresh_fps,
                self.active_target_fps.unwrap_or(60.0),
                self.active_rendered_frames,
                self.active_duration_ms,
            );
        }
        if !sample.active
            || !sample.delta_ms.is_finite()
            || sample.delta_ms <= 0.0
            || sample.delta_ms > INACTIVE_GAP_MS
        {
            self.reset_active_window();
            return status(
                false,
                self.refresh_fps,
                target_fps(sample, self.refresh_fps),
                0,
                0.0,
            );
        }
        let previous_refresh = self.refresh_fps;
        self.refresh_fps =
            updated_refresh_fps(self.refresh_fps, &self.refresh_samples, sample.delta_ms);
        self.refresh_samples.push(sample.delta_ms);
        if self.refresh_samples.len() > MAX_REFRESH_SAMPLES {
            self.refresh_samples.remove(0);
        }
        let target = target_fps(sample, self.refresh_fps);
        if let Some(active_target) = self.active_target_fps
            && ((active_target - target).abs() > 0.5
                || (previous_refresh - self.refresh_fps).abs() > 0.5)
        {
            self.reset_active_window();
        }
        self.active_target_fps = Some(target);
        self.active_frames.push(TimedFrame {
            duration_ms: sample.delta_ms,
            rendered: sample.rendered,
        });
        self.active_duration_ms += sample.delta_ms;
        if sample.rendered {
            self.active_rendered_frames += 1;
        }
        while self.active_frames.len() > 1
            && self.active_duration_ms - self.active_frames[0].duration_ms >= HEALTH_WINDOW_MS
        {
            let removed = self.active_frames.remove(0);
            self.active_duration_ms -= removed.duration_ms;
            if removed.rendered {
                self.active_rendered_frames -= 1;
            }
        }
        let observed_fps =
            f64::from(self.active_rendered_frames) / (self.active_duration_ms / 1_000.0);
        if self.active_duration_ms >= HEALTH_WINDOW_MS && observed_fps < target * HEALTH_RATIO {
            self.downgrade = true;
        }
        status(
            self.downgrade,
            self.refresh_fps,
            target,
            self.active_rendered_frames,
            self.active_duration_ms,
        )
    }
}
fn target_fps(sample: FrameHealthSample, refresh_fps: f64) -> f64 {
    if sample.mobile || sample.workload == FrameWorkload::Dust {
        AMBIENT_FPS
    } else {
        refresh_fps.min(MAX_INTERACTIVE_FPS)
    }
}
fn updated_refresh_fps(current: f64, samples: &[f64], delta_ms: f64) -> f64 {
    if !(4.0..=50.0).contains(&delta_ms) {
        return current;
    }
    let mut next = samples.to_vec();
    next.push(delta_ms);
    let start = next.len().saturating_sub(REFRESH_SAMPLE_COUNT);
    let mut sorted = next[start..].to_vec();
    if sorted.len() < REFRESH_SAMPLE_COUNT {
        return current;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let lower = sorted[(sorted.len() as f64 * 0.2).floor() as usize];
    let upper = sorted[(sorted.len() as f64 * 0.8).floor() as usize];
    // Accept faster refresh only after a stable sample window. Never lower the
    // seed, so a 60 Hz display cannot accidentally target 90.
    if (upper - lower) / median > 0.12 {
        return current;
    }
    let candidate = 1_000.0 / median;
    if candidate > current + 4.0 {
        candidate
    } else {
        current
    }
}
fn status(
    downgrade: bool,
    estimated_refresh_fps: f64,
    target_fps: f64,
    rendered_frames: u32,
    active_window_ms: f64,
) -> FrameHealthStatus {
    FrameHealthStatus {
        downgrade,
        estimated_refresh_fps,
        target_fps,
        threshold_fps: target_fps * HEALTH_RATIO,
        observed_fps: if active_window_ms > 0.0 {
            Some(f64::from(rendered_frames) / (active_window_ms / 1_000.0))
        } else {
            None
        },
        active_window_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(rendered: bool) -> FrameHealthSample {
        FrameHealthSample {
            delta_ms: 1_000.0 / 60.0,
            active: true,
            rendered,
            mobile: false,
            workload: FrameWorkload::Interactive,
        }
    }
    #[test]
    fn unhealthy_window_is_active_and_downgrade_stays_latched_after_reset() {
        let mut monitor = FrameHealthMonitor::new();
        for _ in 0..119 {
            assert!(!monitor.record(sample(false)).downgrade);
        }
        assert!(monitor.record(sample(false)).downgrade);
        monitor.reset();
        let result = monitor.record(sample(true));
        assert!(result.downgrade);
        assert_eq!(result.active_window_ms, 0.0);
        assert_eq!(result.observed_fps, None);
    }
    #[test]
    fn inactivity_and_long_gaps_discard_partial_window() {
        for delta in [0.0, -1.0, f64::NAN, f64::INFINITY, 251.0] {
            let mut monitor = FrameHealthMonitor::new();
            for _ in 0..90 {
                monitor.record(sample(false));
            }
            assert!(
                !monitor
                    .record(FrameHealthSample {
                        delta_ms: delta,
                        ..sample(false)
                    })
                    .downgrade
            );
            for _ in 0..119 {
                assert!(!monitor.record(sample(false)).downgrade);
            }
        }
    }
    #[test]
    fn healthy_caps_and_fast_refresh_do_not_downgrade() {
        for (fps, mobile, workload, stride, target) in [
            (60.0, false, FrameWorkload::Interactive, 1, 60.0),
            (120.0, false, FrameWorkload::Interactive, 4, 90.0),
            (120.0, true, FrameWorkload::Interactive, 4, 30.0),
            (120.0, false, FrameWorkload::Dust, 4, 30.0),
        ] {
            let mut monitor = FrameHealthMonitor::new();
            for i in 0..360 {
                let rendered = if target == 90.0 {
                    i % stride != 0
                } else {
                    i % stride == 0
                };
                let result = monitor.record(FrameHealthSample {
                    delta_ms: 1_000.0 / fps,
                    mobile,
                    workload,
                    ..sample(rendered)
                });
                assert!(!result.downgrade);
                if i > 20 {
                    assert!((result.target_fps - target).abs() < 1e-10);
                }
            }
        }
    }
}
