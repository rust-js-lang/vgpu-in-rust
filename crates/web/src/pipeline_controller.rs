//! Serialized theme/quality preparation with the last complete image retained.
use crate::dark_graph::{DarkFrame, DarkGraph, DarkSettings, create_dark_graph};
use crate::light_graph::{LightFrame, LightGraph, LightSettings, create_light_graph};
use crate::runtime::PrismRuntime;
use js::Promise;
use react::webapi::{GPUDevice, GPUTextureView};
use std::cell::RefCell;
use std::rc::Rc;
use vgpu_prism::controls::PrismTheme;
use vgpu_prism::quality::PipelineQuality;

pub enum SceneGraph {
    Dark(DarkGraph),
    Light(LightGraph),
}
impl SceneGraph {
    pub fn mode(&self) -> PrismTheme {
        match self {
            Self::Dark(_) => PrismTheme::Dark,
            Self::Light(_) => PrismTheme::Light,
        }
    }
    pub fn resize(&mut self, device: &GPUDevice, runtime: &PrismRuntime) -> Result<bool, String> {
        match self {
            Self::Dark(graph) => graph.resize(device, runtime),
            Self::Light(graph) => graph.resize(device, runtime),
        }
    }
    pub fn render(
        &mut self,
        device: &GPUDevice,
        runtime: &PrismRuntime,
        output: &GPUTextureView,
        frame: DarkFrame,
    ) -> Result<(), String> {
        match self {
            Self::Dark(graph) => graph.render(device, runtime, output, frame),
            Self::Light(graph) => graph.render(
                device,
                runtime,
                output,
                LightFrame {
                    update_scene: frame.update_scene,
                    reveal_progress: frame.reveal_progress,
                    beam_width_reveal: frame.beam_width_reveal,
                },
            ),
        }
    }
    pub fn destroy(&self) {
        match self {
            Self::Dark(graph) => graph.destroy(),
            Self::Light(graph) => graph.destroy(),
        }
    }
}

