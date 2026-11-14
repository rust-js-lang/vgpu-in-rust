//! The light theme's indexed wall-space cast shadow. No blur target is needed.
use crate::buffers::create_vertex_buffer;
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUBindGroup, GPUBlendComponent, GPUBlendState, GPUBuffer, GPUBufferDescriptor,
    GPUColorTargetState, GPUDevice, GPUFragmentState, GPUPrimitiveState, GPURenderPassEncoder,
    GPURenderPipeline, GPURenderPipelineDescriptor, GPUVertexAttribute, GPUVertexBufferLayout,
    GPUVertexState, gpu_buffer_usage,
};
use vgpu_prism::light_uniforms::LIGHT_SHADOW_MESH;
use vgpu_prism::shadow_mesh::build_shadow_mesh;
use vgpu_prism::types::PRISM_TRIANGLE;

pub struct ShadowBuffers {
    pub vertices: &'static GPUBuffer,
    pub indices: &'static GPUBuffer,
    pub index_count: usize,
}

pub fn create_shadow_geometry(device: &GPUDevice, label: &str) -> ShadowBuffers {
    let mesh = build_shadow_mesh(*PRISM_TRIANGLE, LIGHT_SHADOW_MESH)
        .expect("The prism cast-shadow hull has three points.");
    let vertices = create_vertex_buffer(
        device,
        js::float32_array::from(&mesh.vertices),
        &format!("{label}.vertices"),
        None,
    );
    let data = js::uint32_array::from(&mesh.indices);
    let indices = device.create_buffer(GPUBufferDescriptor {
        label: Some(&format!("{label}.indices")),
        size: data.byte_length(),
        usage: f64::from(gpu_buffer_usage::INDEX | gpu_buffer_usage::COPY_DST),
        mapped_at_creation: None,
    });
    device.queue().write_buffer(indices, 0, data, None, None);
    ShadowBuffers {
        vertices,
        indices,
        index_count: mesh.indices.len(),
    }
}
impl ShadowBuffers {
    pub fn destroy(&self) {
        self.vertices.destroy();
        self.indices.destroy();
    }
}

pub async fn shadow_pipeline(
    device: &GPUDevice,
    source: &ShaderSource,
    format: &str,
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
                    array_stride: 16.0,
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
                            shader_location: 1.0,
                        },
                        GPUVertexAttribute {
                            format: "float32",
                            offset: 12.0,
                            shader_location: 2.0,
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
                        color: over,
                        alpha: over,
                    }),
                    write_mask: None,
                }],
            }),
        })
        .await
}

pub fn encode_shadow(
    pass: &GPURenderPassEncoder,
    pipeline: &GPURenderPipeline,
    group: &GPUBindGroup,
    mesh: &ShadowBuffers,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(group), None);
    pass.set_vertex_buffer(0, Some(mesh.vertices), None, None);
    pass.set_index_buffer(mesh.indices, "uint32", None, None);
    pass.draw_indexed(mesh.index_count as f64, None, None, None, None);
}
