//! Rollback inventory for partially constructed native GPU graphs.
//! Ownership stays with the prepared graph; this inventory is used on failure.
use crate::bindings::ShaderBindings;
use crate::targets::{RenderTarget, create_render_target};
use react::webapi::{GPUBuffer, GPUDevice, GPUTexture};
#[derive(Default)]
pub(crate) struct Allocations {
    textures: Vec<&'static GPUTexture>,
    buffers: Vec<&'static GPUBuffer>,
}
impl Allocations {
    pub(crate) fn target(
        &mut self,
        device: &GPUDevice,
        size: [f64; 2],
        format: &str,
        sample_count: u32,
        label: &str,
    ) -> RenderTarget {
        let target = create_render_target(device, size, format, sample_count, label);
        self.retain_target(&target);
        target
    }
    pub(crate) fn retain_target(&mut self, target: &RenderTarget) {
        self.textures.push(target.texture);
        if let Some(msaa) = target.multisampled {
            self.textures.push(msaa);
        }
    }
    pub(crate) fn retain_bindings(&mut self, bindings: &ShaderBindings) {
        for uniform in &bindings.uniforms {
            self.buffers.push(uniform.resource.buffer);
        }
    }
    pub(crate) fn retain_texture(&mut self, texture: &'static GPUTexture) {
        self.textures.push(texture);
    }
    pub(crate) fn retain_buffer(&mut self, buffer: &'static GPUBuffer) {
        self.buffers.push(buffer);
    }
    pub(crate) fn destroy(&self) {
        for texture in &self.textures {
            texture.destroy();
        }
        for buffer in &self.buffers {
            buffer.destroy();
        }
    }
}
