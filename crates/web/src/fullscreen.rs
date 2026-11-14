//! The fullscreen triangle used by vgpu effects. Shader code remains the
//! original WGSL; WebGPU pipeline and command encoding are Rust.
use react::webapi::{
    GPUBindGroup, GPUColorTargetState, GPUCommandEncoder, GPUDevice, GPUFragmentState,
    GPURenderPassColorAttachment, GPURenderPassDescriptor, GPURenderPipeline,
    GPURenderPipelineDescriptor, GPUShaderModule, GPUTextureView, GPUVertexState,
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
    label: &str,
) -> &'static GPURenderPipeline {
    device
        .create_render_pipeline_async(GPURenderPipelineDescriptor {
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
            multisample: None,
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
        .await
}

/// Encodes one effect into the caller's frame. The frame owns submission.
pub fn encode_fullscreen(
    encoder: &GPUCommandEncoder,
    pipeline: &GPURenderPipeline,
    bindings: &GPUBindGroup,
    output: &GPUTextureView,
    load: bool,
) {
    let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
        label: None,
        color_attachments: &[GPURenderPassColorAttachment {
            view: output.into(),
            depth_slice: None,
            resolve_target: None,
            clear_value: None,
            load_op: if load { "load" } else { "clear" },
            store_op: "store",
        }],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        max_draw_count: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(bindings), None);
    pass.draw(3, None, None, None);
    pass.end();
}
