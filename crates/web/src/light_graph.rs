//! Complete light production graph, retaining the source's material and pass order.
use crate::bindings::{BoundPipeline, NamedResource, UniformInput};
use crate::fullscreen::{encode_fullscreen, fullscreen_pipeline, fullscreen_vertex};
use crate::geometry::encode_prism;
use crate::glass_pass::{GlassLayer, glass_pipeline};
use crate::light_assets::{LightAssetTextures, load_light_asset_textures};
use crate::resources::Allocations;
use crate::runtime::{GraphInputs, PrismRuntime};
use crate::shader::{ShaderSource, shader_module};
use crate::shadow::{ShadowBuffers, create_shadow_geometry, encode_shadow, shadow_pipeline};
use crate::spectral_pass::{encode_light, light_pipeline};
use crate::targets::RenderTarget;
use crate::wall::wall_pipeline;
use crate::wireframe::{WireframeKind, create_wireframe_draw};
use react::webapi::{
    GPUDevice, GPURenderBundle, GPURenderBundleEncoderDescriptor, GPURenderPassColorAttachment,
    GPURenderPassDescriptor, GPURenderPipeline, GPUSampler, GPUSamplerDescriptor, GPUTextureView,
};
use std::rc::Rc;
use vgpu_prism::controls::{PrismTheme, PrismView};
use vgpu_prism::light_uniforms::{
    light_caustic_uniforms, light_glass_accent_uniforms, light_present_uniforms,
    light_wall_uniforms, prism_shadow_uniforms,
};
use vgpu_prism::uniforms::{WireframeUniforms, glass_uniforms, scene_uniforms};

unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/wall/wall.wgsl#default"]
    safe static WALL: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/wall/wall-low.wgsl#default"]
    safe static WALL_LOW: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/shadow/shadow.wgsl#default"]
    safe static SHADOW: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/caustic/caustic.wgsl#default"]
    safe static CAUSTIC: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/glass/glass-back.wgsl#default"]
    safe static BACK: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/glass/glass.wgsl#default"]
    safe static FRONT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/glass-accent/glass-accent.wgsl#default"]
    safe static ACCENT: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/shared/presentation/copy-linear.wgsl#default"]
    safe static COPY: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/passes/presentation/present.wgsl#default"]
    safe static PRESENT: &'static ShaderSource;
}

pub struct LightSettings<'a> {
    pub output_format: &'a str,
    pub low_quality: bool,
    pub sample_count: u32,
}
#[derive(Clone, Copy)]
pub struct LightFrame {
    pub update_scene: bool,
    pub reveal_progress: f64,
    pub beam_width_reveal: f64,
}
impl Default for LightFrame {
    fn default() -> Self {
        Self {
            update_scene: true,
            reveal_progress: 1.0,
            beam_width_reveal: 1.0,
        }
    }
}
pub struct LightGraph {
    pub background: RenderTarget,
    pub scene: RenderTarget,
    pub assets: LightAssetTextures,
    pub shadow_geometry: ShadowBuffers,
    pub material_sampler: &'static GPUSampler,
    pub wall: BoundPipeline,
    pub shadow: BoundPipeline,
    pub caustic: BoundPipeline,
    pub back: BoundPipeline,
    pub front: BoundPipeline,
    pub accent: BoundPipeline,
    pub copy: BoundPipeline,
    pub present: BoundPipeline,
    pub backdrop_bundle: &'static GPURenderBundle,
    pub studio_view: &'static GPUTextureView,
    pub debug_view: &'static GPUTextureView,
    pub presentation_valid: bool,
    pub wireframe: Option<BoundPipeline>,
    pub light_wireframe: Option<BoundPipeline>,
}

