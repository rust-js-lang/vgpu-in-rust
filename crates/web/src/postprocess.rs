//! Retained dark bloom and display presentation. WGSL remains unchanged.
use crate::bindings::{NamedResource, ShaderBindings, UniformInput, prepare_shader_bindings};
use crate::fullscreen::{encode_fullscreen, fullscreen_pipeline, fullscreen_vertex};
use crate::shader::{ShaderSource, shader_module};
use crate::targets::{RenderTarget, create_render_target};
use react::webapi::{
    GPUBindGroup, GPUBuffer, GPUCommandEncoder, GPUDevice, GPURenderPassDescriptor,
    GPURenderPipeline, GPUSampler, GPUTexture,
};
use vgpu_prism::bloom::{
    BLOOM_LEVEL_FACTORS, bloom_blur_uniforms, bloom_level_size, bloom_spread, bloom_uses_pairs,
};
use vgpu_prism::{controls::PrismTheme, presentation::presentation_reveal_uniforms};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/bloom/bloom-extract.wgsl#default"]
    safe static EXTRACT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/bloom/bloom-blur.wgsl#default"]
    safe static BLUR: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/bloom/bloom-blur-paired.wgsl#default"]
    safe static PAIRED: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/bloom/bloom-composite.wgsl#default"]
    safe static COMPOSITE: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/bloom/bloom-composite-low.wgsl#default"]
    safe static COMPOSITE_LOW: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/particles/particle-light-downsample.wgsl#default"]
    safe static DOWNSAMPLE: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/presentation/present.wgsl#default"]
    safe static PRESENT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/presentation/copy-presentation.wgsl#default"]
    safe static COPY: &'static ShaderSource;
}

struct Threshold {
    threshold: f64,
}
struct Composite {
    radius: f64,
    factors: Vec<f64>,
}
struct Present {
    bloom_strength: f64,
}

struct Downsample {
    source_texel_size: [f64; 2],
    source_to_target_scale: [f64; 2],
}

pub struct Effect {
    pub pipeline: &'static GPURenderPipeline,
    pub values: ShaderBindings,
    pub group: &'static GPUBindGroup,
}
impl Effect {
    pub async fn create(
        device: &GPUDevice,
        source: &'static ShaderSource,
        format: &str,
        inputs: &[UniformInput<'_>],
        resources: &[NamedResource<'_>],
        label: &str,
    ) -> Result<Self, String> {
        // Compile before allocating retained buffers, so rejected pipelines do
        // not leak a half-prepared effect's resources.
        let pipeline = fullscreen_pipeline(
            device,
            fullscreen_vertex(device),
            shader_module(device, source, label),
            format,
            1,
            label,
        )
        .await;
        let values = prepare_shader_bindings(device, source, inputs, label)?;
        let group = match values.bind(device, pipeline, resources, label) {
            Ok(group) => group,
            Err(error) => {
                values.destroy();
                return Err(error);
            }
        };
        Ok(Self {
            pipeline,
            values,
            group,
        })
    }
    pub fn encode(&self, encoder: &GPUCommandEncoder, target: &RenderTarget) {
        let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
            label: Some(&target.label),
            color_attachments: &[target.color_attachment(false, Some(&[0.0, 0.0, 0.0, 1.0]))],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            max_draw_count: None,
        });
        encode_fullscreen(pass, self.pipeline, self.group);
        pass.end();
    }
    pub fn destroy(&self) {
        self.values.destroy();
    }
}
pub struct BloomLevel {
    pub horizontal: RenderTarget,
    pub vertical: RenderTarget,
    pub blur_horizontal: Effect,
    pub blur_vertical: Effect,
}
pub struct DarkPostprocess {
    pub levels: Vec<BloomLevel>,
    pub presentation: RenderTarget,
    pub extract: Effect,
    pub downsample: Option<Effect>,
    pub composite: Effect,
    pub present: Effect,
    pub copy: Effect,
    pub visible_levels: usize,
    pub low_quality: bool,
}

pub struct PostprocessSettings<'a> {
    pub output_format: &'a str,
    pub low_quality: bool,
    pub threshold: f64,
    pub radius: f64,
    pub strength: f64,
    pub label: &'a str,
}
#[derive(Default)]
struct Allocations {
    textures: Vec<&'static GPUTexture>,
    buffers: Vec<&'static GPUBuffer>,
}
impl Allocations {
    fn target(
        &mut self,
        device: &GPUDevice,
        size: [f64; 2],
        format: &str,
        label: &str,
    ) -> RenderTarget {
        let target = create_render_target(device, size, format, 1, label);
        self.textures.push(target.texture);
        target
    }
    fn retain_effect(&mut self, effect: &Effect) {
        for uniform in &effect.values.uniforms {
            self.buffers.push(uniform.resource.buffer);
        }
    }
    fn destroy(&self) {
        for texture in &self.textures {
            texture.destroy();
        }
        for buffer in &self.buffers {
            buffer.destroy();
        }
    }
}

