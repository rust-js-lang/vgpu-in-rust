//! Shared retained GPU resources and mutable scene state from runtime/resources.ts.
use crate::buffers::create_vertex_buffer;
use crate::environment::{
    EnvironmentTexture, create_environment_sampler, create_environment_texture,
};
use crate::geometry::{PrismBuffers, create_prism_geometry};
use crate::light_mesh::build_light_mesh;
use js::{Float32Array, JsError, Promise};
use react::webapi::{GPUBuffer, GPUDevice, GPUSampler, GPUSamplerDescriptor, GPUTextureView};
use std::cell::RefCell;
use std::rc::Rc;
use vgpu_prism::controls::PrismControls;
use vgpu_prism::framing::NormalizedViewport;
use vgpu_prism::light_mesh::{
    LIGHT_VERTEX_FLOATS, LIGHT_VERTEX_STRIDE, LightMeshLayout, LightMeshStats,
};
use vgpu_prism::quality::PipelineQuality;
use vgpu_prism::runtime_state::RuntimeState;

/// Preparation owns these immutable inputs while the live runtime remains usable.
/// GPU handles are borrowed from the runtime, which outlives pending preparations.
pub struct GraphInputs {
    pub label: String,
    pub state: RuntimeState,
    pub light_buffer: &'static GPUBuffer,
    pub prism: PrismBuffers,
    pub scene_sampler: &'static GPUSampler,
    pub environment_sampler: &'static GPUSampler,
    pub debug_environment_enabled: bool,
    pub studio_view: &'static GPUTextureView,
    pub debug_view: &'static GPUTextureView,
    pub environment_ready: Promise<()>,
}

pub struct PrismRuntime {
    pub label: String,
    pub state: RuntimeState,
    pub light_buffer: &'static GPUBuffer,
    pub light_vertices: &'static Float32Array,
    pub light_vertex_scratch: Vec<f64>,
    pub light_stats: LightMeshStats,
    pub prism: PrismBuffers,
    pub prism_wireframe: Option<PrismBuffers>,
    pub scene_sampler: &'static GPUSampler,
    pub environment_sampler: &'static GPUSampler,
    pub debug_environment_enabled: bool,
    pub studio_environment: Option<EnvironmentTexture>,
    pub debug_environment: Option<EnvironmentTexture>,
    pub environment_ready: Option<Promise<()>>,
}
pub fn create_prism_runtime(
    device: &GPUDevice,
    output: [f64; 2],
    label: &str,
    layout: LightMeshLayout,
    debug_environment: bool,
) -> PrismRuntime {
    let state = RuntimeState::new(output, layout);
    let mut scratch = Vec::new();
    let mut options = state.light_mesh_options();
    // Initial creation uses the unextended wall. Responsive refresh later
    // applies the portrait overscan, as runtime/resources.ts and state.ts do.
    options.wall_half_extent = state.wall_extent();
    let mesh = build_light_mesh(&options, None, Some(&mut scratch));
    let high = PipelineQuality::High.mesh_layout();
    let light_buffer = create_vertex_buffer(
        device,
        mesh.vertices,
        &format!("{label}.light-vertices"),
        Some((high.vertex_count * LIGHT_VERTEX_STRIDE) as f64),
    );
    PrismRuntime {
        label: label.into(),
        state,
        light_buffer,
        light_vertices: mesh.vertices,
        light_vertex_scratch: scratch,
        light_stats: mesh.stats,
        prism: create_prism_geometry(device, &format!("{label}.prism"), false),
        prism_wireframe: None,
        scene_sampler: device.create_sampler(Some(GPUSamplerDescriptor {
            min_filter: Some("linear"),
            mag_filter: Some("linear"),
            address_mode_u: Some("clamp-to-edge"),
            address_mode_v: Some("clamp-to-edge"),
            ..Default::default()
        })),
        environment_sampler: create_environment_sampler(device),
        debug_environment_enabled: debug_environment,
        studio_environment: None,
        debug_environment: None,
        environment_ready: None,
    }
}
impl PrismRuntime {
    /// Capture a candidate's layout without uploading it into the active mesh.
    /// The controller changes the live layout only when that candidate activates.
    pub fn graph_inputs(
        &mut self,
        device: &'static GPUDevice,
        layout: LightMeshLayout,
    ) -> Result<GraphInputs, String> {
        if self.state.controls.wireframe {
            self.ensure_wireframe(device);
        }
        let environment_ready = self.prepare_environment(device)?;
        let studio_view = self
            .studio_environment
            .as_ref()
            .expect("studio created")
            .texture
            .create_view(None);
        let debug_view = self
            .debug_environment
            .as_ref()
            .map(|debug| debug.texture.create_view(None))
            .unwrap_or(studio_view);
        let mut state = self.state.clone();
        state.set_light_mesh_layout(layout);
        Ok(GraphInputs {
            label: self.label.clone(),
            state,
            light_buffer: self.light_buffer,
            prism: self.prism,
            scene_sampler: self.scene_sampler,
            environment_sampler: self.environment_sampler,
            debug_environment_enabled: self.debug_environment_enabled,
            studio_view,
            debug_view,
            environment_ready,
        })
    }

