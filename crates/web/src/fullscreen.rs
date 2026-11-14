//! The fullscreen triangle used by vgpu effects. Shader code remains the
//! original WGSL; WebGPU pipeline and command encoding are Rust.
use react::webapi::{
    GPUBindGroup, GPUColorTargetState, GPUDevice, GPUFragmentState, GPUMultisampleState,
    GPURenderPassEncoder, GPURenderPipeline, GPURenderPipelineDescriptor, GPUShaderModule,
    GPUVertexState,
};

use crate::shader::{ShaderSource, shader_module};

unsafe extern "Rust" {
    #[link_name = "../../../assets/vgpu/fullscreen.wgsl#default"]
    safe static FULLSCREEN_VERTEX: &'static ShaderSource;
}

/// Uses vgpu's original fullscreen stage, including its top-origin UVs.
pub fn fullscreen_vertex(device: &GPUDevice) -> &'static GPUShaderModule {
    shader_module(device, FULLSCREEN_VERTEX, "vgpu.fullscreen.vertex")
}

/// The fullscreen effect pipeline: one triangle, no vertex buffers or depth.
pub async fn fullscreen_pipeline(
    device: &GPUDevice,
    vertex: &GPUShaderModule,
    fragment: &GPUShaderModule,
    format: &str,
    sample_count: u32,
    label: &str,
) -> &'static GPURenderPipeline {
    with_pipeline_descriptor(
        vertex,
        fragment,
        format,
        sample_count,
        label,
        |descriptor| device.create_render_pipeline_async(descriptor),
    )
    .await
}

/// Build the shared descriptor once for the native synchronous and asynchronous
/// compilation paths. The callback consumes stack-borrowed color targets.
fn with_pipeline_descriptor<T>(
    vertex: &GPUShaderModule,
    fragment: &GPUShaderModule,
    format: &str,
    sample_count: u32,
    label: &str,
    create: impl FnOnce(GPURenderPipelineDescriptor<'_>) -> T,
) -> T {
    create(GPURenderPipelineDescriptor {
        label: Some(label),
        layout: "auto".into(),
        vertex: GPUVertexState {
            module: vertex,
            entry_point: Some("vgpu_fullscreen_vs"),
            constants: None,
            buffers: None,
        },
        primitive: None,
        depth_stencil: None,
        multisample: Some(GPUMultisampleState {
            count: Some(f64::from(sample_count)),
            mask: None,
            alpha_to_coverage_enabled: None,
        }),
        fragment: Some(GPUFragmentState {
            module: fragment,
            entry_point: Some("fs_main"),
            constants: None,
            targets: &[GPUColorTargetState {
                format,
                blend: None,
                write_mask: None,
            }],
        }),
    })
}

/// Synchronous compilation, as vgpu's immediate effect draw uses.
pub fn fullscreen_pipeline_sync(
    device: &GPUDevice,
    vertex: &GPUShaderModule,
    fragment: &GPUShaderModule,
    format: &str,
    sample_count: u32,
    label: &str,
) -> &'static GPURenderPipeline {
    with_pipeline_descriptor(
        vertex,
        fragment,
        format,
        sample_count,
        label,
        |descriptor| device.create_render_pipeline(descriptor),
    )
}

/// Encodes one effect into the caller's frame. The frame owns submission.
pub fn encode_fullscreen(
    pass: &GPURenderPassEncoder,
    pipeline: &GPURenderPipeline,
    bindings: &GPUBindGroup,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(bindings), None);
    pass.draw(3, None, None, None);
}