/// A rejected native pipeline or a binding validation error releases all
/// resources already made by this construction attempt.
pub async fn create_dark_postprocess(
    device: &GPUDevice,
    scene: &RenderTarget,
    sampler: &GPUSampler,
    settings: PostprocessSettings<'_>,
) -> Result<DarkPostprocess, String> {
    let mut allocations = Allocations::default();
    let result = match js::settle(js::promise(build_dark_postprocess(
        device,
        scene,
        sampler,
        settings,
        &mut allocations,
    )))
    .await
    {
        Ok(result) => result,
        Err(error) => Err(error.message()),
    };
    if result.is_err() {
        allocations.destroy();
    }
    result
}
async fn build_dark_postprocess(
    device: &GPUDevice,
    scene: &RenderTarget,
    sampler: &GPUSampler,
    settings: PostprocessSettings<'_>,
    allocations: &mut Allocations,
) -> Result<DarkPostprocess, String> {
    let PostprocessSettings {
        output_format,
        low_quality,
        threshold,
        radius,
        strength,
        label,
    } = settings;
    let count = if low_quality { 2 } else { 4 };
    let visible_levels = if low_quality { 2 } else { 3 };
    let mut levels: Vec<BloomLevel> = Vec::new();
    for level in 0..count {
        let size = bloom_level_size(scene.size, level);
        let format = if level < 3 && device.features().has("rg11b10ufloat-renderable") {
            "rg11b10ufloat"
        } else {
            "rgba16float"
        };
        let horizontal = allocations.target(
            device,
            size,
            format,
            &format!("{label}.bloom-{level}-horizontal"),
        );
        let vertical = allocations.target(
            device,
            size,
            format,
            &format!("{label}.bloom-{level}-vertical"),
        );
        let source = if level == 0 || (!low_quality && level == visible_levels) {
            vertical.view
        } else {
            levels[level - 1].vertical.view
        };
        let blur_horizontal = Effect::create(
            device,
            if bloom_uses_pairs(level, true) {
                PAIRED
            } else {
                BLUR
            },
            format,
            &[UniformInput::new(
                "params",
                bloom_blur_uniforms(level, true, size),
            )],
            &[
                NamedResource {
                    name: "sourceTexture",
                    resource: source.into(),
                },
                NamedResource {
                    name: "sourceSampler",
                    resource: sampler.into(),
                },
            ],
            &format!("{label}.bloom-{level}-horizontal"),
        )
        .await?;
        allocations.retain_effect(&blur_horizontal);
        let blur_vertical = Effect::create(
            device,
            PAIRED,
            format,
            &[UniformInput::new(
                "params",
                bloom_blur_uniforms(level, false, size),
            )],
            &[
                NamedResource {
                    name: "sourceTexture",
                    resource: horizontal.view.into(),
                },
                NamedResource {
                    name: "sourceSampler",
                    resource: sampler.into(),
                },
            ],
            &format!("{label}.bloom-{level}-vertical"),
        )
        .await?;
        allocations.retain_effect(&blur_vertical);
        levels.push(BloomLevel {
            horizontal,
            vertical,
            blur_horizontal,
            blur_vertical,
        });
    }
    let extract = Effect::create(
        device,
        EXTRACT,
        &levels[0].vertical.format,
        &[UniformInput::new("params", Threshold { threshold })],
        &[
            NamedResource {
                name: "sourceTexture",
                resource: scene.view.into(),
            },
            NamedResource {
                name: "sourceSampler",
                resource: sampler.into(),
            },
        ],
        &format!("{label}.bloom-extract"),
    )
    .await?;
    allocations.retain_effect(&extract);
    let downsample = if low_quality {
        None
    } else {
        let target = &levels[visible_levels].vertical;
        Some(
            Effect::create(
                device,
                DOWNSAMPLE,
                &target.format,
                &[UniformInput::new(
                    "params",
                    Downsample {
                        source_texel_size: [1.0 / scene.size[0], 1.0 / scene.size[1]],
                        source_to_target_scale: [
                            scene.size[0] / target.size[0],
                            scene.size[1] / target.size[1],
                        ],
                    },
                )],
                &[
                    NamedResource {
                        name: "sourceTexture",
                        resource: scene.view.into(),
                    },
                    NamedResource {
                        name: "sourceSampler",
                        resource: sampler.into(),
                    },
                ],
                &format!("{label}.particle-light-downsample"),
            )
            .await?,
        )
    };
    if let Some(effect) = &downsample {
        allocations.retain_effect(effect);
    }
    let mut resources: Vec<NamedResource<'_>> = levels
        .iter()
        .take(visible_levels)
        .enumerate()
        .map(|(index, level)| NamedResource {
            name: ["level0Texture", "level1Texture", "level2Texture"][index],
            resource: level.vertical.view.into(),
        })
        .collect();
    resources.push(NamedResource {
        name: "levelSampler",
        resource: sampler.into(),
    });
    let factors = if low_quality {
        BLOOM_LEVEL_FACTORS[..2].to_vec()
    } else {
        vec![1.0, 0.8, 0.55, 0.0]
    };
    let composite = Effect::create(
        device,
        if low_quality {
            COMPOSITE_LOW
        } else {
            COMPOSITE
        },
        &levels[0].horizontal.format,
        &[UniformInput::new(
            "params",
            Composite {
                radius: bloom_spread(radius, 0.25, 3.0),
                factors,
            },
        )],
        &resources,
        &format!("{label}.bloom-composite"),
    )
    .await?;
    allocations.retain_effect(&composite);
    let presentation = allocations.target(
        device,
        scene.size,
        output_format,
        &format!("{label}.retained-dark-presentation"),
    );
    let present = Effect::create(
        device,
        PRESENT,
        output_format,
        &[UniformInput::new(
            "params",
            Present {
                bloom_strength: if low_quality { 0.15 } else { strength },
            },
        )],
        &[
            NamedResource {
                name: "sceneTexture",
                resource: scene.view.into(),
            },
            NamedResource {
                name: "bloomTexture",
                resource: levels[0].horizontal.view.into(),
            },
            NamedResource {
                name: "bloomSampler",
                resource: sampler.into(),
            },
        ],
        &format!("{label}.present"),
    )
    .await?;
    allocations.retain_effect(&present);
    let copy = Effect::create(
        device,
        COPY,
        output_format,
        &[UniformInput::new(
            "params",
            presentation_reveal_uniforms(PrismTheme::Dark, 1.0),
        )],
        &[NamedResource {
            name: "sourceTexture",
            resource: presentation.view.into(),
        }],
        &format!("{label}.copy-presentation"),
    )
    .await?;
    allocations.retain_effect(&copy);
    Ok(DarkPostprocess {
        levels,
        presentation,
        extract,
        downsample,
        composite,
        present,
        copy,
        visible_levels,
        low_quality,
    })
}