    pub fn set_controls(&mut self, device: &GPUDevice, controls: PrismControls) {
        if self.state.set_controls(controls) {
            self.refresh_light_mesh(device);
        }
    }
    pub fn set_lamp_aim(&mut self, device: &GPUDevice, arc: f64, target: f64) {
        if self.state.set_lamp_aim(arc, target) {
            self.refresh_light_mesh(device);
        }
    }
    pub fn set_orbit(&mut self, x: f64, y: f64) {
        self.state.set_orbit(x, y);
    }
    pub fn set_framing_viewport(
        &mut self,
        device: &GPUDevice,
        viewport: Option<NormalizedViewport>,
    ) {
        if self.state.set_framing_viewport(viewport) {
            self.refresh_light_mesh(device);
        }
    }
    pub fn resize(&mut self, device: &GPUDevice, output: [f64; 2]) {
        if self.state.resize(output) {
            self.refresh_light_mesh(device);
        }
    }
    pub fn set_light_mesh_layout(&mut self, device: &GPUDevice, layout: LightMeshLayout) {
        if self.state.set_light_mesh_layout(layout) {
            self.light_vertices =
                js::float32_array::new((layout.vertex_count * LIGHT_VERTEX_FLOATS) as f64);
            self.refresh_light_mesh(device);
        }
    }
    fn refresh_light_mesh(&mut self, device: &GPUDevice) {
        let mesh = build_light_mesh(
            &self.state.light_mesh_options(),
            Some(self.light_vertices),
            Some(&mut self.light_vertex_scratch),
        );
        device
            .queue()
            .write_buffer(self.light_buffer, 0, self.light_vertices, None, None);
        self.light_stats = mesh.stats;
    }
    pub fn ensure_wireframe(&mut self, device: &GPUDevice) -> &PrismBuffers {
        self.prism_wireframe.get_or_insert_with(|| {
            create_prism_geometry(device, &format!("{}.prism-wireframe", self.label), true)
        })
    }
    /// The same pending or fulfilled promise is returned on repeated calls.
    /// Both GPU-owning bakes settle before a failure is rethrown.
    pub fn prepare_environment(
        &mut self,
        device: &'static GPUDevice,
    ) -> Result<Promise<()>, String> {
        if let Some(ready) = self.environment_ready {
            return Ok(ready);
        }
        if self.studio_environment.is_none() {
            self.studio_environment = Some(create_environment_texture(
                device,
                &format!("{}.environment-studio", self.label),
                false,
            )?);
        }
        if self.debug_environment_enabled && self.debug_environment.is_none() {
            self.debug_environment = Some(create_environment_texture(
                device,
                &format!("{}.environment-debug", self.label),
                true,
            )?);
        }
        let ready = js::promise(prepare_environments(
            self.studio_environment.as_mut().expect("studio created"),
            self.debug_environment.as_mut(),
            device,
            self.environment_sampler,
        ));
        self.environment_ready = Some(ready);
        Ok(ready)
    }
    pub fn destroy(&mut self) {
        if let Some(studio) = self.studio_environment.take() {
            studio.destroy();
        }
        if let Some(debug) = self.debug_environment.take() {
            debug.destroy();
        }
        self.environment_ready = None;
        self.light_buffer.destroy();
        self.prism.destroy();
        if let Some(wireframe) = self.prism_wireframe.take() {
            wireframe.destroy();
        }
    }
}
async fn prepare_environments(
    studio: &mut EnvironmentTexture,
    debug: Option<&mut EnvironmentTexture>,
    device: &GPUDevice,
    sampler: &GPUSampler,
) {
    let first_failure = Rc::new(RefCell::new(None));
    let mut work = vec![js::promise(prepare_one(
        studio,
        device,
        sampler,
        Rc::clone(&first_failure),
    ))];
    if let Some(debug) = debug {
        work.push(js::promise(prepare_one(
            debug,
            device,
            sampler,
            Rc::clone(&first_failure),
        )));
    }
    js::promise::all_settled(work).await;
    if let Some(error) = first_failure.take() {
        js::throw(error.as_unknown());
    }
}
async fn prepare_one(
    environment: &mut EnvironmentTexture,
    device: &GPUDevice,
    sampler: &GPUSampler,
    first_failure: Rc<RefCell<Option<&'static JsError>>>,
) {
    let error = match js::settle(js::promise(environment.prepare(device, sampler))).await {
        Ok(Ok(())) => return,
        Ok(Err(message)) => js::error::new(Some(&message), None),
        Err(error) => error,
    };
    let mut first = first_failure.borrow_mut();
    if first.is_none() {
        *first = Some(error);
    }
    js::throw(error.as_unknown());
}
