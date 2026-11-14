//! Both rasterized glass interfaces from pipelines/dark/create-graph.ts.
use crate::geometry::PrismBuffers;
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUBindGroup, GPUBlendComponent, GPUBlendState, GPUColorTargetState, GPUDevice,
    GPUFragmentState, GPUMultisampleState, GPUPrimitiveState, GPURenderPassEncoder,
    GPURenderPipeline, GPURenderPipelineDescriptor, GPUVertexAttribute, GPUVertexBufferLayout,
    GPUVertexState,
};
use vgpu_prism::prism_mesh::PRISM_VERTEX_STRIDE;

pub async fn glass_pipeline(
    device: &GPUDevice,
    source: &ShaderSource,
    format: &str,
    back_face: bool,
    sample_count: u32,
    label: &str,
) -> &'static GPURenderPipeline {
    let module = shader_module(device, source, label);
    let over = GPUBlendComponent {
        operation: Some("add"),
        src_factor: Some("one"),
        dst_factor: Some("one-minus-src-alpha"),
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
                    array_stride: PRISM_VERTEX_STRIDE as f64,
                    step_mode: None,
                    attributes: &[
                        GPUVertexAttribute {
                            format: "float32x3",
                            offset: 0.0,
                            shader_location: 0.0,
                        },
                        GPUVertexAttribute {
                            format: "float32x3",
                            offset: 12.0,
                            shader_location: 1.0,
                        },
                    ],
                }]),
            },
            primitive: Some(GPUPrimitiveState {
                cull_mode: Some(if back_face { "front" } else { "back" }),
                ..Default::default()
            }),
            depth_stencil: None,
            multisample: Some(GPUMultisampleState {
                count: Some(f64::from(sample_count)),
                mask: None,
                alpha_to_coverage_enabled: None,
            }),
            fragment: Some(GPUFragmentState {
                module,
                entry_point: Some("fs_main"),
                constants: None,
                targets: &[GPUColorTargetState {
                    format,
                    blend: back_face.then_some(GPUBlendState {
                        color: over,
                        alpha: over,
                    }),
                    write_mask: None,
                }],
            }),
        })
        .await
}

/// The graph supplies its load/resolve target so background, back interface,
/// internal light and front interface keep their original composition order.
pub fn encode_glass(
    pass: &GPURenderPassEncoder,
    pipeline: &GPURenderPipeline,
    bindings: &GPUBindGroup,
    mesh: &PrismBuffers,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(bindings), None);
    pass.set_vertex_buffer(0, Some(mesh.vertices), None, None);
    pass.set_index_buffer(mesh.indices, "uint16", None, None);
    pass.draw_indexed(mesh.index_count as f64, None, None, None, None);
}
