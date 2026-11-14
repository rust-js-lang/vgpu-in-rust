//! Lazy prism edges and spectral triangle topology from shared wireframe shaders.
use crate::bindings::{BoundPipeline, NamedResource, UniformInput};
use crate::shader::{ShaderSource, shader_module};
use crate::targets::RenderTarget;
use react::webapi::{
    GPUBlendComponent, GPUBlendState, GPUColorTargetState, GPUDevice, GPUFragmentState,
    GPUMultisampleState, GPUPrimitiveState, GPURenderPipelineDescriptor, GPUVertexAttribute,
    GPUVertexBufferLayout, GPUVertexState,
};
use vgpu_prism::{light_mesh::LIGHT_VERTEX_STRIDE, prism_mesh::PRISM_VERTEX_STRIDE};
unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/shared/wireframe/wireframe.wgsl#default"]
    safe static PRISM: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/spectral/light-wireframe.wgsl#default"]
    safe static LIGHT: &'static ShaderSource;
}
#[derive(Clone, Copy, PartialEq)]
pub enum WireframeKind {
    Prism,
    Light,
}
pub async fn create_wireframe_draw(
    device: &GPUDevice,
    kind: WireframeKind,
    target: &RenderTarget,
    inputs: &[UniformInput<'_>],
    label: &str,
) -> Result<BoundPipeline, String> {
    let source = if kind == WireframeKind::Prism {
        PRISM
    } else {
        LIGHT
    };
    let module = shader_module(device, source, label);
    let attributes = if kind == WireframeKind::Prism {
        vec![
            GPUVertexAttribute {
                shader_location: 0.0,
                offset: 0.0,
                format: "float32x3",
            },
            GPUVertexAttribute {
                shader_location: 1.0,
                offset: 12.0,
                format: "float32x3",
            },
        ]
    } else {
        vec![
            GPUVertexAttribute {
                shader_location: 0.0,
                offset: 0.0,
                format: "float32x2",
            },
            GPUVertexAttribute {
                shader_location: 3.0,
                offset: 8.0,
                format: "float32",
            },
        ]
    };
    let blend = GPUBlendComponent {
        operation: Some("add"),
        src_factor: Some("one"),
        dst_factor: Some("one-minus-src-alpha"),
    };
    let pipeline = device
        .create_render_pipeline_async(GPURenderPipelineDescriptor {
            label: Some(label),
            layout: "auto".into(),
            vertex: GPUVertexState {
                module,
                entry_point: Some("vs_main"),
                constants: None,
                buffers: Some(&[GPUVertexBufferLayout {
                    array_stride: if kind == WireframeKind::Prism {
                        PRISM_VERTEX_STRIDE as f64
                    } else {
                        LIGHT_VERTEX_STRIDE as f64
                    },
                    step_mode: None,
                    attributes: &attributes,
                }]),
            },
            primitive: Some(GPUPrimitiveState {
                topology: Some(if kind == WireframeKind::Prism {
                    "line-list"
                } else {
                    "triangle-list"
                }),
                cull_mode: Some("none"),
                ..Default::default()
            }),
            depth_stencil: None,
            multisample: Some(GPUMultisampleState {
                count: Some(f64::from(target.sample_count)),
                mask: None,
                alpha_to_coverage_enabled: None,
            }),
            fragment: Some(GPUFragmentState {
                module,
                entry_point: Some("fs_main"),
                constants: None,
                targets: &[GPUColorTargetState {
                    format: &target.format,
                    blend: Some(GPUBlendState {
                        color: blend,
                        alpha: blend,
                    }),
                    write_mask: None,
                }],
            }),
        })
        .await;
    let resources: [NamedResource<'_>; 0] = [];
    BoundPipeline::prepare(device, source, pipeline, inputs, &resources, label)
}