impl DarkPostprocess {
    /// The graph owns a single command buffer. Each retained effect has its own
    /// uniform buffer; no queue write can overwrite an earlier encoded pass.
    pub fn encode_scene(&self, encoder: &GPUCommandEncoder) {
        self.extract.encode(encoder, &self.levels[0].vertical);
        for level in &self.levels[..self.visible_levels] {
            level.blur_horizontal.encode(encoder, &level.horizontal);
            level.blur_vertical.encode(encoder, &level.vertical);
        }
        if let Some(downsample) = &self.downsample {
            let level = &self.levels[self.visible_levels];
            downsample.encode(encoder, &level.vertical);
            level.blur_horizontal.encode(encoder, &level.horizontal);
            level.blur_vertical.encode(encoder, &level.vertical);
        }
        self.composite.encode(encoder, &self.levels[0].horizontal);
        self.present.encode(encoder, &self.presentation);
    }
    pub fn encode_output(&self, encoder: &GPUCommandEncoder, output: &RenderTarget) {
        self.copy.encode(encoder, output);
    }
    pub fn set_reveal(&self, device: &GPUDevice, progress: f64) -> Result<(), String> {
        self.copy.values.write_value(
            device.queue(),
            "params",
            presentation_reveal_uniforms(PrismTheme::Dark, progress),
        )
    }
    pub fn destroy(&self) {
        for level in &self.levels {
            level.horizontal.destroy();
            level.vertical.destroy();
            level.blur_horizontal.destroy();
            level.blur_vertical.destroy();
        }
        self.presentation.destroy();
        self.extract.destroy();
        self.composite.destroy();
        self.present.destroy();
        self.copy.destroy();
        if let Some(downsample) = &self.downsample {
            downsample.destroy();
        }
    }
}