pub async fn create_light_graph(
    device: &'static GPUDevice,
    runtime: GraphInputs,
    settings: LightSettings<'_>,
) -> Result<LightGraph, String> {
    let mut allocations = Allocations::default();
    let result = match js::settle(js::promise(build_light_graph(
        device,
        &runtime,
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
    let prepared = match js::settle(js::promise(graph.prepare_wireframes(device, &runtime))).await {
        Ok(result) => result,
        Err(error) => Err(error.message()),
    };
    if let Err(error) = prepared {
        graph.destroy();
        return Err(error);
    }
    Ok(graph)
}

async fn compile_light_pipelines(
    device: &GPUDevice,
    wall: &ShaderSource,
    settings: &LightSettings<'_>,
) -> Vec<&'static GPURenderPipeline> {
    let fullscreen = fullscreen_vertex(device);
    js::promise::all(vec![
        js::promise(wall_pipeline(device, wall, "rgba16float", "light.wall")),
        js::promise(shadow_pipeline(
            device,
            SHADOW,
            "rgba16float",
            "light.shadow",
        )),
        js::promise(light_pipeline(
            device,
            CAUSTIC,
            "rgba16float",
            "light.caustic",
        )),
        js::promise(glass_pipeline(
            device,
            BACK,
            "rgba16float",
            GlassLayer::Back,
            1,
            "light.back",
        )),
        js::promise(glass_pipeline(
            device,
            FRONT,
            "rgba16float",
            GlassLayer::Front,
            settings.sample_count,
            "light.front",
        )),
        js::promise(glass_pipeline(
            device,
            ACCENT,
            "rgba16float",
            GlassLayer::Accent,
            settings.sample_count,
            "light.accent",
        )),
        js::promise(fullscreen_pipeline(
            device,
            fullscreen,
            shader_module(device, COPY, "light.copy"),
            "rgba16float",
            settings.sample_count,
            "light.copy",
        )),
        js::promise(fullscreen_pipeline(
            device,
            fullscreen,
            shader_module(device, PRESENT, "light.present"),
            settings.output_format,
            1,
            "light.present",
        )),
    ])
    .await
}

async fn build_light_graph(
    device: &'static GPUDevice,
    runtime: &GraphInputs,
    settings: LightSettings<'_>,
    allocations: &mut Allocations,
) -> Result<LightGraph, String> {
    let background = allocations.target(
        device,
        runtime.state.output_size,
        "rgba16float",
        1,
        "light.backdrop",
    );
    let scene = allocations.target(
        device,
        runtime.state.output_size,
        "rgba16float",
        settings.sample_count,
        "light.scene",
    );
    let shadow_geometry = create_shadow_geometry(device, "light.prism-shadow");
    allocations.retain_buffer(shadow_geometry.vertices);
    allocations.retain_buffer(shadow_geometry.indices);
    let wall_source = if settings.low_quality { WALL_LOW } else { WALL };
    let environment = runtime.environment_ready;
    // Every GPU-owning branch settles before any failure releases graph or runtime resources.
    // Settling each promise turns rejection into a value, so all3 still waits for all three.
    let (assets, environment, pipelines) = js::promise::all3((
        js::promise(js::settle(js::promise(load_light_asset_textures(device)))),
        js::promise(js::settle(environment)),
        js::promise(js::settle(js::promise(compile_light_pipelines(
            device,
            wall_source,
            &settings,
        )))),
    ))
    .await;
    let assets = assets.map_err(|error| error.message())??;
    for texture in [
        assets.wall_material,
        assets.wall_lighting,
        assets.caustic_profile,
    ] {
        allocations.retain_texture(texture);
    }
    environment.map_err(|error| error.message())?;
    let pipelines = pipelines.map_err(|error| error.message())?;
    let state = &runtime.state;
    let material_sampler = device.create_sampler(Some(GPUSamplerDescriptor {
        min_filter: Some("linear"),
        mag_filter: Some("linear"),
        mipmap_filter: Some("linear"),
        address_mode_u: Some("repeat"),
        address_mode_v: Some("repeat"),
        ..Default::default()
    }));
    let wall_material = assets.wall_material.create_view(None);
    let wall_lighting = assets.wall_lighting.create_view(None);
    let caustic_profile = assets.caustic_profile.create_view(None);
    let wall = BoundPipeline::prepare(
        device,
        wall_source,
        pipelines[0],
        &[UniformInput::new("params", light_wall_uniforms(state))],
        &[
            NamedResource {
                name: "wallMaterial",
                resource: wall_material.into(),
            },
            NamedResource {
                name: "wallLighting",
                resource: wall_lighting.into(),
            },
            NamedResource {
                name: "materialSampler",
                resource: material_sampler.into(),
            },
        ],
        "light.wall",
    )?;
    allocations.retain_bindings(&wall.values);
    let shadow = BoundPipeline::prepare(
        device,
        SHADOW,
        pipelines[1],
        &[UniformInput::new(
            "shadow",
            prism_shadow_uniforms(Rc::clone(&state.view.view_projection)),
        )],
        &[],
        "light.shadow",
    )?;
    allocations.retain_bindings(&shadow.values);
    let caustic = BoundPipeline::prepare(
        device,
        CAUSTIC,
        pipelines[2],
        &[
            UniformInput::new(
                "scene",
                scene_uniforms(
                    &state.view,
                    state.wall_extent(),
                    state.light_mesh_options().light,
                    &state.controls,
                    &state.light_mesh_layout,
                    1.0,
                ),
            ),
            UniformInput::new("caustic", light_caustic_uniforms(state)),
        ],
        &[
            NamedResource {
                name: "causticProfile",
                resource: caustic_profile.into(),
            },
            NamedResource {
                name: "causticSampler",
                resource: material_sampler.into(),
            },
            NamedResource {
                name: "wallMaterial",
                resource: wall_material.into(),
            },
        ],
        "light.caustic",
    )?;
    allocations.retain_bindings(&caustic.values);
    let studio_view = runtime.studio_view;
    let debug_view = runtime.debug_view;
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
        pipelines[3],
        &[UniformInput::new(
            "params",
            glass_uniforms(
                &state.view,
                &state.controls,
                PrismTheme::Light,
                runtime.debug_environment_enabled,
            ),
        )],
        &shared,
        "light.back",
    )?;
    allocations.retain_bindings(&back.values);
    let accent = BoundPipeline::prepare(
        device,
        ACCENT,
        pipelines[5],
        &[
            UniformInput::new(
                "params",
                glass_uniforms(
                    &state.view,
                    &state.controls,
                    PrismTheme::Light,
                    runtime.debug_environment_enabled,
                ),
            ),
            UniformInput::new("accent", light_glass_accent_uniforms()),
        ],
        &shared,
        "light.accent",
    )?;
    allocations.retain_bindings(&accent.values);
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
        pipelines[4],
        &[UniformInput::new(
            "params",
            glass_uniforms(
                &state.view,
                &state.controls,
                PrismTheme::Light,
                runtime.debug_environment_enabled,
            ),
        )],
        &front_resources,
        "light.front",
    )?;
    allocations.retain_bindings(&front.values);
    let copy = BoundPipeline::prepare(
        device,
        COPY,
        pipelines[6],
        &[],
        &[NamedResource {
            name: "sceneTexture",
            resource: background.view.into(),
        }],
        "light.copy",
    )?;
    allocations.retain_bindings(&copy.values);
    let present = BoundPipeline::prepare(
        device,
        PRESENT,
        pipelines[7],
        &[UniformInput::new(
            "params",
            light_present_uniforms(state, 1.0),
        )],
        &[NamedResource {
            name: "sceneTexture",
            resource: scene.view.into(),
        }],
        "light.present",
    )?;
    allocations.retain_bindings(&present.values);
    let bundle = device.create_render_bundle_encoder(GPURenderBundleEncoderDescriptor {
        label: Some("light.backdrop-bundle"),
        color_formats: &["rgba16float".into()],
        depth_stencil_format: None,
        sample_count: Some(1.0),
        depth_read_only: None,
        stencil_read_only: None,
    });
    bundle.set_pipeline(wall.pipeline);
    bundle.set_bind_group(0, Some(wall.group), None);
    bundle.draw(6, None, None, None);
    bundle.set_pipeline(shadow.pipeline);
    bundle.set_bind_group(0, Some(shadow.group), None);
    bundle.set_vertex_buffer(0, Some(shadow_geometry.vertices), None, None);
    bundle.set_index_buffer(shadow_geometry.indices, "uint32", None, None);
    bundle.draw_indexed(shadow_geometry.index_count as f64, None, None, None, None);
    let layout = &state.light_mesh_layout;
    bundle.set_pipeline(caustic.pipeline);
    bundle.set_bind_group(0, Some(caustic.group), None);
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
    bundle.set_pipeline(caustic.pipeline);
    bundle.set_bind_group(0, Some(caustic.group), None);
    bundle.set_vertex_buffer(0, Some(runtime.light_buffer), None, None);
    bundle.draw(
        layout.internal_vertices as f64,
        None,
        Some(layout.internal_first_vertex as f64),
        None,
    );
    Ok(LightGraph {
        background,
        scene,
        assets,
        shadow_geometry,
        material_sampler,
        wall,
        shadow,
        caustic,
        back,
        front,
        accent,
        copy,
        present,
        backdrop_bundle: bundle.finish(None),
        studio_view,
        debug_view,
        presentation_valid: false,
        wireframe: None,
        light_wireframe: None,
    })
}

