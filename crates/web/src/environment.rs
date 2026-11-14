//! Authored studio/debug environment baking and mip filtering from environment/texture.ts.
use crate::bindings::{NamedResource, ShaderBindings, UniformInput, prepare_shader_bindings};
use crate::fullscreen::{encode_fullscreen, fullscreen_pipeline, fullscreen_vertex};
use crate::shader::{ShaderSource, shader_module};
use crate::targets::{RenderTarget, create_render_target, create_texture};
use react::webapi::{
    GPUBindGroup, GPUCommandEncoderDescriptor, GPUDevice, GPURenderPassDescriptor,
    GPURenderPipeline, GPUSampler, GPUSamplerDescriptor, GPUTexelCopyTextureInfo, GPUTexture,
    gpu_texture_usage,
};
use vgpu_prism::uniforms::{ENVIRONMENT_LEVELS, ENVIRONMENT_SIZE};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/environment/environment-bake.wgsl#default"]
    safe static BAKE: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/environment/environment-blur.wgsl#default"]
    safe static BLUR: &'static ShaderSource;
}
const ENVIRONMENT_BLUR_RADIUS: f64 = 1.15;
struct BakeParams {
    debug: f64,
}
struct BlurParams {
    texel: [f64; 2],
    direction: [f64; 2],
    radius: f64,
    #[cfg_attr(rust_js, rust_js::name = "equirect_compensation")]
    equirect_compensation: f64,
}

// These shader parameter records always have an object representation.
unsafe impl js::Defined for BakeParams {}
unsafe impl js::Defined for BlurParams {}

pub struct EnvironmentTexture {
    pub texture: &'static GPUTexture,
    pub prepared: bool,
    label: String,
    bake_bindings: ShaderBindings,
    blur_bindings: Option<ShaderBindings>,
    bake_pipeline: Option<&'static GPURenderPipeline>,
    blur_pipeline: Option<&'static GPURenderPipeline>,
}

pub fn create_environment_sampler(device: &GPUDevice) -> &'static GPUSampler {
    device.create_sampler(Some(GPUSamplerDescriptor {
        min_filter: Some("linear"),
        mag_filter: Some("linear"),
        mipmap_filter: Some("linear"),
        address_mode_u: Some("repeat"),
        address_mode_v: Some("clamp-to-edge"),
        ..Default::default()
    }))
}

pub fn create_environment_texture(
    device: &GPUDevice,
    label: &str,
    debug: bool,
) -> Result<EnvironmentTexture, String> {
    let bake_bindings = prepare_shader_bindings(
        device,
        BAKE,
        &[UniformInput {
            name: "params",
            value: Some(js::unknown_of(BakeParams {
                debug: if debug { 1.0 } else { 0.0 },
            })),
        }],
        &format!("{label}.bake"),
    )?;
    let texture = create_texture(
        device,
        ENVIRONMENT_SIZE,
        "rgba16float",
        1,
        ENVIRONMENT_LEVELS as u32,
        gpu_texture_usage::TEXTURE_BINDING | gpu_texture_usage::COPY_DST,
        &format!("{label}.texture"),
    );
    Ok(EnvironmentTexture {
        texture,
        prepared: false,
        label: label.into(),
        bake_bindings,
        blur_bindings: None,
        bake_pipeline: None,
        blur_pipeline: None,
    })
}

impl EnvironmentTexture {
    pub async fn prepare(
        &mut self,
        device: &GPUDevice,
        sampler: &GPUSampler,
    ) -> Result<(), String> {
        if self.prepared {
            return Ok(());
        }
        let mut source = create_render_target(
            device,
            ENVIRONMENT_SIZE,
            "rgba16float",
            1,
            &format!("{}.texture.level0", self.label),
        );
        if self.bake_pipeline.is_none() || self.blur_pipeline.is_none() {
            let vertex = fullscreen_vertex(device);
            let bake = shader_module(device, BAKE, &format!("{}.bake", self.label));
            let blur = shader_module(device, BLUR, &format!("{}.blur", self.label));
            let compiled = js::settle(js::promise::all(vec![
                js::promise(fullscreen_pipeline(
                    device,
                    vertex,
                    bake,
                    "rgba16float",
                    1,
                    &format!("{}.bake", self.label),
                )),
                js::promise(fullscreen_pipeline(
                    device,
                    vertex,
                    blur,
                    "rgba16float",
                    1,
                    &format!("{}.blur", self.label),
                )),
            ]))
            .await;
            let pipelines = match compiled {
                Ok(pipelines) => pipelines,
                Err(error) => {
                    source.destroy();
                    return Err(error.message());
                }
            };
            self.bake_pipeline = Some(pipelines[0]);
            self.blur_pipeline = Some(pipelines[1]);
        }
        let result = self.bake_mips(device, sampler, &mut source);
        source.destroy();
        if result.is_ok() {
            self.prepared = true;
        }
        result
    }

