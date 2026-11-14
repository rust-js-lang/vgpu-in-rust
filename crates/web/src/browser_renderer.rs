//! Browser canvas lifecycle from prism-background/renderer.ts.
use crate::auto_quality::{
    AutoQualityController, AutoQualityOptions, create_auto_quality_controller,
};
use crate::canvas_surface::{
    CanvasSurface, catch_native, checked, create_canvas_surface, request_prism_device,
};
use crate::dark_graph::DarkFrame;
use crate::pipeline_controller::{
    ControllerOptions, PipelineController, create_pipeline_controller,
};
use crate::quality_logging::{QualityLogger, default_quality_logger};
use crate::runtime::{PrismRuntime, create_prism_runtime};
use js::{JsError, Promise, TimeoutId};
use react::webapi::{
    Event, EventTargetExt, GPUDevice, HTMLCanvasElement, HTMLElement, IntersectionObserver,
    IntersectionObserverInit, MediaQueryList, PointerEvent, ResizeObserver, document, events,
    intersection_observer, listener, performance, resize_observer, window,
};
use std::cell::RefCell;
use std::rc::Rc;
use vgpu_prism::controls::{PrismControls, PrismTheme, PrismView};
use vgpu_prism::frame_health::{FrameHealthSample, FrameWorkload};
use vgpu_prism::framing::{NormalizedViewport, RectLike, viewport_within_canvas};
use vgpu_prism::interaction::{PrismInteraction, automatic_pointer_position};
use vgpu_prism::presentation::hero_reveal_progress;
use vgpu_prism::quality::{PipelineQuality, QualityPreference, QualityReason, QualityState};