struct ControllerState {
    requested_mode: PrismTheme,
    requested_quality: PipelineQuality,
    active: Option<SceneGraph>,
    active_quality: Option<PipelineQuality>,
    running: Option<Promise<()>>,
    initial_ready: Option<Promise<()>>,
    cleanup: Option<Promise<()>>,
    disposed: bool,
}
#[derive(Clone)]
pub struct PipelineController {
    state: Rc<RefCell<ControllerState>>,
    pub runtime: Rc<RefCell<PrismRuntime>>,
    device: &'static GPUDevice,
    output_format: String,
    sample_count: u32,
    on_activate: Rc<dyn Fn(PrismTheme, PipelineQuality)>,
}
pub struct ControllerOptions<'a> {
    pub initial_mode: PrismTheme,
    pub initial_quality: PipelineQuality,
    pub output_format: &'a str,
    pub sample_count: u32,
    pub on_activate: Rc<dyn Fn(PrismTheme, PipelineQuality)>,
}
pub fn create_pipeline_controller(
    device: &'static GPUDevice,
    runtime: Rc<RefCell<PrismRuntime>>,
    options: ControllerOptions<'_>,
) -> PipelineController {
    let controller = PipelineController {
        state: Rc::new(RefCell::new(ControllerState {
            requested_mode: options.initial_mode,
            requested_quality: options.initial_quality,
            active: None,
            active_quality: None,
            running: None,
            initial_ready: None,
            cleanup: None,
            disposed: false,
        })),
        runtime,
        device,
        output_format: options.output_format.into(),
        sample_count: options.sample_count,
        on_activate: options.on_activate,
    };
    controller.ensure_running();
    controller
}
impl PipelineController {
    pub fn ready(&self) -> Promise<()> {
        self.state
            .borrow()
            .initial_ready
            .expect("initial preparation started")
    }
    pub fn active_mode(&self) -> Option<PrismTheme> {
        self.state.borrow().active.as_ref().map(SceneGraph::mode)
    }
    pub fn active_quality(&self) -> Option<PipelineQuality> {
        self.state.borrow().active_quality
    }
    pub fn requested_mode(&self) -> PrismTheme {
        self.state.borrow().requested_mode
    }
    pub fn requested_quality(&self) -> PipelineQuality {
        self.state.borrow().requested_quality
    }
    pub fn set_mode(&self, mode: PrismTheme) -> Promise<()> {
        let mut state = self.state.borrow_mut();
        if state.disposed {
            return js::promise::resolve(());
        }
        state.requested_mode = mode;
        drop(state);
        self.ensure_running()
    }
    pub fn set_quality(&self, quality: PipelineQuality) -> Promise<()> {
        let mut state = self.state.borrow_mut();
        if state.disposed {
            return js::promise::resolve(());
        }
        state.requested_quality = quality;
        drop(state);
        self.ensure_running()
    }
    fn ensure_running(&self) -> Promise<()> {
        if let Some(task) = self.state.borrow().running {
            return task;
        }
        let task = js::promise(self.clone().run());
        {
            let mut state = self.state.borrow_mut();
            state.running = Some(task);
            state.initial_ready.get_or_insert(task);
        }
        let fulfilled = Rc::clone(&self.state);
        let rejected = Rc::clone(&self.state);
        let _: Promise<()> = task.then_with(
            move |_| {
                fulfilled.borrow_mut().running = None;
            },
            move |_| {
                rejected.borrow_mut().running = None;
            },
        );
        task
    }
    fn matches(&self, mode: PrismTheme, quality: PipelineQuality) -> bool {
        let state = self.state.borrow();
        !state.disposed && state.requested_mode == mode && state.requested_quality == quality
    }
    async fn run(self) {
        if let Err(message) = self.run_result().await {
            js::throw(js::error::new(Some(&message), None).as_unknown());
        }
    }
    async fn run_result(self) -> Result<(), String> {
        loop {
            let (mode, quality) = {
                let state = self.state.borrow();
                if state.disposed
                    || (state
                        .active
                        .as_ref()
                        .is_some_and(|graph| graph.mode() == state.requested_mode)
                        && state.active_quality == Some(state.requested_quality))
                {
                    return Ok(());
                }
                (state.requested_mode, state.requested_quality)
            };
            let inputs = self
                .runtime
                .borrow_mut()
                .graph_inputs(self.device, quality.mesh_layout());
            let candidate = match inputs {
                Ok(inputs) => {
                    let task = js::promise(async {
                        match mode {
                            PrismTheme::Dark => create_dark_graph(
                                self.device,
                                inputs,
                                DarkSettings {
                                    output_format: &self.output_format,
                                    low_quality: quality == PipelineQuality::Low,
                                    sample_count: self.sample_count,
                                },
                            )
                            .await
                            .map(SceneGraph::Dark),
                            PrismTheme::Light => create_light_graph(
                                self.device,
                                inputs,
                                LightSettings {
                                    output_format: &self.output_format,
                                    low_quality: quality == PipelineQuality::Low,
                                    sample_count: self.sample_count,
                                },
                            )
                            .await
                            .map(SceneGraph::Light),
                        }
                    });
                    match js::settle(task).await {
                        Ok(result) => result,
                        Err(error) => Err(error.message()),
                    }
                }
                Err(error) => Err(error),
            };
            let mut candidate = match candidate {
                Ok(candidate) => candidate,
                Err(error) => {
                    if self.state.borrow().disposed {
                        return Ok(());
                    }
                    if !self.matches(mode, quality) {
                        continue;
                    }
                    return Err(error);
                }
            };
            if !self.matches(mode, quality) {
                candidate.destroy();
                continue;
            }
            // No pending graph changes the shared mesh. Only the winning candidate uploads it.
            self.runtime
                .borrow_mut()
                .set_light_mesh_layout(self.device, quality.mesh_layout());
            let resized = candidate.resize(self.device, &self.runtime.borrow());
            if let Err(error) = resized {
                candidate.destroy();
                return Err(error);
            }
            let previous = {
                let mut state = self.state.borrow_mut();
                let previous = state.active.replace(candidate);
                state.active_quality = Some(quality);
                previous
            };
            if let Some(previous) = previous {
                previous.destroy();
            }
            (self.on_activate)(mode, quality);
        }
    }
    pub fn resize(&self, size: [f64; 2]) -> Result<(), String> {
        if self.state.borrow().disposed {
            return Ok(());
        }
        self.runtime.borrow_mut().resize(self.device, size);
        if let Some(active) = &mut self.state.borrow_mut().active {
            active.resize(self.device, &self.runtime.borrow())?;
        }
        Ok(())
    }
    pub fn render(&self, output: &GPUTextureView, frame: DarkFrame) -> Result<bool, String> {
        let mut state = self.state.borrow_mut();
        if state.disposed {
            return Ok(false);
        }
        if let Some(active) = &mut state.active {
            active.render(self.device, &self.runtime.borrow(), output, frame)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// The runtime owner must await this before destroying shared GPU resources.
    pub fn destroy(&self) -> Option<Promise<()>> {
        let mut state = self.state.borrow_mut();
        if let Some(cleanup) = state.cleanup {
            return Some(cleanup);
        }
        if state.disposed {
            return None;
        }
        state.disposed = true;
        let active = state.active.take();
        state.active_quality = None;
        let running = state.running;
        drop(state);
        if let Some(active) = active {
            active.destroy();
        }
        if let Some(running) = running {
            let cleanup = running.then_with(|_| (), |_| ());
            self.state.borrow_mut().cleanup = Some(cleanup);
            Some(cleanup)
        } else {
            None
        }
    }
}