    fn bake_mips(
        &mut self,
        device: &GPUDevice,
        sampler: &GPUSampler,
        source: &mut RenderTarget,
    ) -> Result<(), String> {
        let bake = self.bake_pipeline.expect("compiled bake pipeline");
        let blur = self.blur_pipeline.expect("compiled blur pipeline");
        let bindings =
            self.bake_bindings
                .bind(device, bake, &[], &format!("{}.bake", self.label))?;
        submit_effect(device, bake, bindings, source);
        copy_into_level(device, source, self.texture, 0, &self.label);
        for level in 1..ENVIRONMENT_LEVELS {
            let size = [
                (ENVIRONMENT_SIZE[0] as u32 >> level).max(1) as f64,
                (ENVIRONMENT_SIZE[1] as u32 >> level).max(1) as f64,
            ];
            let horizontal = create_render_target(
                device,
                size,
                "rgba16float",
                1,
                &format!("{}.texture.blur-h{level}", self.label),
            );
            let vertical = create_render_target(
                device,
                size,
                "rgba16float",
                1,
                &format!("{}.texture.level{level}", self.label),
            );
            let result = (|| {
                let params = BlurParams {
                    texel: [1.0 / size[0], 1.0 / size[1]],
                    direction: [1.0, 0.0],
                    radius: ENVIRONMENT_BLUR_RADIUS,
                    equirect_compensation: 1.0,
                };
                if let Some(bindings) = &self.blur_bindings {
                    bindings.write(device.queue(), "blur", Some(js::unknown_of(params)))?;
                } else {
                    self.blur_bindings = Some(prepare_shader_bindings(
                        device,
                        BLUR,
                        &[UniformInput {
                            name: "blur",
                            value: Some(js::unknown_of(params)),
                        }],
                        &format!("{}.blur", self.label),
                    )?);
                }
                let uniforms = self.blur_bindings.as_ref().expect("blur uniforms prepared");
                let group = uniforms.bind(
                    device,
                    blur,
                    &[
                        NamedResource {
                            name: "src",
                            resource: source.view.into(),
                        },
                        NamedResource {
                            name: "src_samp",
                            resource: sampler.into(),
                        },
                    ],
                    &format!("{}.blur-h{level}", self.label),
                )?;
                submit_effect(device, blur, group, &horizontal);
                uniforms.write(
                    device.queue(),
                    "blur",
                    Some(js::unknown_of(BlurParams {
                        texel: [1.0 / size[0], 1.0 / size[1]],
                        direction: [0.0, 1.0],
                        radius: ENVIRONMENT_BLUR_RADIUS,
                        equirect_compensation: 0.0,
                    })),
                )?;
                let group = uniforms.bind(
                    device,
                    blur,
                    &[
                        NamedResource {
                            name: "src",
                            resource: horizontal.view.into(),
                        },
                        NamedResource {
                            name: "src_samp",
                            resource: sampler.into(),
                        },
                    ],
                    &format!("{}.blur-v{level}", self.label),
                )?;
                submit_effect(device, blur, group, &vertical);
                copy_into_level(device, &vertical, self.texture, level, &self.label);
                Ok(())
            })();
            horizontal.destroy();
            if let Err(error) = result {
                vertical.destroy();
                return Err(error);
            }
            source.destroy();
            *source = vertical;
        }
        Ok(())
    }

    pub fn destroy(&self) {
        self.texture.destroy();
        self.bake_bindings.destroy();
        if let Some(bindings) = &self.blur_bindings {
            bindings.destroy();
        }
    }
}

fn submit_effect(
    device: &GPUDevice,
    pipeline: &GPURenderPipeline,
    bindings: &GPUBindGroup,
    target: &RenderTarget,
) {
    let encoder = device.create_command_encoder(None);
    let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
        label: None,
        color_attachments: &[target.color_attachment(false, Some(&[0.0, 0.0, 0.0, 1.0]))],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        max_draw_count: None,
    });
    encode_fullscreen(pass, pipeline, bindings);
    pass.end();
    device.queue().submit(&[encoder.finish(None)]);
}
fn copy_into_level(
    device: &GPUDevice,
    source: &RenderTarget,
    environment: &GPUTexture,
    level: usize,
    label: &str,
) {
    let encoder = device.create_command_encoder(Some(GPUCommandEncoderDescriptor {
        label: Some(&format!("{label}.texture.copy-level{level}")),
    }));
    encoder.copy_texture_to_texture(
        GPUTexelCopyTextureInfo {
            texture: source.texture,
            mip_level: None,
            origin: None,
            aspect: None,
        },
        GPUTexelCopyTextureInfo {
            texture: environment,
            mip_level: Some(level as f64),
            origin: None,
            aspect: None,
        },
        &[source.size[0], source.size[1], 1.0][..],
    );
    device.queue().submit(&[encoder.finish(None)]);
}