const MOBILE_QUERY: &str = "(max-width: 767px)";
const REDUCED_QUERY: &str = "(prefers-reduced-motion: reduce)";
const OFFSCREEN_MARGIN: f64 = 256.0;
#[derive(Clone, Copy)]
pub struct RenderSize {
    pub width: f64,
    pub height: f64,
    pub dpr: f64,
}
struct Listeners {
    pointer: &'static dyn Fn(&'static PointerEvent),
    blur: &'static dyn Fn(&'static Event),
    resize: &'static dyn Fn(&'static Event),
    visibility: &'static dyn Fn(&'static Event),
}
struct RendererState {
    disposed: bool,
    reported_error: bool,
    ready: Option<Promise<()>>,
    cleanup: Option<Promise<()>>,
    device: Option<&'static GPUDevice>,
    surface: Option<CanvasSurface>,
    runtime: Option<Rc<RefCell<PrismRuntime>>>,
    controller: Option<PipelineController>,
    controls: PrismControls,
    requested_mode: PrismTheme,
    preference: QualityPreference,
    requested_quality: PipelineQuality,
    effective_quality: PipelineQuality,
    requested_reason: QualityReason,
    effective_reason: QualityReason,
    auto_generation: u32,
    auto_frame: Option<f64>,
    auto_task: Option<&'static TimeoutId>,
    auto_scheduled: bool,
    auto_controller: Option<AutoQualityController>,
    quality_transition: u32,
    interaction: PrismInteraction,
    mobile: Option<&'static MediaQueryList>,
    reduced_motion: bool,
    listeners: Option<Listeners>,
    observer: Option<&'static ResizeObserver>,
    visibility_observer: Option<&'static IntersectionObserver>,
    resize_frame: Option<f64>,
    pending_size: Option<RenderSize>,
    pending_framing: Option<NormalizedViewport>,
    framing_pending: bool,
    frame: Option<f64>,
    scheduling_ready: bool,
    running: bool,
    document_visible: bool,
    near_viewport: bool,
    pending_present: bool,
    last_dust_time: f64,
    last_reveal: f64,
    last_beam: f64,
    clock_last_ms: f64,
    clock_time: f64,
    reveal_start: Option<f64>,
    budget: f64,
    has_capped_frame: bool,
    submitted_frames: u32,
}
pub type AutoQualityFactory = fn(AutoQualityOptions) -> AutoQualityController;
pub struct BrowserRenderer {
    quality_listeners: &'static js::Set<dyn Fn(QualityState)>,
    quality_logger: QualityLogger,
    load_auto_quality: Option<Rc<dyn Fn() -> Promise<AutoQualityFactory>>>,
    canvas: &'static HTMLCanvasElement,
    framing_element: Option<&'static HTMLElement>,
    on_error: Rc<dyn Fn(&'static JsError)>,
    state: RefCell<RendererState>,
}
pub struct RendererOptions {
    pub canvas: &'static HTMLCanvasElement,
    pub framing_element: Option<&'static HTMLElement>,
    pub initial_mode: PrismTheme,
    pub initial_quality: QualityPreference,
    pub initial_controls: Option<PrismControls>,
    pub quality_logger: Option<QualityLogger>,
    pub load_auto_quality: Option<Rc<dyn Fn() -> Promise<AutoQualityFactory>>>,
    pub on_error: Rc<dyn Fn(&'static JsError)>,
}
pub struct RendererStatus {
    pub disposed: bool,
    pub running: bool,
    pub submitted_frames: u32,
    pub size: [f64; 2],
    pub mode: Option<PrismTheme>,
    pub quality: PipelineQuality,
}
pub fn create_renderer(options: RendererOptions) -> Rc<BrowserRenderer> {
    let can_match_media = js::is_defined((window, react::webapi::Window::match_media));
    let mobile = can_match_media.then(|| window.match_media(MOBILE_QUERY));
    let reduced_motion = can_match_media && window.match_media(REDUCED_QUERY).matches();
    let quality = options.initial_quality.initial_quality();
    let reason = if options.initial_quality == QualityPreference::Auto {
        QualityReason::Initial
    } else {
        QualityReason::Forced
    };
    let renderer = Rc::new(BrowserRenderer {
        quality_listeners: js::set::new(None),
        quality_logger: options
            .quality_logger
            .unwrap_or_else(default_quality_logger),
        load_auto_quality: options.load_auto_quality,
        canvas: options.canvas,
        framing_element: options.framing_element,
        on_error: options.on_error,
        state: RefCell::new(RendererState {
            disposed: false,
            reported_error: false,
            ready: None,
            cleanup: None,
            device: None,
            surface: None,
            runtime: None,
            controller: None,
            controls: options.initial_controls.unwrap_or_default(),
            requested_mode: options.initial_mode,
            preference: options.initial_quality,
            requested_quality: quality,
            effective_quality: quality,
            requested_reason: reason,
            effective_reason: reason,
            auto_generation: 0,
            auto_frame: None,
            auto_task: None,
            auto_scheduled: false,
            auto_controller: None,
            quality_transition: 0,
            interaction: PrismInteraction::default(),
            mobile,
            reduced_motion,
            listeners: None,
            observer: None,
            visibility_observer: None,
            resize_frame: None,
            pending_size: None,
            pending_framing: None,
            framing_pending: false,
            frame: None,
            scheduling_ready: false,
            running: false,
            document_visible: true,
            near_viewport: true,
            pending_present: true,
            last_dust_time: -1.0,
            last_reveal: -1.0,
            last_beam: -1.0,
            clock_last_ms: 0.0,
            clock_time: 0.0,
            reveal_start: None,
            budget: 0.0,
            has_capped_frame: false,
            submitted_frames: 0,
        }),
    });
    let ready = js::promise(Rc::clone(&renderer).start());
    renderer.state.borrow_mut().ready = Some(ready);
    // The React integration awaits the original ready promise. Keep ignored handles quiet too.
    let _: Promise<()> = ready.then_with(|_| (), |_| ());
    renderer
}
impl BrowserRenderer {
    pub fn ready(&self) -> Promise<()> {
        self.state
            .borrow()
            .ready
            .expect("renderer initialization started")
    }
    async fn start(self: Rc<Self>) {
        let result = js::settle(js::promise(Rc::clone(&self).initialize())).await;
        if let Err(error) = result
            && !self.state.borrow().disposed
        {
            self.handle_failure(error);
            js::throw(error.as_unknown());
        }
    }
    async fn initialize(self: Rc<Self>) {
        let device = request_prism_device(false).await;
        if self.state.borrow().disposed {
            device.destroy();
            return;
        }
        self.state.borrow_mut().device = Some(device);
        let rect = self.canvas.get_bounding_client_rect();
        let quality = self.state.borrow().requested_quality;
        let dpr = self.dpr(quality);
        let surface = checked(create_canvas_surface(
            device,
            self.canvas,
            [
                (rect.width() * dpr).round().max(1.0),
                (rect.height() * dpr).round().max(1.0),
            ],
        ));
        let format = surface.format.clone();
        let runtime = Rc::new(RefCell::new(create_prism_runtime(
            device,
            surface.size(),
            "prism-rainbow",
            quality.mesh_layout(),
            false,
        )));
        runtime
            .borrow_mut()
            .set_controls(device, self.state.borrow().controls.clone());
        {
            let mut state = self.state.borrow_mut();
            state.surface = Some(surface);
            state.runtime = Some(Rc::clone(&runtime));
        }
        let on_activate = Rc::clone(&self);
        let controller = create_pipeline_controller(
            device,
            runtime,
            ControllerOptions {
                initial_mode: self.state.borrow().requested_mode,
                initial_quality: quality,
                output_format: &format,
                sample_count: 4,
                on_activate: Rc::new(move |_, quality| {
                    let changed = {
                        let mut state = on_activate.state.borrow_mut();
                        if state.disposed {
                            return;
                        }
                        let changed = state.effective_quality != quality;
                        // Commit the matching requested reason only at activation.
                        // An older winning scene keeps its previous reason.
                        state.pending_present = true;
                        state.last_dust_time = -1.0;
                        state.budget = 0.0;
                        state.has_capped_frame = false;
                        changed
                    };
                    let reason = {
                        let state = on_activate.state.borrow();
                        if quality == state.requested_quality {
                            state.requested_reason
                        } else {
                            state.effective_reason
                        }
                    };
                    on_activate.set_effective_quality(quality, reason);
                    on_activate.reset_health();
                    if changed {
                        on_activate.measure_for_quality(quality, false);
                    }
                }),
            },
        );
        self.state.borrow_mut().controller = Some(controller.clone());
        self.install_listeners();
        self.measure();
        let lost = Rc::clone(&self);
        let _: Promise<()> = device.lost().then(move |info| {
            if !lost.state.borrow().disposed {
                let message = format!("WebGPU device lost: {}", info.message());
                lost.handle_failure(js::error::new(Some(&message), None));
            }
        });
        controller.ready().await;
        if self.state.borrow().disposed {
            return;
        }
        let near = self.is_near_viewport();
        {
            let mut state = self.state.borrow_mut();
            state.clock_last_ms = state
                .runtime
                .as_ref()
                .and_then(|runtime| {
                    let runtime = runtime.borrow();
                    runtime
                        .studio_environment
                        .as_ref()
                        .and_then(|environment| environment.first_frame_ms)
                })
                .unwrap_or_else(|| performance.now());
            state.document_visible = !document.hidden();
            state.near_viewport = near;
            state.scheduling_ready = true;
        }
        if js::is_defined(intersection_observer::new) {
            let renderer = Rc::clone(&self);
            let observer = intersection_observer::new(
                Box::new(move |entries, _| {
                    if let Some(entry) = entries.last() {
                        {
                            let mut state = renderer.state.borrow_mut();
                            if state.disposed {
                                return;
                            }
                            state.near_viewport = entry.is_intersecting();
                        }
                        renderer.reconcile_loop();
                    }
                }),
                Some(IntersectionObserverInit {
                    root_margin: Some("256px 0px"),
                    threshold: Some(0.0.into()),
                    ..Default::default()
                }),
            );
            observer.observe(self.canvas);
            self.state.borrow_mut().visibility_observer = Some(observer);
        }
        self.reconcile_loop();
    }
    fn install_listeners(self: &Rc<Self>) {
        let renderer = Rc::clone(self);
        let pointer = listener(move |event: &'static PointerEvent| {
            let mobile = renderer
                .state
                .borrow()
                .mobile
                .is_some_and(|query| query.matches());
            if mobile || renderer.state.borrow().disposed {
                return;
            }
            let rect = renderer.canvas.get_bounding_client_rect();
            let mut state = renderer.state.borrow_mut();
            if state.interaction.on_pointer_move(
                event.is_primary(),
                [event.client_x(), event.client_y()],
                [rect.left(), rect.top()],
                [rect.width(), rect.height()],
            ) {
                state.pending_present = true;
            }
        });
        let renderer = Rc::clone(self);
        let blur = listener(move |_: &'static Event| {
            renderer.state.borrow_mut().interaction.on_pointer_leave()
        });
        let renderer = Rc::clone(self);
        let resize = listener(move |_: &'static Event| renderer.measure());
        let renderer = Rc::clone(self);
        let visibility = listener(move |_: &'static Event| {
            {
                let mut state = renderer.state.borrow_mut();
                if state.disposed {
                    return;
                }
                state.document_visible = !document.hidden();
                if !state.document_visible {
                    state.interaction.on_pointer_leave();
                }
            }
            renderer.reconcile_loop();
        });
        window.add_event_listener(
            events::Pointermove,
            pointer,
            Some(&react::webapi::AddEventListenerOptions {
                passive: Some(true),
                ..Default::default()
            }),
        );
        window.add_event_listener(events::Blur, blur, None);
        window.add_event_listener(events::Resize, resize, None);
        document.add_event_listener(events::Visibilitychange, visibility, None);
        self.state.borrow_mut().listeners = Some(Listeners {
            pointer,
            blur,
            resize,
            visibility,
        });
        if js::is_defined(resize_observer::new) {
            let renderer = Rc::clone(self);
            let observer = resize_observer::new(Box::new(move |_, _| renderer.measure()));
            observer.observe(self.canvas, None);
            if let Some(framing) = self.framing_element {
                observer.observe(framing, None);
            }
            self.state.borrow_mut().observer = Some(observer);
        }
    }
    fn dpr(&self, quality: PipelineQuality) -> f64 {
        if quality == PipelineQuality::Low {
            1.0
        } else {
            let dpr = window.device_pixel_ratio();
            if dpr == 0.0 || dpr.is_nan() {
                1.0
            } else {
                dpr.clamp(1.0, 2.0)
            }
        }
    }
    fn measure(self: &Rc<Self>) {
        let quality = self.state.borrow().requested_quality;
        self.measure_for_quality(quality, false);
    }
    fn measure_for_quality(self: &Rc<Self>, quality: PipelineQuality, immediate: bool) {
        if self.state.borrow().disposed {
            return;
        }
        let rect = self.canvas.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let framing = self.framing_element.map(|element| {
            let target = element.get_bounding_client_rect();
            viewport_within_canvas(
                RectLike {
                    left: rect.left(),
                    top: rect.top(),
                    width: rect.width(),
                    height: rect.height(),
                },
                RectLike {
                    left: target.left(),
                    top: target.top(),
                    width: target.width(),
                    height: target.height(),
                },
            )
        });
        let dpr = self.dpr(quality);
        self.reset_health();
        {
            let mut state = self.state.borrow_mut();
            state.pending_size = Some(RenderSize {
                width: rect.width(),
                height: rect.height(),
                dpr,
            });
            if let Some(framing) = framing {
                state.pending_framing = framing;
                state.framing_pending = true;
            }
        }
        self.queue_resize();
        if immediate {
            if let Some(frame) = self.state.borrow_mut().resize_frame.take() {
                window.cancel_animation_frame(frame);
            }
            self.apply_resize();
        }
    }
    pub fn resize(self: &Rc<Self>, size: RenderSize) {
        if self.state.borrow().disposed || size.width <= 0.0 || size.height <= 0.0 {
            return;
        }
        self.reset_health();
        self.state.borrow_mut().pending_size = Some(size);
        self.queue_resize();
    }
    fn queue_resize(self: &Rc<Self>) {
        if self.state.borrow().resize_frame.is_some() {
            return;
        }
        let renderer = Rc::clone(self);
        let frame = window.request_animation_frame(Box::new(move |_| renderer.apply_resize()));
        self.state.borrow_mut().resize_frame = Some(frame);
    }
    fn apply_resize(self: &Rc<Self>) {
        if let Err(error) = catch_native(|| {
            let (controller, runtime, device, size, framing, apply_framing) = {
                let mut state = self.state.borrow_mut();
                state.resize_frame = None;
                let size = state.pending_size.take();
                let framing = state.pending_framing.take();
                let apply_framing = state.framing_pending;
                state.framing_pending = false;
                if state.disposed {
                    return;
                }
                (
                    state.controller.clone(),
                    state.runtime.clone(),
                    state.device,
                    size,
                    framing,
                    apply_framing,
                )
            };
            if let (Some(controller), Some(runtime), Some(device), Some(size)) =
                (controller, runtime, device, size)
            {
                let next = [
                    (size.width * size.dpr).round().max(1.0),
                    (size.height * size.dpr).round().max(1.0),
                ];
                if let Some(surface) = &self.state.borrow().surface {
                    surface.resize(next);
                }
                checked(controller.resize(next));
                if apply_framing {
                    runtime.borrow_mut().set_framing_viewport(device, framing);
                }
                self.state.borrow_mut().pending_present = true;
            }
        }) {
            self.handle_failure(error);
        }
    }
    fn is_near_viewport(&self) -> bool {
        let rect = self.canvas.get_bounding_client_rect();
        let height = window.inner_height();
        if !height.is_finite() || !rect.top().is_finite() || !rect.bottom().is_finite() {
            return true;
        }
        rect.bottom() >= -OFFSCREEN_MARGIN && rect.top() <= height + OFFSCREEN_MARGIN
    }
    fn reconcile_loop(self: &Rc<Self>) {
        let (start, stop) = {
            let mut state = self.state.borrow_mut();
            if !state.scheduling_ready {
                return;
            }
            let should_run = !state.disposed && state.document_visible && state.near_viewport;
            if should_run == state.running {
                return;
            }
            state.running = should_run;
            if should_run {
                state.pending_present = true;
                state.last_dust_time = -1.0;
                state.budget = 0.0;
                state.has_capped_frame = false;
                (true, None)
            } else {
                (false, state.frame.take())
            }
        };
        self.reset_health();
        if let Some(frame) = stop {
            window.cancel_animation_frame(frame);
        }
        if start {
            self.measure();
            self.schedule_frame();
        }
    }
    fn schedule_frame(self: &Rc<Self>) {
        let state = self.state.borrow();
        if !state.running || state.disposed || state.frame.is_some() {
            return;
        }
        drop(state);
        let renderer = Rc::clone(self);
        let frame = window.request_animation_frame(Box::new(move |_| renderer.tick()));
        self.state.borrow_mut().frame = Some(frame);
    }
    fn tick(self: &Rc<Self>) {
        self.state.borrow_mut().frame = None;
        if let Err(error) = catch_native(|| self.paint_frame()) {
            self.handle_failure(error);
        }
        self.schedule_frame();
    }
    fn paint_frame(self: &Rc<Self>) {
        let health_sample;
        let controller = self.state.borrow().controller.clone();
        let Some(controller) = controller else {
            return;
        };
        let Some(mode) = controller.active_mode() else {
            return;
        };
        let (frame, aim, orbit, runtime, context, device) = {
            let mut state = self.state.borrow_mut();
            if state.disposed || !state.running {
                return;
            }
            let now = performance.now();
            let delta = ((now - state.clock_last_ms) / 1000.0).max(0.0);
            state.clock_last_ms = now;
            state.clock_time += delta;
            let time = state.clock_time;
            let mobile = state.mobile.is_some_and(|query| query.matches());
            if mobile {
                state
                    .interaction
                    .set_normalized_pointer(automatic_pointer_position(time));
            }
            let aim = state.interaction.step_aim();
            let orbit = state.interaction.step_orbit();
            let reveal_start = *state.reveal_start.get_or_insert(time);
            let reveal = if state.reduced_motion {
                vgpu_prism::presentation::HeroReveal {
                    opacity: 1.0,
                    beam_width: 1.0,
                }
            } else {
                hero_reveal_progress(time - reveal_start)
            };
            let reveal_changed = reveal.opacity != state.last_reveal;
            let update_scene = aim.is_some()
                || orbit.is_some()
                || state.pending_present
                || reveal.beam_width != state.last_beam;
            let dust_time = (time * 30.0).floor() / 30.0;
            let dust_moved = mode == PrismTheme::Dark
                && state.controls.view == PrismView::Glass
                && dust_time != state.last_dust_time;
            let dust_animating =
                mode == PrismTheme::Dark && state.controls.view == PrismView::Glass;
            health_sample = FrameHealthSample {
                delta_ms: delta * 1000.0,
                active: (update_scene || dust_moved || reveal_changed || dust_animating)
                    && state.preference == QualityPreference::Auto
                    && controller.active_quality() == Some(PipelineQuality::High)
                    && state.document_visible
                    && state.near_viewport,
                rendered: false,
                mobile,
                workload: if dust_animating && !update_scene && !reveal_changed {
                    FrameWorkload::Dust
                } else {
                    FrameWorkload::Interactive
                },
            };
            if state.has_capped_frame && delta.is_finite() && delta > 0.0 {
                let interval = 1.0
                    / if mobile {
                        30.0
                    } else if controller.active_quality() == Some(PipelineQuality::Low) {
                        60.0
                    } else {
                        90.0
                    };
                state.budget = (state.budget + delta).min(interval * 2.0);
                if state.budget + 1e-9 < interval {
                    drop(state);
                    self.record_health(health_sample);
                    return;
                }
                state.budget = (state.budget - interval).max(0.0);
            } else {
                state.has_capped_frame = true;
            }
            if !update_scene && !dust_moved && !reveal_changed {
                drop(state);
                self.record_health(health_sample);
                return;
            }
            (
                DarkFrame {
                    time: dust_time,
                    update_scene,
                    reveal_progress: reveal.opacity,
                    beam_width_reveal: reveal.beam_width,
                },
                aim,
                orbit,
                state.runtime.clone().expect("runtime initialized"),
                state.surface.as_ref().expect("surface initialized").context,
                state.device.expect("device initialized"),
            )
        };
        if let Some(aim) = aim {
            runtime.borrow_mut().set_lamp_aim(device, aim[0], aim[1]);
        }
        if let Some(orbit) = orbit {
            runtime.borrow_mut().set_orbit(orbit[0], orbit[1]);
        }
        if checked(controller.render(context.get_current_texture().create_view(None), frame)) {
            let mut state = self.state.borrow_mut();
            state.pending_present = false;
            state.last_dust_time = frame.time;
            state.last_reveal = frame.reveal_progress;
            state.last_beam = frame.beam_width_reveal;
            state.submitted_frames += 1;
            drop(state);
            self.record_health(FrameHealthSample {
                rendered: true,
                ..health_sample
            });
            self.schedule_auto_quality();
        }
    }
    pub fn set_controls(&self, controls: PrismControls) {
        let (runtime, device) = {
            let mut state = self.state.borrow_mut();
            if state.disposed {
                return;
            }
            state.controls = controls.clone();
            state.pending_present = true;
            (state.runtime.clone(), state.device)
        };
        if let (Some(runtime), Some(device)) = (runtime, device) {
            runtime.borrow_mut().set_controls(device, controls);
        }
    }
    pub fn invalidate(&self) {
        if !self.state.borrow().disposed {
            self.state.borrow_mut().pending_present = true;
        }
    }
    pub async fn set_mode(self: Rc<Self>, mode: PrismTheme) {
        self.reset_health();
        let controller = {
            let mut state = self.state.borrow_mut();
            if state.disposed {
                return;
            }
            state.requested_mode = mode;
            state.controller.clone()
        };
        let Some(controller) = controller else {
            self.ready().await;
            return;
        };
        match js::settle(controller.set_mode(mode)).await {
            Ok(()) => {
                let mut state = self.state.borrow_mut();
                if state.disposed {
                    return;
                }
                state.pending_present = true;
                state.last_dust_time = -1.0;
                drop(state);
                self.reset_health();
            }
            Err(error) => {
                if self.state.borrow().disposed {
                    return;
                }
                self.report_recoverable(error);
                js::throw(error.as_unknown());
            }
        }
    }
    pub fn get_quality_state(&self) -> QualityState {
        let state = self.state.borrow();
        QualityState {
            preference: state.preference,
            effective: state.effective_quality,
            reason: state.effective_reason,
        }
    }
    fn emit_quality_state(&self) {
        let snapshot = self.get_quality_state();
        for listener in self.quality_listeners.values() {
            listener(snapshot);
        }
    }
    fn set_effective_quality(&self, quality: PipelineQuality, reason: QualityReason) {
        {
            let mut state = self.state.borrow_mut();
            if state.effective_quality == quality && state.effective_reason == reason {
                return;
            }
            state.effective_quality = quality;
            state.effective_reason = reason;
        }
        self.emit_quality_state();
    }
    pub fn subscribe_quality(
        self: &Rc<Self>,
        listener: &'static dyn Fn(QualityState),
    ) -> Rc<dyn Fn()> {
        if self.state.borrow().disposed {
            return Rc::new(|| {});
        }
        self.quality_listeners.add(listener);
        let listeners = self.quality_listeners;
        Rc::new(move || {
            listeners.delete(listener);
        })
    }
    fn record_health(&self, sample: FrameHealthSample) {
        let record = self
            .state
            .borrow()
            .auto_controller
            .as_ref()
            .map(|controller| Rc::clone(&controller.record_frame));
        if let Some(record) = record {
            record(sample);
        }
    }
    fn reset_health(&self) {
        let reset = self
            .state
            .borrow()
            .auto_controller
            .as_ref()
            .map(|controller| Rc::clone(&controller.reset_health));
        if let Some(reset) = reset {
            reset();
        }
    }
    fn cancel_auto_quality(&self) {
        let (controller, frame, task) = {
            let mut state = self.state.borrow_mut();
            state.auto_generation += 1;
            state.auto_scheduled = false;
            (
                state.auto_controller.take(),
                state.auto_frame.take(),
                state.auto_task.take(),
            )
        };
        if let Some(controller) = controller {
            (controller.dispose)();
        }
        if let Some(frame) = frame {
            window.cancel_animation_frame(frame);
        }
        if let Some(task) = task {
            js::clear_timeout(Some(task));
        }
    }
    fn schedule_auto_quality(self: &Rc<Self>) {
        let generation = {
            let mut state = self.state.borrow_mut();
            if state.disposed
                || state.preference != QualityPreference::Auto
                || state.effective_quality != PipelineQuality::High
                || state.auto_controller.is_some()
                || state.auto_scheduled
            {
                return;
            }
            state.auto_scheduled = true;
            state.auto_generation
        };
        let renderer = Rc::clone(self);
        let frame = window.request_animation_frame(Box::new(move |_| {
            renderer.state.borrow_mut().auto_frame = None;
            let deferred = Rc::clone(&renderer);
            let task = js::set_timeout(
                move || {
                    deferred.state.borrow_mut().auto_task = None;
                    let _: Promise<()> =
                        js::promise(Rc::clone(&deferred).start_auto_quality(generation))
                            .then_with(|_| (), |_| ());
                },
                Some(0.0),
            );
            renderer.state.borrow_mut().auto_task = Some(task);
        }));
        self.state.borrow_mut().auto_frame = Some(frame);
    }
    fn auto_is_current(&self, generation: u32) -> bool {
        let state = self.state.borrow();
        !state.disposed
            && state.auto_generation == generation
            && state.preference == QualityPreference::Auto
            && state.effective_quality == PipelineQuality::High
    }
    async fn start_auto_quality(self: Rc<Self>, generation: u32) {
        let result = match &self.load_auto_quality {
            Some(load) => match catch_native(|| load()) {
                Ok(task) => js::settle(task).await,
                Err(error) => Err(error),
            },
            None => js::settle(js::import!(create_auto_quality_controller))
                .await
                .map(|create| create as AutoQualityFactory),
        };
        if let Ok(create) = result {
            if !self.auto_is_current(generation) {
                return;
            }
            self.state.borrow_mut().auto_scheduled = false;
            let downgrade = Rc::clone(&self);
            let controller = create(AutoQualityOptions {
                logger: Some(Rc::clone(&self.quality_logger)),
                navigator: None,
                load_gpu_tier: None,
                on_downgrade: Rc::new(move |reason| {
                    if !downgrade.auto_is_current(generation) {
                        return;
                    }
                    let _: Promise<()> =
                        js::promise(Rc::clone(&downgrade).apply_effective_quality(
                            PipelineQuality::Low,
                            reason,
                            generation,
                        ))
                        .then_with(|_| (), |_| ());
                }),
            });
            self.state.borrow_mut().auto_controller = Some(controller);
        } else if self.state.borrow().auto_generation == generation {
            self.state.borrow_mut().auto_scheduled = false;
        }
    }
    pub async fn set_quality_preference(self: Rc<Self>, preference: QualityPreference) {
        if self.state.borrow().disposed {
            return;
        }
        self.cancel_auto_quality();
        let quality = preference.initial_quality();
        let reason = if preference == QualityPreference::Auto {
            QualityReason::Initial
        } else {
            QualityReason::Forced
        };
        let generation = {
            let mut state = self.state.borrow_mut();
            state.preference = preference;
            if state.effective_quality == quality {
                state.effective_reason = reason;
            }
            state.auto_generation
        };
        self.emit_quality_state();
        self.apply_effective_quality(quality, reason, generation)
            .await;
    }
    async fn apply_effective_quality(
        self: Rc<Self>,
        quality: PipelineQuality,
        reason: QualityReason,
        generation: u32,
    ) {
        let (transition, previous, previous_reason) = {
            let mut state = self.state.borrow_mut();
            if state.disposed || state.auto_generation != generation {
                return;
            }
            let previous = state.requested_quality;
            let previous_reason = state.requested_reason;
            state.requested_quality = quality;
            state.requested_reason = reason;
            state.quality_transition += 1;
            (state.quality_transition, previous, previous_reason)
        };
        if self.state.borrow().controller.is_none() {
            self.ready().await;
        }
        {
            let state = self.state.borrow();
            if state.disposed
                || state.auto_generation != generation
                || state.quality_transition != transition
            {
                return;
            }
        }
        let controller = self.state.borrow().controller.clone();
        let Some(controller) = controller else {
            return;
        };
        if controller.active_quality() == Some(quality) && controller.requested_quality() == quality
        {
            self.set_effective_quality(quality, reason);
            self.state.borrow_mut().pending_present = true;
            self.reset_health();
            return;
        }
        if quality == PipelineQuality::Low {
            self.measure_for_quality(quality, true);
        }
        let result = js::settle(controller.set_quality(quality)).await;
        {
            let state = self.state.borrow();
            if state.disposed
                || state.auto_generation != generation
                || state.quality_transition != transition
            {
                return;
            }
        }
        match result {
            Ok(()) => {
                if quality == PipelineQuality::High {
                    self.measure_for_quality(quality, true);
                }
                self.set_effective_quality(quality, reason);
                let auto = {
                    let mut state = self.state.borrow_mut();
                    state.pending_present = true;
                    state.last_dust_time = -1.0;
                    if quality == PipelineQuality::Low {
                        state.auto_controller.take()
                    } else {
                        None
                    }
                };
                if let Some(auto) = auto {
                    (auto.dispose)();
                }
                if quality == PipelineQuality::Low && reason != QualityReason::Forced {
                    let details = DowngradeLog {
                        preference: self.state.borrow().preference,
                        reason,
                        from: PipelineQuality::High,
                        to: PipelineQuality::Low,
                        dpr: 1.0,
                    };
                    (self.quality_logger)(
                        "[Prism quality] Downgraded to Low.",
                        js::unknown(&details),
                    );
                }
            }
            Err(error) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.requested_quality = previous;
                    state.requested_reason = previous_reason;
                }
                let effective = self.state.borrow().effective_quality;
                let _: Promise<()> = controller.set_quality(effective).then_with(|_| (), |_| ());
                if quality == PipelineQuality::Low {
                    self.measure_for_quality(effective, true);
                }
                self.report_recoverable(error);
                js::throw(error.as_unknown());
            }
        }
    }
    fn report_recoverable(&self, error: &'static JsError) {
        if !self.state.borrow().disposed {
            let _ = catch_native(|| (self.on_error)(error));
        }
    }
    fn handle_failure(self: &Rc<Self>, error: &'static JsError) {
        {
            let mut state = self.state.borrow_mut();
            if state.disposed {
                return;
            }
            if !state.reported_error {
                state.reported_error = true;
                drop(state);
                let _ = catch_native(|| (self.on_error)(error));
            }
        }
        self.dispose();
    }
    pub fn status(&self) -> RendererStatus {
        let state = self.state.borrow();
        RendererStatus {
            disposed: state.disposed,
            running: state.running,
            submitted_frames: state.submitted_frames,
            size: state
                .surface
                .as_ref()
                .map(CanvasSurface::size)
                .unwrap_or([self.canvas.width(), self.canvas.height()]),
            mode: state
                .controller
                .as_ref()
                .and_then(PipelineController::active_mode),
            quality: state.effective_quality,
        }
    }
    pub fn dispose(self: &Rc<Self>) -> Promise<()> {
        self.cancel_auto_quality();
        self.quality_listeners.clear();
        let (
            controller,
            runtime,
            surface,
            device,
            ready,
            listeners,
            observer,
            visibility,
            frame,
            resize,
        ) = {
            let mut state = self.state.borrow_mut();
            if let Some(cleanup) = state.cleanup {
                return cleanup;
            }
            if state.disposed {
                return js::promise::resolve(());
            }
            state.disposed = true;
            state.running = false;
            state.scheduling_ready = false;
            (
                state.controller.take(),
                state.runtime.take(),
                state.surface.take(),
                state.device.take(),
                state.ready,
                state.listeners.take(),
                state.observer.take(),
                state.visibility_observer.take(),
                state.frame.take(),
                state.resize_frame.take(),
            )
        };
        if let Some(frame) = frame {
            window.cancel_animation_frame(frame);
        }
        if let Some(frame) = resize {
            window.cancel_animation_frame(frame);
        }
        if let Some(observer) = observer {
            observer.disconnect();
        }
        if let Some(observer) = visibility {
            observer.disconnect();
        }
        if let Some(listeners) = listeners {
            window.remove_event_listener(events::Pointermove, listeners.pointer, None);
            window.remove_event_listener(events::Blur, listeners.blur, None);
            window.remove_event_listener(events::Resize, listeners.resize, None);
            document.remove_event_listener(events::Visibilitychange, listeners.visibility, None);
        }
        let pending = controller
            .as_ref()
            .and_then(PipelineController::destroy)
            .or(ready);
        let owned = Rc::new(RefCell::new(Some((runtime, surface, device))));
        let finish: Rc<dyn Fn()> = Rc::new(move || {
            if let Some((runtime, surface, device)) = owned.borrow_mut().take() {
                if let Some(runtime) = runtime {
                    runtime.borrow_mut().destroy();
                }
                if let Some(surface) = surface {
                    surface.destroy();
                }
                if let Some(device) = device {
                    device.destroy();
                }
            }
        });
        let cleanup = if let Some(pending) = pending {
            let rejected = Rc::clone(&finish);
            pending.then_with(move |_| finish(), move |_| rejected())
        } else {
            finish();
            js::promise::resolve(())
        };
        self.state.borrow_mut().cleanup = Some(cleanup);
        cleanup
    }
}

struct DowngradeLog {
    preference: QualityPreference,
    reason: QualityReason,
    from: PipelineQuality,
    to: PipelineQuality,
    dpr: f64,
}
// SAFETY: the log payload is always an ordinary JavaScript object.
unsafe impl js::Defined for DowngradeLog {}
