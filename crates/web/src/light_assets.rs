//! Native compute baking, shared mask preload and portable CPU recovery for light materials.
use crate::shader::{ShaderSource, shader_module};
use crate::targets::create_texture;
use js::{Promise, promise::PromiseSettledResult};
use react::webapi::{
    Blob, GPUBindGroupDescriptor, GPUBindGroupEntry, GPUCommandEncoder,
    GPUCommandEncoderDescriptor, GPUComputePassDescriptor, GPUComputePipeline,
    GPUComputePipelineDescriptor, GPUCopyExternalImageDestInfo, GPUCopyExternalImageSourceInfo,
    GPUDevice, GPUProgrammableStage, GPUSamplerDescriptor, GPUShaderModule,
    GPUTexelCopyBufferLayout, GPUTexelCopyTextureInfo, GPUTexture, GPUTextureViewDescriptor,
    ImageBitmap, ImageBitmapOptions, global, gpu_texture_usage,
};
use std::cell::Cell;
use vgpu_prism::light_assets::{
    GeneratedLightAsset, LIGHT_ASSET_IDS, LightAssetId, generate_light_asset, generate_mip_chain,
};
unsafe extern "Rust" {
    #[link_name = "../../../assets/prism-background/pipelines/light/assets/bake.wgsl#default"]
    safe static BAKE: &'static ShaderSource;
    #[link_name = "../../../assets/prism-background/pipelines/light/assets/downsample.wgsl#default"]
    safe static DOWNSAMPLE: &'static ShaderSource;
    #[link_name = "../../../assets/prism-light/wall-global-light-mask.webp?url#default"]
    pub safe static WALL_MASK_URL: &'static str;
}
const WORKGROUP_SIZE: f64 = 8.0;
const WALL_MASK_SIZE: [f64; 2] = [768.0, 512.0];
thread_local! { static PENDING_MASK: Cell<Option<Promise<&'static Blob>>> = const { Cell::new(None) }; }

pub struct LightAssetTextures {
    pub wall_material: &'static GPUTexture,
    pub wall_lighting: &'static GPUTexture,
    pub caustic_profile: &'static GPUTexture,
}
impl LightAssetTextures {
    pub fn destroy(&self) {
        self.wall_material.destroy();
        self.wall_lighting.destroy();
        self.caustic_profile.destroy();
    }
}
async fn fetch_blob(url: &str) -> &'static Blob {
    let response = global::fetch(url, None).await;
    if !response.ok() {
        let message = format!("{} {}", response.status(), response.status_text());
        js::throw(js::error::new(Some(&message), None).as_unknown());
    }
    response.blob().await
}
pub fn load_preloaded_wall_mask() -> Promise<&'static Blob> {
    if let Some(pending) = PENDING_MASK.get() {
        return pending;
    }
    let pending = js::promise(fetch_blob(WALL_MASK_URL)).catch(|error| -> &'static Blob {
        PENDING_MASK.set(None);
        js::throw(error.as_unknown())
    });
    PENDING_MASK.set(Some(pending));
    pending
}
pub fn preload_light_assets() {
    js::spawn(Box::new(async {
        let _ = js::settle(load_preloaded_wall_mask()).await;
    }));
}
async fn load_wall_mask_texture(device: &GPUDevice) -> Option<&'static GPUTexture> {
    let mut bitmap: Option<&ImageBitmap> = None;
    let mut texture: Option<&GPUTexture> = None;
    let result = js::settle(js::promise(async {
        let blob = load_preloaded_wall_mask().await;
        if !js::is_defined(
            global::create_image_bitmap
                as fn(
                    &'static Blob,
                    Option<ImageBitmapOptions<'static>>,
                ) -> Promise<&'static ImageBitmap>,
        ) {
            return None;
        }
        let decoded = js::settle(global::create_image_bitmap(
            blob,
            Some(ImageBitmapOptions {
                color_space_conversion: Some("none"),
                premultiply_alpha: Some("none"),
                ..Default::default()
            }),
        ))
        .await;
        let image = match decoded {
            Ok(image) => image,
            Err(_) => global::create_image_bitmap(blob, None).await,
        };
        bitmap = Some(image);
        if image.width() != WALL_MASK_SIZE[0] || image.height() != WALL_MASK_SIZE[1] {
            let message = format!(
                "Wall mask is {}x{}; expected 768x512.",
                image.width(),
                image.height()
            );
            js::throw(js::error::new(Some(&message), None).as_unknown());
        }
        let made = create_texture(
            device,
            WALL_MASK_SIZE,
            "rgba8unorm",
            1,
            1,
            gpu_texture_usage::TEXTURE_BINDING
                | gpu_texture_usage::COPY_DST
                | gpu_texture_usage::RENDER_ATTACHMENT,
            "prism.light.wall-mask",
        );
        texture = Some(made);
        device.queue().copy_external_image_to_texture(
            GPUCopyExternalImageSourceInfo {
                source: image.into(),
                origin: None,
                flip_y: None,
            },
            GPUCopyExternalImageDestInfo {
                texture: made,
                mip_level: None,
                origin: None,
                aspect: None,
                color_space: None,
                premultiplied_alpha: None,
            },
            &[WALL_MASK_SIZE[0], WALL_MASK_SIZE[1], 1.0][..],
        );
        Some(made)
    }))
    .await;
    if let Some(bitmap) = bitmap {
        bitmap.close();
    }
    match result {
        Ok(texture) => texture,
        Err(_) => {
            if let Some(texture) = texture {
                texture.destroy();
            }
            None
        }
    }
}
async fn create_pipeline(
    device: &GPUDevice,
    module: &GPUShaderModule,
    entry_point: &str,
) -> &'static GPUComputePipeline {
    let descriptor = GPUComputePipelineDescriptor {
        label: Some(&format!("prism.light.{entry_point}")),
        layout: "auto".into(),
        compute: GPUProgrammableStage {
            module,
            entry_point: Some(entry_point),
            constants: None,
        },
    };
    if js::is_defined((device, GPUDevice::create_compute_pipeline_async)) {
        device.create_compute_pipeline_async(descriptor).await
    } else {
        device.create_compute_pipeline(descriptor)
    }
}
fn create_baked_texture(device: &GPUDevice, id: LightAssetId) -> &'static GPUTexture {
    let [width, height] = id.size();
    create_texture(
        device,
        [width as f64, height as f64],
        "rgba8unorm",
        1,
        width.max(height).ilog2() + 1,
        gpu_texture_usage::TEXTURE_BINDING | gpu_texture_usage::STORAGE_BINDING,
        &format!("prism.light.{}", id.name()),
    )
}
fn encode_base(
    device: &GPUDevice,
    encoder: &GPUCommandEncoder,
    pipeline: &GPUComputePipeline,
    output: &GPUTexture,
    mask: Option<&GPUTexture>,
) {
    let view = output.create_view(Some(GPUTextureViewDescriptor {
        base_mip_level: Some(0.0),
        mip_level_count: Some(1.0),
        ..Default::default()
    }));
    let mut entries = vec![GPUBindGroupEntry {
        binding: 0.0,
        resource: view.into(),
    }];
    if let Some(mask) = mask {
        entries.push(GPUBindGroupEntry {
            binding: 1.0,
            resource: mask.create_view(None).into(),
        });
        entries.push(GPUBindGroupEntry {
            binding: 2.0,
            resource: device
                .create_sampler(Some(GPUSamplerDescriptor {
                    min_filter: Some("linear"),
                    mag_filter: Some("linear"),
                    ..Default::default()
                }))
                .into(),
        });
    }
    let pass = encoder.begin_compute_pass(Some(GPUComputePassDescriptor {
        label: Some(&format!("{}.base", output.label())),
        timestamp_writes: None,
    }));
    pass.set_pipeline(pipeline);
    pass.set_bind_group(
        0,
        Some(device.create_bind_group(GPUBindGroupDescriptor {
            label: None,
            layout: pipeline.get_bind_group_layout(0),
            entries: &entries,
        })),
        None,
    );
    pass.dispatch_workgroups(
        (output.width() / WORKGROUP_SIZE).ceil(),
        Some((output.height() / WORKGROUP_SIZE).ceil()),
        None,
    );
    pass.end();
}
fn encode_mips(
    device: &GPUDevice,
    encoder: &GPUCommandEncoder,
    pipeline: &GPUComputePipeline,
    texture: &GPUTexture,
) {
    for level in 1..texture.mip_level_count() as u32 {
        let width = ((texture.width() as u32 >> level).max(1)) as f64;
        let height = ((texture.height() as u32 >> level).max(1)) as f64;
        let pass = encoder.begin_compute_pass(Some(GPUComputePassDescriptor {
            label: Some(&format!("{}.mip{level}", texture.label())),
            timestamp_writes: None,
        }));
        pass.set_pipeline(pipeline);
        pass.set_bind_group(
            0,
            Some(
                device.create_bind_group(GPUBindGroupDescriptor {
                    label: None,
                    layout: pipeline.get_bind_group_layout(0),
                    entries: &[
                        GPUBindGroupEntry {
                            binding: 0.0,
                            resource: texture
                                .create_view(Some(GPUTextureViewDescriptor {
                                    base_mip_level: Some(f64::from(level - 1)),
                                    mip_level_count: Some(1.0),
                                    ..Default::default()
                                }))
                                .into(),
                        },
                        GPUBindGroupEntry {
                            binding: 1.0,
                            resource: texture
                                .create_view(Some(GPUTextureViewDescriptor {
                                    base_mip_level: Some(f64::from(level)),
                                    mip_level_count: Some(1.0),
                                    ..Default::default()
                                }))
                                .into(),
                        },
                    ],
                }),
            ),
            None,
        );
        pass.dispatch_workgroups(
            (width / WORKGROUP_SIZE).ceil(),
            Some((height / WORKGROUP_SIZE).ceil()),
            None,
        );
        pass.end();
    }
}
pub async fn bake_light_asset_textures(device: &GPUDevice) -> Result<LightAssetTextures, String> {
    // Await the shared fetch before touching GPU state, preserving cancellation
    // and teardown ordering around the single initialization task.
    let mask = load_wall_mask_texture(device).await;
    let mut baked = Vec::new();
    let result = js::settle(js::promise(async {
        let bake = shader_module(device, BAKE, "prism.light.bake");
        let downsample = shader_module(device, DOWNSAMPLE, "prism.light.downsample");
        let pipelines = js::promise::all(vec![
            js::promise(create_pipeline(device, bake, "wall_material")),
            js::promise(create_pipeline(device, bake, "caustic_profile")),
            js::promise(create_pipeline(device, downsample, "main")),
            js::promise(create_pipeline(
                device,
                bake,
                if mask.is_some() {
                    "wall_lighting"
                } else {
                    "wall_lighting_fallback"
                },
            )),
        ])
        .await;
        for id in LIGHT_ASSET_IDS {
            baked.push(create_baked_texture(device, id));
        }
        let encoder = device.create_command_encoder(Some(GPUCommandEncoderDescriptor {
            label: Some("prism.light.bake"),
        }));
        encode_base(device, encoder, pipelines[0], baked[0], None);
        encode_base(device, encoder, pipelines[3], baked[1], mask);
        encode_base(device, encoder, pipelines[1], baked[2], None);
        for texture in &baked {
            encode_mips(device, encoder, pipelines[2], texture);
        }
        device.queue().submit(&[encoder.finish(None)]);
        device.queue().on_submitted_work_done().await;
        LightAssetTextures {
            wall_material: baked[0],
            wall_lighting: baked[1],
            caustic_profile: baked[2],
        }
    }))
    .await;
    if let Some(mask) = mask {
        mask.destroy();
    }
    match result {
        Ok(textures) => Ok(textures),
        Err(error) => {
            for texture in baked {
                texture.destroy();
            }
            Err(error.message())
        }
    }
}
async fn upload_recovery_asset(
    device: &GPUDevice,
    id: LightAssetId,
    levels: Vec<GeneratedLightAsset>,
) -> Result<&'static GPUTexture, String> {
    let base = &levels[0];
    let texture = create_texture(
        device,
        [base.width as f64, base.height as f64],
        "rgba8unorm",
        1,
        levels.len() as u32,
        gpu_texture_usage::TEXTURE_BINDING | gpu_texture_usage::COPY_DST,
        &format!("prism.light.{}", id.name()),
    );
    let result = js::settle(js::promise(async {
        for (level, image) in levels.iter().enumerate() {
            let bytes = js::uint8_array::from(&image.pixels);
            device.queue().write_texture(
                GPUTexelCopyTextureInfo {
                    texture,
                    mip_level: Some(level as f64),
                    origin: None,
                    aspect: None,
                },
                bytes,
                GPUTexelCopyBufferLayout {
                    offset: None,
                    bytes_per_row: Some((image.width * 4) as f64),
                    rows_per_image: Some(image.height as f64),
                },
                &[image.width as f64, image.height as f64, 1.0][..],
            );
        }
    }))
    .await;
    match result {
        Ok(()) => Ok(texture),
        Err(error) => {
            texture.destroy();
            Err(error.message())
        }
    }
}
/// Each task owns generation as well as upload, so a generation exception
/// becomes its rejection and cannot bypass all-settled resource cleanup.
async fn load_recovery_asset(
    device: &GPUDevice,
    id: LightAssetId,
) -> Result<&'static GPUTexture, String> {
    let levels = generate_mip_chain(generate_light_asset(id));
    upload_recovery_asset(device, id, levels).await
}
pub async fn load_recovery_asset_textures(
    device: &GPUDevice,
) -> Result<LightAssetTextures, String> {
    let tasks = LIGHT_ASSET_IDS
        .into_iter()
        .map(|id| js::promise(load_recovery_asset(device, id)))
        .collect();
    let settled = js::promise::all_settled(tasks).await;
    let mut loaded = Vec::new();
    let mut failure = None;
    for result in settled {
        match result {
            PromiseSettledResult::Fulfilled { value: Ok(texture) } => loaded.push(texture),
            PromiseSettledResult::Fulfilled { value: Err(error) } => {
                failure.get_or_insert(error);
            }
            PromiseSettledResult::Rejected { reason } => {
                failure.get_or_insert_with(|| reason.message());
            }
        }
    }
    if let Some(failure) = failure {
        for texture in loaded {
            texture.destroy();
        }
        return Err(failure);
    }
    Ok(LightAssetTextures {
        wall_material: loaded[0],
        wall_lighting: loaded[1],
        caustic_profile: loaded[2],
    })
}
pub async fn load_light_asset_textures(device: &GPUDevice) -> Result<LightAssetTextures, String> {
    match bake_light_asset_textures(device).await {
        Ok(textures) => Ok(textures),
        Err(_) => load_recovery_asset_textures(device).await,
    }
}
