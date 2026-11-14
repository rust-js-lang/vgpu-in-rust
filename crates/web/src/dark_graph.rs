//! Complete retained dark production composition from pipelines/dark.
use crate::bindings::{BoundPipeline, NamedResource, UniformInput};
use crate::dust::{create_dust_draw, encode_dust};
use crate::fullscreen::encode_fullscreen;
use crate::geometry::encode_prism;
use crate::glass_pass::glass_pipeline;
use crate::postprocess::{
    DarkPostprocess, PostprocessSettings, create_dark_postprocess, create_effect,
};
use crate::resources::Allocations;
use crate::runtime::PrismRuntime;
use crate::shader::ShaderSource;
use crate::spectral_pass::{encode_light, light_pipeline};
use crate::targets::RenderTarget;
use crate::wireframe::{WireframeKind, create_wireframe_draw};
use react::webapi::{
    GPUDevice, GPURenderBundle, GPURenderBundleEncoderDescriptor, GPURenderPassColorAttachment,
    GPURenderPassDescriptor, GPUTextureView,
};
use std::rc::Rc;
use vgpu_prism::color::dark_wall_clear;
use vgpu_prism::controls::{PrismTheme, PrismView};
use vgpu_prism::dust::dust_uniforms;
use vgpu_prism::light_mesh::light_mesh_layout;
use vgpu_prism::uniforms::WireframeUniforms;
use vgpu_prism::uniforms::{glass_uniforms, scene_uniforms};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/dark/passes/light/light.wgsl#default"]
    safe static LIGHT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/glass/glass-back.wgsl#default"]
    safe static BACK: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/glass/glass.wgsl#default"]
    safe static FRONT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/presentation/copy-linear.wgsl#default"]
    safe static COPY: &'static ShaderSource;
}
pub struct DarkSettings<'a> {
    pub output_format: &'a str,
    pub low_quality: bool,
    pub sample_count: u32,
}
#[derive(Clone, Copy)]
pub struct DarkFrame {
    pub time: f64,
    pub update_scene: bool,
    pub reveal_progress: f64,
    pub beam_width_reveal: f64,
}
impl Default for DarkFrame {
    fn default() -> Self {
        Self {
            time: 0.0,
            update_scene: true,
            reveal_progress: 1.0,
            beam_width_reveal: 1.0,
        }
    }
}
pub struct DarkGraph {
    pub background: RenderTarget,
    pub scene: RenderTarget,
    pub postprocess: DarkPostprocess,
    pub light: BoundPipeline,
    pub back: BoundPipeline,
    pub front: BoundPipeline,
    pub copy: BoundPipeline,
    pub dust: BoundPipeline,
    pub backdrop_bundle: &'static GPURenderBundle,
    pub studio_view: &'static GPUTextureView,
    pub debug_view: &'static GPUTextureView,
    pub presentation_valid: bool,
    pub bound_reveal_progress: f64,
    pub wireframe: Option<BoundPipeline>,
    pub light_wireframe: Option<BoundPipeline>,
}
pub async fn create_dark_graph(
    device: &'static GPUDevice,
    runtime: &mut PrismRuntime,
    settings: DarkSettings<'_>,
) -> Result<DarkGraph, String> {
    runtime.set_light_mesh_layout(
        device,
        if settings.low_quality {
            light_mesh_layout(64, 12)
        } else {
            light_mesh_layout(128, 24)
        },
    );
    if let Err(error) = js::settle(runtime.prepare_environment(device)?).await {
        return Err(error.message());
    }
    let mut allocations = Allocations::default();
    let result = match js::settle(js::promise(build_dark_graph(
        device,
        runtime,
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
    let mut graph = result?;
    let prepared = match js::settle(js::promise(graph.prepare_wireframes(device, runtime))).await {
        Ok(result) => result,
        Err(error) => Err(error.message()),
    };
    if let Err(error) = prepared {
        graph.destroy();
        return Err(error);
    }
    Ok(graph)
}
async fn build_dark_graph(
    device: &GPUDevice,
    runtime: &PrismRuntime,
    settings: DarkSettings<'_>,
    allocations: &mut Allocations,
) -> Result<DarkGraph, String> {
    let label = &runtime.label;
    let state = &runtime.state;
    let background = allocations.target(
        device,
        state.output_size,
        "rgba16float",
        1,
        &format!("{label}.pass-a-back-and-light"),
    );
    let scene = allocations.target(
        device,
        state.output_size,
        "rgba16float",
        settings.sample_count,
        &format!("{label}.pass-b-front-glass"),
    );
    let pipelines = js::promise::all(vec![
        js::promise(light_pipeline(
            device,
            LIGHT,
            "rgba16float",
            &format!("{label}.light"),
        )),
        js::promise(glass_pipeline(
            device,
            BACK,
            "rgba16float",
            true,
            1,
            &format!("{label}.glass-back"),
        )),
        js::promise(glass_pipeline(
            device,
            FRONT,
            "rgba16float",
            false,
            settings.sample_count,
            &format!("{label}.glass-front"),
        )),
    ])
    .await;
    let light = BoundPipeline::prepare(
        device,
        LIGHT,
        pipelines[0],
        &[UniformInput::new(
            "scene",
            scene_uniforms(
                &state.view,
                state.wall_extent(),
                state.light_mesh_options().light,
                &state.controls,
                &state.light_mesh_layout,
                1.0,
            ),
        )],
        &[],
        &format!("{label}.light"),
    )?;
    allocations.retain_bindings(&light.values);
    let studio = runtime
        .studio_environment
        .as_ref()
        .expect("environment prepared");
    let studio_view = studio.texture.create_view(None);
    let debug_view = runtime
        .debug_environment
        .as_ref()
        .map(|debug| debug.texture.create_view(None))
        .unwrap_or(studio_view);
    let shared = [
        NamedResource {
            name: "studioEnvironment",
            resource: studio_view.into(),
        },
        NamedResource {
            name: "debugEnvironment",
            resource: debug_view.into(),
        },
        NamedResource {
            name: "environmentSampler",
            resource: runtime.environment_sampler.into(),
        },
    ];
    let back = BoundPipeline::prepare(
        device,
        BACK,
        pipelines[1],
        &[UniformInput::new(
            "params",
            glass_uniforms(
                &state.view,
                &state.controls,
                PrismTheme::Dark,
                runtime.debug_environment_enabled,
            ),
        )],
        &shared,
        &format!("{label}.glass-back"),
    )?;
    allocations.retain_bindings(&back.values);
    let mut front_resources = vec![
        NamedResource {
            name: "sceneTexture",
            resource: background.view.into(),
        },
        NamedResource {
            name: "sceneSampler",
            resource: runtime.scene_sampler.into(),
        },
    ];
    front_resources.extend(shared);
    let front = BoundPipeline::prepare(
        device,
        FRONT,
        pipelines[2],
        &[UniformInput::new(
            "params",
            glass_uniforms(
                &state.view,
                &state.controls,
                PrismTheme::Dark,
                runtime.debug_environment_enabled,
            ),
        )],
        &front_resources,
        &format!("{label}.glass-front"),
    )?;
    allocations.retain_bindings(&front.values);
    let copy = create_effect(
        device,
        COPY,
        &scene,
        &[],
        &[NamedResource {
            name: "sceneTexture",
            resource: background.view.into(),
        }],
        &format!("{label}.pass-b-copy-a"),
    )
    .await?;
    allocations.retain_bindings(&copy.values);
    let postprocess = create_dark_postprocess(
        device,
        &scene,
        runtime.scene_sampler,
        PostprocessSettings {
            output_format: settings.output_format,
            low_quality: settings.low_quality,
            threshold: state.controls.postprocess.bloom_threshold,
            radius: state.controls.postprocess.bloom_radius,
            strength: state.controls.postprocess.bloom_strength,
            label,
        },
    )
    .await?;
    postprocess.retain_allocations(allocations);
    let particle_level = if settings.low_quality {
        postprocess.visible_levels - 1
    } else {
        postprocess.visible_levels
    };
    let dust = create_dust_draw(
        device,
        dust_uniforms(
            &state.view,
            state.wall_extent(),
            state.output_size,
            state.camera_distance,
            0.0,
            1.0,
        ),
        postprocess.levels[1].vertical.view,
        postprocess.levels[particle_level].vertical.view,
        runtime.scene_sampler,
        settings.output_format,
        &format!("{label}.dust"),
    )
    .await?;
    allocations.retain_bindings(&dust.values);
    let bundle = device.create_render_bundle_encoder(GPURenderBundleEncoderDescriptor {
        label: Some(&format!("{label}.dark-backdrop")),
        color_formats: &["rgba16float".into()],
        depth_stencil_format: None,
        sample_count: Some(1.0),
        depth_read_only: None,
        stencil_read_only: None,
    });
    let layout = &state.light_mesh_layout;
    bundle.set_pipeline(light.pipeline);
    bundle.set_bind_group(0, Some(light.group), None);
    bundle.set_vertex_buffer(0, Some(runtime.light_buffer), None, None);
    bundle.draw(layout.white_vertices as f64, None, Some(0.0), None);
    bundle.draw(
        layout.outgoing_vertices as f64,
        None,
        Some(layout.outgoing_first_vertex as f64),
        None,
    );
    bundle.set_pipeline(back.pipeline);
    bundle.set_bind_group(0, Some(back.group), None);
    bundle.set_vertex_buffer(0, Some(runtime.prism.vertices), None, None);
    bundle.set_index_buffer(runtime.prism.indices, "uint16", None, None);
    bundle.draw_indexed(runtime.prism.index_count as f64, None, None, None, None);
    bundle.set_pipeline(light.pipeline);
    bundle.set_bind_group(0, Some(light.group), None);
    bundle.set_vertex_buffer(0, Some(runtime.light_buffer), None, None);
    bundle.draw(
        layout.internal_vertices as f64,
        None,
        Some(layout.internal_first_vertex as f64),
        None,
    );
    Ok(DarkGraph {
        background,
        scene,
        postprocess,
        light,
        back,
        front,
        copy,
        dust,
        backdrop_bundle: bundle.finish(None),
        studio_view,
        debug_view,
        presentation_valid: false,
        bound_reveal_progress: 1.0,
        wireframe: None,
        light_wireframe: None,
    })
}
impl DarkGraph {
    pub async fn prepare_wireframes(
        &mut self,
        device: &GPUDevice,
        runtime: &mut PrismRuntime,
    ) -> Result<(), String> {
        if runtime.state.controls.wireframe && self.wireframe.is_none() {
            runtime.ensure_wireframe(device);
            self.wireframe = Some(
                create_wireframe_draw(
                    device,
                    WireframeKind::Prism,
                    &self.scene,
                    &[UniformInput::new(
                        "params",
                        WireframeUniforms {
                            view_projection: Rc::clone(&runtime.state.view.view_projection),
                        },
                    )],
                    &format!("{}.wireframe", runtime.label),
                )
                .await?,
            );
        }
        if runtime.state.controls.light_wireframe && self.light_wireframe.is_none() {
            let state = &runtime.state;
            self.light_wireframe = Some(
                create_wireframe_draw(
                    device,
                    WireframeKind::Light,
                    &self.background,
                    &[UniformInput::new(
                        "scene",
                        scene_uniforms(
                            &state.view,
                            state.wall_extent(),
                            state.light_mesh_options().light,
                            &state.controls,
                            &state.light_mesh_layout,
                            1.0,
                        ),
                    )],
                    &format!("{}.light-wireframe", runtime.label),
                )
                .await?,
            );
        }
        Ok(())
    }

    pub fn resize(&mut self, device: &GPUDevice, runtime: &PrismRuntime) -> Result<bool, String> {
        self.presentation_valid = false;
        let size = runtime.state.output_size;
        let changed = self.background.resize(device, size) | self.scene.resize(device, size);
        if !changed {
            return Ok(false);
        }
        self.copy.rebind(
            device,
            &[NamedResource {
                name: "sceneTexture",
                resource: self.background.view.into(),
            }],
            "dark.copy-background",
        )?;
        self.front.rebind(
            device,
            &[
                NamedResource {
                    name: "sceneTexture",
                    resource: self.background.view.into(),
                },
                NamedResource {
                    name: "sceneSampler",
                    resource: runtime.scene_sampler.into(),
                },
                NamedResource {
                    name: "studioEnvironment",
                    resource: self.studio_view.into(),
                },
                NamedResource {
                    name: "debugEnvironment",
                    resource: self.debug_view.into(),
                },
                NamedResource {
                    name: "environmentSampler",
                    resource: runtime.environment_sampler.into(),
                },
            ],
            "dark.glass-front",
        )?;
        self.postprocess
            .resize(device, &self.scene, runtime.scene_sampler)?;
        let particle = if self.postprocess.low_quality {
            self.postprocess.visible_levels - 1
        } else {
            self.postprocess.visible_levels
        };
        self.dust.rebind(
            device,
            &[
                NamedResource {
                    name: "colorTexture",
                    resource: self.postprocess.levels[1].vertical.view.into(),
                },
                NamedResource {
                    name: "lightTexture",
                    resource: self.postprocess.levels[particle].vertical.view.into(),
                },
                NamedResource {
                    name: "lightSampler",
                    resource: runtime.scene_sampler.into(),
                },
            ],
            "dark.dust",
        )?;
        Ok(true)
    }

    pub fn render(
        &mut self,
        device: &GPUDevice,
        runtime: &PrismRuntime,
        output: &GPUTextureView,
        frame: DarkFrame,
    ) -> Result<(), String> {
        let state = &runtime.state;
        if (state.controls.wireframe && self.wireframe.is_none())
            || (state.controls.light_wireframe && self.light_wireframe.is_none())
        {
            return Err(
                "prepare_wireframes() must finish before rendering enabled debug draws.".into(),
            );
        }
        let update_scene = frame.update_scene || !self.presentation_valid;
        if frame.reveal_progress != self.bound_reveal_progress {
            self.postprocess.set_reveal(device, frame.reveal_progress)?;
            self.bound_reveal_progress = frame.reveal_progress;
        }
        if update_scene {
            self.light.values.write_value(
                device.queue(),
                "scene",
                scene_uniforms(
                    &state.view,
                    state.wall_extent(),
                    state.light_mesh_options().light,
                    &state.controls,
                    &state.light_mesh_layout,
                    frame.beam_width_reveal,
                ),
            )?;
            self.back.values.write_value(
                device.queue(),
                "params",
                glass_uniforms(
                    &state.view,
                    &state.controls,
                    PrismTheme::Dark,
                    runtime.debug_environment_enabled,
                ),
            )?;
            self.front.values.write_value(
                device.queue(),
                "params",
                glass_uniforms(
                    &state.view,
                    &state.controls,
                    PrismTheme::Dark,
                    runtime.debug_environment_enabled,
                ),
            )?;

            if let Some(wireframe) = &self.wireframe {
                wireframe.values.write_value(
                    device.queue(),
                    "params",
                    WireframeUniforms {
                        view_projection: Rc::clone(&state.view.view_projection),
                    },
                )?;
            }
            if let Some(wireframe) = &self.light_wireframe {
                wireframe.values.write_value(
                    device.queue(),
                    "scene",
                    scene_uniforms(
                        &state.view,
                        state.wall_extent(),
                        state.light_mesh_options().light,
                        &state.controls,
                        &state.light_mesh_layout,
                        frame.beam_width_reveal,
                    ),
                )?;
            }
            self.postprocess.set_controls(device, &state.controls)?;
        }
        self.dust.values.write_value(
            device.queue(),
            "params",
            dust_uniforms(
                &state.view,
                state.wall_extent(),
                state.output_size,
                state.camera_distance,
                frame.time,
                frame.reveal_progress,
            ),
        )?;
        let encoder = device.create_command_encoder(None);
        if update_scene {
            let clear = dark_wall_clear(&state.controls.wall_color, state.controls.view);
            let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
                label: Some("dark.backdrop"),
                color_attachments: &[self.background.color_attachment(false, Some(&clear))],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                max_draw_count: None,
            });
            if state.controls.view == PrismView::Glass && !state.controls.light_wireframe {
                pass.execute_bundles(&[self.backdrop_bundle]);
            } else {
                let layout = &state.light_mesh_layout;
                if state.controls.view != PrismView::Wall {
                    encode_light(
                        pass,
                        self.light.pipeline,
                        self.light.group,
                        runtime.light_buffer,
                        0,
                        layout.white_vertices,
                    );
                    encode_light(
                        pass,
                        self.light.pipeline,
                        self.light.group,
                        runtime.light_buffer,
                        layout.outgoing_first_vertex,
                        layout.outgoing_vertices,
                    );
                    if state.controls.light_wireframe
                        && let Some(wireframe) = &self.light_wireframe
                    {
                        encode_light(
                            pass,
                            wireframe.pipeline,
                            wireframe.group,
                            runtime.light_buffer,
                            0,
                            layout.white_vertices,
                        );
                        encode_light(
                            pass,
                            wireframe.pipeline,
                            wireframe.group,
                            runtime.light_buffer,
                            layout.outgoing_first_vertex,
                            layout.outgoing_vertices,
                        );
                    }
                }
                if state.controls.view == PrismView::Glass || state.controls.view == PrismView::Back
                {
                    encode_prism(pass, self.back.pipeline, self.back.group, &runtime.prism);
                }
                if state.controls.view != PrismView::Wall {
                    encode_light(
                        pass,
                        self.light.pipeline,
                        self.light.group,
                        runtime.light_buffer,
                        layout.internal_first_vertex,
                        layout.internal_vertices,
                    );
                    if state.controls.light_wireframe
                        && let Some(wireframe) = &self.light_wireframe
                    {
                        encode_light(
                            pass,
                            wireframe.pipeline,
                            wireframe.group,
                            runtime.light_buffer,
                            layout.internal_first_vertex,
                            layout.internal_vertices,
                        );
                    }
                }
            }
            pass.end();
            let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
                label: Some("dark.scene"),
                color_attachments: &[self
                    .scene
                    .color_attachment(false, Some(&[0.0, 0.0, 0.0, 1.0]))],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                max_draw_count: None,
            });
            encode_fullscreen(pass, self.copy.pipeline, self.copy.group);
            if state.controls.view == PrismView::Glass {
                encode_prism(pass, self.front.pipeline, self.front.group, &runtime.prism);
                if state.controls.wireframe
                    && let Some(wireframe) = &self.wireframe
                {
                    encode_prism(
                        pass,
                        wireframe.pipeline,
                        wireframe.group,
                        runtime
                            .prism_wireframe
                            .as_ref()
                            .expect("wireframe prepared"),
                    );
                }
            }
            pass.end();
            self.postprocess.encode_scene(encoder);
        }
        let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
            label: Some("dark.output"),
            color_attachments: &[GPURenderPassColorAttachment {
                view: output.into(),
                depth_slice: None,
                resolve_target: None,
                clear_value: None,
                load_op: "load",
                store_op: "store",
            }],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            max_draw_count: None,
        });
        encode_fullscreen(
            pass,
            self.postprocess.copy.pipeline,
            self.postprocess.copy.group,
        );
        if state.controls.view == PrismView::Glass {
            encode_dust(&self.dust, pass);
        }
        pass.end();
        device.queue().submit(&[encoder.finish(None)]);
        self.presentation_valid = true;
        Ok(())
    }
    pub fn destroy(&self) {
        if let Some(wireframe) = &self.wireframe {
            wireframe.destroy();
        }
        if let Some(wireframe) = &self.light_wireframe {
            wireframe.destroy();
        }

        self.background.destroy();
        self.scene.destroy();
        self.postprocess.destroy();
        for draw in [&self.light, &self.back, &self.front, &self.copy, &self.dust] {
            draw.destroy();
        }
    }
}
