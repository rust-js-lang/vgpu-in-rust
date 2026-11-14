//! The dark graph's additive spectral sheet draw, from create-graph.ts.
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUBindGroup, GPUBlendComponent, GPUBlendState, GPUBuffer, GPUColorTargetState, GPUDevice,
    GPUFragmentState, GPUPrimitiveState, GPURenderPassEncoder, GPURenderPipeline,
    GPURenderPipelineDescriptor, GPUVertexAttribute, GPUVertexBufferLayout, GPUVertexState,
};
use vgpu_prism::light_mesh::LIGHT_VERTEX_STRIDE;

pub async fn light_pipeline(
    device: &GPUDevice,
    source: &ShaderSource,
    format: &str,
    label: &str,
) -> &'static GPURenderPipeline {
    let module = shader_module(device, source, label);
    let additive = GPUBlendComponent {
        operation: Some("add"),
        src_factor: Some("one"),
        dst_factor: Some("one"),
    };
    device
        .create_render_pipeline_async(GPURenderPipelineDescriptor {
            label: Some(label),
            layout: "auto".into(),
            vertex: GPUVertexState {
                module,
                entry_point: Some("vs_main"),
                constants: None,
                buffers: Some(&[GPUVertexBufferLayout {
                    array_stride: LIGHT_VERTEX_STRIDE as f64,
                    step_mode: None,
                    attributes: &[
                        GPUVertexAttribute {
                            format: "float32x2",
                            offset: 0.0,
                            shader_location: 0.0,
                        },
                        GPUVertexAttribute {
                            format: "float32",
                            offset: 8.0,
                            shader_location: 3.0,
                        },
                    ],
                }]),
            },
            primitive: Some(GPUPrimitiveState {
                cull_mode: Some("none"),
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
                    blend: Some(GPUBlendState {
                        color: additive,
                        alpha: additive,
                    }),
                    write_mask: None,
                }],
            }),
        })
        .await
}

/// Encodes into the frame's current light target; frame submission stays owned
/// by the graph, so later bloom and glass passes can consume this exact output.
pub fn encode_light(
    pass: &GPURenderPassEncoder,
    pipeline: &GPURenderPipeline,
    scene: &GPUBindGroup,
    vertices: &GPUBuffer,
    first_vertex: usize,
    vertex_count: usize,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(scene), None);
    pass.set_vertex_buffer(0, Some(vertices), None, None);
    pass.draw(vertex_count as f64, None, Some(first_vertex as f64), None);
}
