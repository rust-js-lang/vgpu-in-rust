//! Deferred advisory signals from performance/auto-quality.ts.
use crate::canvas_surface::catch_native;
use crate::quality_logging::{QualityLogger, default_quality_logger};
use js::{JsError, Promise};
use react::webapi::{
    BatteryManager, Event, EventTargetExt, Navigator, events, listener, navigator,
};
use std::cell::RefCell;
use std::rc::Rc;
use vgpu_prism::frame_health::{FrameHealthMonitor, FrameHealthSample, FrameWorkload};
use vgpu_prism::quality::QualityReason;
use vgpu_prism::quality_signals::{
    BatterySignal, GpuTierSignal, GpuTierType, battery_requests_low, gpu_tier_requests_low,
};

pub struct GpuTierDetection {
    pub tier: f64,
    pub r#type: GpuTierType,
    pub is_mobile: Option<bool>,
    pub gpu: Option<String>,
    pub fps: Option<f64>,
    pub device: Option<String>,
}
struct GpuTierOptions<'a> {
    #[cfg_attr(rust_js, rust_js::name = "benchmarksURL")]
    benchmarks_url: &'a str,
}
#[cfg_attr(rust_js, rust_js::link_name = "@pmndrs/detect-gpu#getGPUTier")]
fn get_gpu_tier(_options: GpuTierOptions<'_>) -> Promise<GpuTierDetection> {
    unreachable!()
}
pub struct AutoQualityOptions {
    pub on_downgrade: Rc<dyn Fn(QualityReason)>,
    pub navigator: Option<&'static Navigator>,
    pub load_gpu_tier: Option<Rc<dyn Fn() -> Promise<GpuTierDetection>>>,
    pub logger: Option<QualityLogger>,
}
/// Closures retain their shared state; the renderer can call them without
/// statically importing this deferred module.
pub struct AutoQualityController {
    pub record_frame: Rc<dyn Fn(FrameHealthSample)>,
    pub reset_health: Rc<dyn Fn()>,
    pub dispose: Rc<dyn Fn()>,
}
struct AutoState {
    disposed: bool,
    downgraded: bool,
    battery: Option<&'static BatteryManager>,
    battery_listener: Option<&'static dyn Fn(&'static Event)>,
    health: FrameHealthMonitor,
    on_downgrade: Rc<dyn Fn(QualityReason)>,
    logger: QualityLogger,
}
struct GpuLog {
    r#type: GpuTierType,
    tier: f64,
    gpu: Option<String>,
    fps: Option<f64>,
    is_mobile: Option<bool>,
    device: Option<String>,
    decision: &'static str,
}
struct UnavailableLog {
    error: Option<String>,
    reason: Option<&'static str>,
    decision: Option<&'static str>,
}
struct BatteryLog {
    source: String,
    level: f64,
    charging: bool,
    decision: &'static str,
}
struct RuntimeLog {
    workload: FrameWorkload,
    mobile: bool,
    estimated_refresh_fps: f64,
    target_fps: f64,
    threshold_fps: f64,
    observed_fps: Option<f64>,
    active_window_ms: f64,
    decision: &'static str,
}
fn error_message(error: &JsError) -> String {
    if error.is_error() {
        error.message()
    } else {
        js::js_error::to_string(error)
    }
}
fn decision(low: bool) -> &'static str {
    if low { "request-low" } else { "keep-high" }
}
fn request_low(state: &Rc<RefCell<AutoState>>, reason: QualityReason) {
    let callback = {
        let mut state = state.borrow_mut();
        if state.disposed || state.downgraded {
            return;
        }
        state.downgraded = true;
        Rc::clone(&state.on_downgrade)
    };
    callback(reason);
}
fn battery_change(state: &Rc<RefCell<AutoState>>, event: Option<&Event>) {
    let (battery, logger) = {
        let state = state.borrow();
        if state.disposed {
            return;
        }
        (state.battery, Rc::clone(&state.logger))
    };
    let Some(battery) = battery else {
        return;
    };
    let low = battery_requests_low(Some(BatterySignal {
        charging: battery.charging(),
        level: battery.level(),
    }));
    let details = BatteryLog {
        source: event
            .map(|event| event.r#type())
            .unwrap_or_else(|| "initial".into()),
        level: battery.level(),
        charging: battery.charging(),
        decision: decision(low),
    };
    logger("[Prism quality] Battery status.", js::unknown(&details));
    if low {
        request_low(state, QualityReason::Battery);
    }
}
async fn load_tier() -> GpuTierDetection {
    let detect = js::import!(get_gpu_tier).await;
    // Relative to the page so the same benchmark assets work below a Pages base.
    detect(GpuTierOptions {
        benchmarks_url: "./prism-gpu-benchmarks",
    })
    .await
}
pub fn create_auto_quality_controller(options: AutoQualityOptions) -> AutoQualityController {
    let state = Rc::new(RefCell::new(AutoState {
        disposed: false,
        downgraded: false,
        battery: None,
        battery_listener: None,
        health: FrameHealthMonitor::new(),
        on_downgrade: options.on_downgrade,
        logger: options.logger.unwrap_or_else(default_quality_logger),
    }));
    let load = options.load_gpu_tier;
    let detection: Promise<GpuTierDetection> = js::promise::resolve(()).then(move |_| match load {
        Some(load) => load(),
        None => js::promise(load_tier()),
    });
    let detected = Rc::clone(&state);
    let detected_error = Rc::clone(&state);
    let task: Promise<()> = detection.then(move |result| {
        if detected.borrow().disposed {
            return;
        }
        let logger = Rc::clone(&detected.borrow().logger);
        let low = gpu_tier_requests_low(GpuTierSignal {
            tier: result.tier,
            r#type: result.r#type,
            is_mobile: result.is_mobile,
        });
        let details = GpuLog {
            r#type: result.r#type,
            tier: result.tier,
            gpu: result.gpu,
            fps: result.fps,
            is_mobile: result.is_mobile,
            device: result.device,
            decision: decision(low),
        };
        logger("[Prism quality] GPU detected.", js::unknown(&details));
        if low {
            request_low(&detected, QualityReason::GpuTier);
        }
    });
    let _: Promise<()> = task.catch(move |error| {
        if detected_error.borrow().disposed {
            return;
        }
        let logger = Rc::clone(&detected_error.borrow().logger);
        let details = UnavailableLog {
            error: Some(error_message(error)),
            reason: None,
            decision: Some("keep-high"),
        };
        logger(
            "[Prism quality] GPU detection unavailable.",
            js::unknown(&details),
        );
    });
    let changed = Rc::clone(&state);
    let handler = listener(move |event: &'static Event| battery_change(&changed, Some(event)));
    state.borrow_mut().battery_listener = Some(handler);
    let browser_navigator = options.navigator.unwrap_or(navigator);
    let logger = Rc::clone(&state.borrow().logger);
    if !js::is_defined((browser_navigator, Navigator::get_battery)) {
        let details = UnavailableLog {
            error: None,
            reason: Some("unsupported"),
            decision: None,
        };
        logger(
            "[Prism quality] Battery status unavailable.",
            js::unknown(&details),
        );
    } else {
        let battery_task = catch_native(|| browser_navigator.get_battery());
        match battery_task {
            Err(error) => {
                let details = UnavailableLog {
                    error: Some(error_message(error)),
                    reason: Some("rejected"),
                    decision: None,
                };
                logger(
                    "[Prism quality] Battery status unavailable.",
                    js::unknown(&details),
                );
            }
            Ok(task) => {
                let battery_state = Rc::clone(&state);
                let battery_error = Rc::clone(&battery_state);
                let subscribed: Promise<()> = task.then(move |manager| {
                    if battery_state.borrow().disposed {
                        return;
                    }
                    battery_state.borrow_mut().battery = Some(manager);
                    battery_change(&battery_state, None);
                    if battery_state.borrow().downgraded {
                        return;
                    }
                    manager.add_event_listener(events::Levelchange, handler, None);
                    manager.add_event_listener(events::Chargingchange, handler, None);
                });
                let _: Promise<()> = subscribed.catch(move |error| {
                    if battery_error.borrow().disposed {
                        return;
                    }
                    let details = UnavailableLog {
                        error: Some(error_message(error)),
                        reason: Some("rejected"),
                        decision: None,
                    };
                    let logger = Rc::clone(&battery_error.borrow().logger);
                    logger(
                        "[Prism quality] Battery status unavailable.",
                        js::unknown(&details),
                    );
                });
            }
        }
    }
    let recorded = Rc::clone(&state);
    let reset = Rc::clone(&state);
    AutoQualityController {
        record_frame: Rc::new(move |sample| {
            let (status, logger) = {
                let mut state = recorded.borrow_mut();
                if state.disposed || state.downgraded {
                    return;
                }
                (state.health.record(sample), Rc::clone(&state.logger))
            };
            if !status.downgrade {
                return;
            }
            let round_fps = |value: f64| js::math::round(value * 10.0) / 10.0;
            let details = RuntimeLog {
                workload: sample.workload,
                mobile: sample.mobile,
                estimated_refresh_fps: round_fps(status.estimated_refresh_fps),
                target_fps: round_fps(status.target_fps),
                threshold_fps: round_fps(status.threshold_fps),
                observed_fps: status.observed_fps.map(round_fps),
                active_window_ms: js::math::round(status.active_window_ms),
                decision: "request-low",
            };
            logger(
                "[Prism quality] Runtime health below target.",
                js::unknown(&details),
            );
            request_low(&recorded, QualityReason::Runtime);
        }),
        reset_health: Rc::new(move || {
            let mut state = reset.borrow_mut();
            if !state.disposed && !state.downgraded {
                state.health.reset();
            }
        }),
        dispose: Rc::new(move || {
            let (battery, handler) = {
                let mut state = state.borrow_mut();
                if state.disposed {
                    return;
                }
                state.disposed = true;
                (state.battery.take(), state.battery_listener.take())
            };
            if let (Some(battery), Some(handler)) = (battery, handler) {
                battery.remove_event_listener(events::Levelchange, handler, None);
                battery.remove_event_listener(events::Chargingchange, handler, None);
            }
        }),
    }
}

// SAFETY: each log payload is always an ordinary object in JavaScript.
unsafe impl js::Defined for GpuLog {}
unsafe impl js::Defined for UnavailableLog {}
unsafe impl js::Defined for BatteryLog {}
unsafe impl js::Defined for RuntimeLog {}
