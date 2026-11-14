//! Native render targets corresponding to vgpu's offscreen target resources.
use react::webapi::{
    GPUDevice, GPURenderPassColorAttachment, GPUTexture, GPUTextureDescriptor, GPUTextureView,
    gpu_texture_usage,
};

/// Pipeline compatibility does not require ownership of a target texture.
/// A canvas attachment supplies the same format/sample contract as an offscreen target.
pub struct RenderSignature<'a> {
    pub format: &'a str,
    pub sample_count: u32,
}

pub struct RenderTarget {
    pub texture: &'static GPUTexture,
    pub view: &'static GPUTextureView,
    pub multisampled: Option<&'static GPUTexture>,
    pub multisampled_view: Option<&'static GPUTextureView>,
    pub size: [f64; 2],
    pub format: String,
    pub sample_count: u32,
    pub label: String,
}

pub fn create_render_target(
    device: &GPUDevice,
    size: [f64; 2],
    format: &str,
    sample_count: u32,
    label: &str,
) -> RenderTarget {
    let texture = create_texture(
        device,
        size,
        format,
        1,
        1,
        gpu_texture_usage::RENDER_ATTACHMENT
            | gpu_texture_usage::TEXTURE_BINDING
            | gpu_texture_usage::COPY_SRC,
        label,
    );
    let view = texture.create_view(None);
    let multisampled = (sample_count > 1).then(|| {
        create_texture(
            device,
            size,
            format,
            sample_count,
            1,
            gpu_texture_usage::RENDER_ATTACHMENT,
            &format!("{label}.msaa"),
        )
    });
    let multisampled_view = multisampled.map(|texture| texture.create_view(None));
    RenderTarget {
        texture,
        view,
        multisampled,
        multisampled_view,
        size,
        format: format.into(),
        sample_count,
        label: label.into(),
    }
}

pub fn create_texture(
    device: &GPUDevice,
    size: [f64; 2],
    format: &str,
    sample_count: u32,
    mip_levels: u32,
    usage: u32,
    label: &str,
) -> &'static GPUTexture {
    device.create_texture(GPUTextureDescriptor {
        label: Some(label),
        size: (&[size[0], size[1], 1.0][..]).into(),
        mip_level_count: Some(f64::from(mip_levels)),
        sample_count: Some(f64::from(sample_count)),
        dimension: Some("2d"),
        format,
        usage: f64::from(usage),
        view_formats: None,
        texture_binding_view_dimension: None,
    })
}

impl RenderTarget {
    pub fn signature(&self) -> RenderSignature<'_> {
        RenderSignature {
            format: &self.format,
            sample_count: self.sample_count,
        }
    }

    pub fn color_attachment<'a>(
        &self,
        load: bool,
        clear: Option<&'a [f64]>,
    ) -> GPURenderPassColorAttachment<'a> {
        GPURenderPassColorAttachment {
            view: self.multisampled_view.unwrap_or(self.view).into(),
            resolve_target: self.multisampled.map(|_| self.view.into()),
            depth_slice: None,
            clear_value: clear.map(Into::into),
            load_op: if load { "load" } else { "clear" },
            store_op: "store",
        }
    }

    pub fn resize(&mut self, device: &GPUDevice, size: [f64; 2]) -> bool {
        if self.size == size {
            return false;
        }
        let next = create_render_target(device, size, &self.format, self.sample_count, &self.label);
        self.destroy();
        *self = next;
        true
    }
    pub fn destroy(&self) {
        self.texture.destroy();
        if let Some(texture) = self.multisampled {
            texture.destroy();
        }
    }
}
