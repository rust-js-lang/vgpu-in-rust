//! Retained dark bloom and display presentation. WGSL remains unchanged.
use crate::bindings::{BoundPipeline, NamedResource, UniformInput};
use crate::capabilities::bloom_format_for_level;
use crate::fullscreen::{encode_fullscreen, fullscreen_pipeline, fullscreen_vertex};
use crate::resources::Allocations;
use crate::shader::{ShaderSource, shader_module};
use crate::targets::{RenderSignature, RenderTarget};
use react::webapi::{GPUCommandEncoder, GPUDevice, GPURenderPassDescriptor, GPUSampler};
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

/// Compile against the target's exact format and sample count before allocating
/// uniforms. Mesh and particle draws share BoundPipeline's binding ownership.
pub async fn create_effect(
    device: &GPUDevice,
    source: &'static ShaderSource,
    target: RenderSignature<'_>,
    inputs: &[UniformInput<'_>],
    resources: &[NamedResource<'_>],
    label: &str,
) -> Result<BoundPipeline, String> {
    let pipeline = fullscreen_pipeline(
        device,
        fullscreen_vertex(device),
        shader_module(device, source, label),
        target.format,
        target.sample_count,
        label,
    )
    .await;
    BoundPipeline::prepare(device, source, pipeline, inputs, resources, label)
}
pub fn encode_effect(effect: &BoundPipeline, encoder: &GPUCommandEncoder, target: &RenderTarget) {
    let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
        label: Some(&target.label),
        color_attachments: &[target.color_attachment(false, Some(&[0.0, 0.0, 0.0, 1.0]))],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        max_draw_count: None,
    });
    encode_fullscreen(pass, effect.pipeline, effect.group);
    pass.end();
}
pub struct BloomLevel {
    pub horizontal: RenderTarget,
    pub vertical: RenderTarget,
    pub blur_horizontal: BoundPipeline,
    pub blur_vertical: BoundPipeline,
}
pub struct DarkPostprocess {
    pub levels: Vec<BloomLevel>,
    pub presentation: RenderTarget,
    pub extract: BoundPipeline,
    pub downsample: Option<BoundPipeline>,
    pub composite: BoundPipeline,
    pub present: BoundPipeline,
    pub copy: BoundPipeline,
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
        let format = bloom_format_for_level(device.features(), level);
        let horizontal = allocations.target(
            device,
            size,
            format,
            1,
            &format!("{label}.bloom-{level}-horizontal"),
        );
        let vertical = allocations.target(
            device,
            size,
            format,
            1,
            &format!("{label}.bloom-{level}-vertical"),
        );
        let source = if level == 0 || (!low_quality && level == visible_levels) {
            vertical.view
        } else {
            levels[level - 1].vertical.view
        };
        let blur_horizontal = create_effect(
            device,
            if bloom_uses_pairs(level, true) {
                PAIRED
            } else {
                BLUR
            },
            horizontal.signature(),
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
        allocations.retain_bindings(&blur_horizontal.values);
        let blur_vertical = create_effect(
            device,
            PAIRED,
            vertical.signature(),
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
        allocations.retain_bindings(&blur_vertical.values);
        levels.push(BloomLevel {
            horizontal,
            vertical,
            blur_horizontal,
            blur_vertical,
        });
    }
    let extract = create_effect(
        device,
        EXTRACT,
        levels[0].vertical.signature(),
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
    allocations.retain_bindings(&extract.values);
    let downsample = if low_quality {
        None
    } else {
        let target = &levels[visible_levels].vertical;
        Some(
            create_effect(
                device,
                DOWNSAMPLE,
                target.signature(),
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
        allocations.retain_bindings(&effect.values);
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
    let composite = create_effect(
        device,
        if low_quality {
            COMPOSITE_LOW
        } else {
            COMPOSITE
        },
        levels[0].horizontal.signature(),
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
    allocations.retain_bindings(&composite.values);
    let presentation = allocations.target(
        device,
        scene.size,
        output_format,
        1,
        &format!("{label}.retained-dark-presentation"),
    );
    let present = create_effect(
        device,
        PRESENT,
        presentation.signature(),
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
    allocations.retain_bindings(&present.values);
    let copy = create_effect(
        device,
        COPY,
        presentation.signature(),
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
    allocations.retain_bindings(&copy.values);
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
        encode_effect(&self.extract, encoder, &self.levels[0].vertical);
        for level in &self.levels[..self.visible_levels] {
            encode_effect(&level.blur_horizontal, encoder, &level.horizontal);
            encode_effect(&level.blur_vertical, encoder, &level.vertical);
        }
        if let Some(downsample) = &self.downsample {
            let level = &self.levels[self.visible_levels];
            encode_effect(downsample, encoder, &level.vertical);
            encode_effect(&level.blur_horizontal, encoder, &level.horizontal);
            encode_effect(&level.blur_vertical, encoder, &level.vertical);
        }
        encode_effect(&self.composite, encoder, &self.levels[0].horizontal);
        encode_effect(&self.present, encoder, &self.presentation);
    }
    /// Resize target storage and reflected groups while retaining pipelines and
    /// every uniform buffer. Shader layouts are independent of texture sizes.
    pub fn resize(
        &mut self,
        device: &GPUDevice,
        scene: &RenderTarget,
        sampler: &GPUSampler,
    ) -> Result<(), String> {
        let mut previous = None;
        for (index, level) in self.levels.iter_mut().enumerate() {
            let size = bloom_level_size(scene.size, index);
            level.horizontal.resize(device, size);
            level.vertical.resize(device, size);
            let source = if index == 0 || (!self.low_quality && index == self.visible_levels) {
                level.vertical.view
            } else {
                previous.expect("previous bloom level")
            };
            level.blur_horizontal.rebind(
                device,
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
                &level.horizontal.label,
            )?;
            level.blur_vertical.rebind(
                device,
                &[
                    NamedResource {
                        name: "sourceTexture",
                        resource: level.horizontal.view.into(),
                    },
                    NamedResource {
                        name: "sourceSampler",
                        resource: sampler.into(),
                    },
                ],
                &level.vertical.label,
            )?;
            level.blur_horizontal.values.write_value(
                device.queue(),
                "params",
                bloom_blur_uniforms(index, true, size),
            )?;
            level.blur_vertical.values.write_value(
                device.queue(),
                "params",
                bloom_blur_uniforms(index, false, size),
            )?;
            previous = Some(level.vertical.view);
        }
        self.extract.rebind(
            device,
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
            "dark.bloom.extract",
        )?;
        if let Some(downsample) = &mut self.downsample {
            let size = self.levels[self.visible_levels].vertical.size;
            downsample.rebind(
                device,
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
                "dark.particle-light.downsample",
            )?;
            downsample.values.write_value(
                device.queue(),
                "params",
                Downsample {
                    source_texel_size: [1.0 / scene.size[0], 1.0 / scene.size[1]],
                    source_to_target_scale: [scene.size[0] / size[0], scene.size[1] / size[1]],
                },
            )?;
        }
        let mut resources: Vec<NamedResource<'_>> = self
            .levels
            .iter()
            .take(self.visible_levels)
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
        self.composite
            .rebind(device, &resources, "dark.bloom.composite")?;
        self.presentation.resize(device, scene.size);
        self.present.rebind(
            device,
            &[
                NamedResource {
                    name: "sceneTexture",
                    resource: scene.view.into(),
                },
                NamedResource {
                    name: "bloomTexture",
                    resource: self.levels[0].horizontal.view.into(),
                },
                NamedResource {
                    name: "bloomSampler",
                    resource: sampler.into(),
                },
            ],
            "dark.present-cache",
        )?;
        self.copy.rebind(
            device,
            &[NamedResource {
                name: "sourceTexture",
                resource: self.presentation.view.into(),
            }],
            "dark.copy-presentation",
        )?;
        Ok(())
    }
    pub fn set_controls(
        &self,
        device: &GPUDevice,
        controls: &vgpu_prism::controls::PrismControls,
    ) -> Result<(), String> {
        self.extract.values.write_value(
            device.queue(),
            "params",
            Threshold {
                threshold: controls.postprocess.bloom_threshold,
            },
        )?;
        self.composite.values.write_value(
            device.queue(),
            "params",
            Composite {
                radius: bloom_spread(controls.postprocess.bloom_radius, 0.25, 3.0),
                factors: if self.low_quality {
                    BLOOM_LEVEL_FACTORS[..2].to_vec()
                } else {
                    vec![1.0, 0.8, 0.55, 0.0]
                },
            },
        )?;
        self.present.values.write_value(
            device.queue(),
            "params",
            Present {
                bloom_strength: if controls.view != vgpu_prism::controls::PrismView::Glass {
                    0.0
                } else if self.low_quality {
                    0.15
                } else {
                    controls.postprocess.bloom_strength
                },
            },
        )
    }
    pub(crate) fn retain_allocations(&self, allocations: &mut Allocations) {
        for level in &self.levels {
            allocations.retain_target(&level.horizontal);
            allocations.retain_target(&level.vertical);
            allocations.retain_bindings(&level.blur_horizontal.values);
            allocations.retain_bindings(&level.blur_vertical.values);
        }
        allocations.retain_target(&self.presentation);
        for effect in [&self.extract, &self.composite, &self.present, &self.copy] {
            allocations.retain_bindings(&effect.values);
        }
        if let Some(downsample) = &self.downsample {
            allocations.retain_bindings(&downsample.values);
        }
    }
    pub fn encode_output(&self, encoder: &GPUCommandEncoder, output: &RenderTarget) {
        encode_effect(&self.copy, encoder, output);
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
