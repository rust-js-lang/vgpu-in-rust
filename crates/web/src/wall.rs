//! Six-vertex wall draw from the light graph, with no vertex buffers or depth.
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUColorTargetState, GPUDevice, GPUFragmentState, GPUPrimitiveState, GPURenderPipeline,
    GPURenderPipelineDescriptor, GPUVertexState,
};

pub async fn wall_pipeline(
    device: &GPUDevice,
    source: &ShaderSource,
    format: &str,
    label: &str,
) -> &'static GPURenderPipeline {
    let module = shader_module(device, source, label);
    device
        .create_render_pipeline_async(GPURenderPipelineDescriptor {
            label: Some(label),
            layout: "auto".into(),
            vertex: GPUVertexState {
                module,
                entry_point: Some("vs_main"),
                constants: None,
                buffers: None,
            },
            primitive: Some(GPUPrimitiveState {
                cull_mode: Some("back"),
                ..Default::default()
            }),
            depth_stencil: None,
            multisample: None,
            fragment: Some(GPUFragmentState {
                module,
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
