//! Native instanced dust rendering. Its authored deterministic simulation is WGSL.
use crate::bindings::{BoundPipeline, NamedResource, UniformInput};
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUBlendComponent, GPUBlendState, GPUColorTargetState, GPUDevice, GPUFragmentState,
    GPUPrimitiveState, GPURenderPassEncoder, GPURenderPipelineDescriptor, GPUSampler,
    GPUTextureView, GPUVertexState,
};
use vgpu_prism::dust::{DUST_PARTICLE_COUNT, DustUniforms};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/particles/dust.wgsl#default"]
    safe static DUST: &'static ShaderSource;
}
pub async fn create_dust_draw(
    device: &GPUDevice,
    params: DustUniforms,
    color: &GPUTextureView,
    light: &GPUTextureView,
    sampler: &GPUSampler,
    format: &str,
    label: &str,
) -> Result<BoundPipeline, String> {
    let module = shader_module(device, DUST, label);
    let additive = GPUBlendComponent {
        operation: Some("add"),
        src_factor: Some("one"),
        dst_factor: Some("one"),
    };
    let pipeline = device
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
        .await;
    BoundPipeline::prepare(
        device,
        DUST,
        pipeline,
        &[UniformInput::new("params", params)],
        &[
            NamedResource {
                name: "colorTexture",
                resource: color.into(),
            },
            NamedResource {
                name: "lightTexture",
                resource: light.into(),
            },
            NamedResource {
                name: "lightSampler",
                resource: sampler.into(),
            },
        ],
        label,
    )
}
/// Composited in the same output pass after the cached display copy.
pub fn encode_dust(draw: &BoundPipeline, pass: &GPURenderPassEncoder) {
    pass.set_pipeline(draw.pipeline);
    pass.set_bind_group(0, Some(draw.group), None);
    pass.draw(6, Some(DUST_PARTICLE_COUNT as f64), None, None);
}
