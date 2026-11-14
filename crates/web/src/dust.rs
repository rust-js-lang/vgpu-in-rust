//! Native instanced dust rendering. Its authored deterministic simulation is WGSL.
use crate::bindings::{NamedResource, ShaderBindings, UniformInput, prepare_shader_bindings};
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUBindGroup, GPUBlendComponent, GPUBlendState, GPUColorTargetState, GPUDevice,
    GPUFragmentState, GPUPrimitiveState, GPURenderPassEncoder, GPURenderPipeline,
    GPURenderPipelineDescriptor, GPUSampler, GPUTextureView, GPUVertexState,
};
use vgpu_prism::dust::{DUST_PARTICLE_COUNT, DustUniforms};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/particles/dust.wgsl#default"]
    safe static DUST: &'static ShaderSource;
}
pub struct DustDraw {
    pub pipeline: &'static GPURenderPipeline,
    pub values: ShaderBindings,
    pub group: &'static GPUBindGroup,
}
pub async fn create_dust_draw(
    device: &GPUDevice,
    params: DustUniforms,
    color: &GPUTextureView,
    light: &GPUTextureView,
    sampler: &GPUSampler,
    format: &str,
    label: &str,
) -> Result<DustDraw, String> {
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
    let values =
        prepare_shader_bindings(device, DUST, &[UniformInput::new("params", params)], label)?;
    let group = match values.bind(
        device,
        pipeline,
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
    ) {
        Ok(group) => group,
        Err(error) => {
            values.destroy();
            return Err(error);
        }
    };
    Ok(DustDraw {
        pipeline,
        values,
        group,
    })
}
impl DustDraw {
    pub fn write(&self, device: &GPUDevice, params: DustUniforms) -> Result<(), String> {
        self.values.write_value(device.queue(), "params", params)
    }
    /// Composited in the same output pass after the cached display copy.
    pub fn encode(&self, pass: &GPURenderPassEncoder) {
        pass.set_pipeline(self.pipeline);
        pass.set_bind_group(0, Some(self.group), None);
        pass.draw(6, Some(DUST_PARTICLE_COUNT as f64), None, None);
    }
    pub fn destroy(&self) {
        self.values.destroy();
    }
}