impl LightGraph {
    pub async fn prepare_wireframes(
        &mut self,
        device: &GPUDevice,
        runtime: &GraphInputs,
    ) -> Result<(), String> {
        if runtime.state.controls.wireframe && self.wireframe.is_none() {
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
            "light.copy",
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
            "light.front",
        )?;
        self.present.rebind(
            device,
            &[NamedResource {
                name: "sceneTexture",
                resource: self.scene.view.into(),
            }],
            "light.present",
        )?;
        Ok(true)
    }

    pub fn render(
        &mut self,
        device: &GPUDevice,
        runtime: &PrismRuntime,
        output: &GPUTextureView,
        frame: LightFrame,
    ) -> Result<(), String> {
        let state = &runtime.state;
        let update_scene = frame.update_scene || !self.presentation_valid;
        if update_scene {
            if (state.controls.wireframe && self.wireframe.is_none())
                || (state.controls.light_wireframe && self.light_wireframe.is_none())
            {
                return Err(
                    "prepare_wireframes() must finish before rendering enabled debug draws.".into(),
                );
            }
            self.wall
                .values
                .write_value(device.queue(), "params", light_wall_uniforms(state))?;
            self.shadow.values.write_value(
                device.queue(),
                "shadow",
                prism_shadow_uniforms(Rc::clone(&state.view.view_projection)),
            )?;
            self.caustic.values.write_value(
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
            self.caustic.values.write_value(
                device.queue(),
                "caustic",
                light_caustic_uniforms(state),
            )?;
            for draw in [&self.back, &self.front, &self.accent] {
                draw.values.write_value(
                    device.queue(),
                    "params",
                    glass_uniforms(
                        &state.view,
                        &state.controls,
                        PrismTheme::Light,
                        runtime.debug_environment_enabled,
                    ),
                )?;
            }
            self.accent.values.write_value(
                device.queue(),
                "accent",
                light_glass_accent_uniforms(),
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
        }
        self.present.values.write_value(
            device.queue(),
            "params",
            light_present_uniforms(state, frame.reveal_progress),
        )?;
        let encoder = device.create_command_encoder(None);
        if update_scene {
            let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
                label: Some("light.backdrop"),
                color_attachments: &[self
                    .background
                    .color_attachment(false, Some(&[0.0, 0.0, 0.0, 1.0]))],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                max_draw_count: None,
            });
            if state.controls.view == PrismView::Glass && !state.controls.light_wireframe {
                pass.execute_bundles(&[self.backdrop_bundle]);
            } else {
                let layout = &state.light_mesh_layout;
                if state.controls.view != PrismView::Caustic {
                    pass.set_pipeline(self.wall.pipeline);
                    pass.set_bind_group(0, Some(self.wall.group), None);
                    pass.draw(6, None, None, None);
                    encode_shadow(
                        pass,
                        self.shadow.pipeline,
                        self.shadow.group,
                        &self.shadow_geometry,
                    );
                }
                if state.controls.view != PrismView::Wall {
                    encode_light(
                        pass,
                        self.caustic.pipeline,
                        self.caustic.group,
                        runtime.light_buffer,
                        0,
                        layout.white_vertices,
                    );
                    encode_light(
                        pass,
                        self.caustic.pipeline,
                        self.caustic.group,
                        runtime.light_buffer,
                        layout.outgoing_first_vertex,
                        layout.outgoing_vertices,
                    );
                }
                if state.controls.view == PrismView::Glass || state.controls.view == PrismView::Back
                {
                    encode_prism(pass, self.back.pipeline, self.back.group, &runtime.prism);
                }
                if state.controls.view != PrismView::Wall {
                    encode_light(
                        pass,
                        self.caustic.pipeline,
                        self.caustic.group,
                        runtime.light_buffer,
                        layout.internal_first_vertex,
                        layout.internal_vertices,
                    );
                }
                // Source draws the complete debug light mesh here, including in wall-only view.
                if state.controls.light_wireframe
                    && let Some(wireframe) = &self.light_wireframe
                {
                    encode_light(
                        pass,
                        wireframe.pipeline,
                        wireframe.group,
                        runtime.light_buffer,
                        0,
                        layout.vertex_count,
                    );
                }
            }
            pass.end();
            let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
                label: Some("light.scene"),
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
                encode_prism(
                    pass,
                    self.accent.pipeline,
                    self.accent.group,
                    &runtime.prism,
                );
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
        }
        let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
            label: Some("light.present"),
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
        encode_fullscreen(pass, self.present.pipeline, self.present.group);
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
        self.assets.destroy();
        self.shadow_geometry.destroy();
        for draw in [
            &self.wall,
            &self.shadow,
            &self.caustic,
            &self.back,
            &self.front,
            &self.accent,
            &self.copy,
            &self.present,
        ] {
            draw.destroy();
        }
    }
}
