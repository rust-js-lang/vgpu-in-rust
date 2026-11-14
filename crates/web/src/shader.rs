//! The shader-data boundary: WGSL imports/reflection are prepared by the existing
//! build-time loader; GPU module creation and diagnostics remain Rust-owned.
use std::marker::PhantomData;

use js::JsObject;
use react::webapi::{GPUDevice, GPUShaderModule, GPUShaderModuleDescriptor};

/// Immutable prepared shader data produced by @vgpu/wgsl's asset loader.
/// This is an imported library type, not a replacement shader representation.
#[cfg_attr(rust_js, rust_js::types = "@vgpu/wgsl#ShaderSource")]
pub struct ShaderSource(PhantomData<JsObject>);

impl ShaderSource {
    #[cfg_attr(rust_js, rust_js::link_name = "get reflection")]
    pub fn reflection(&self) -> &'static crate::reflection::ShaderReflection {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get wgsl")]
    pub fn wgsl(&self) -> String {
        unreachable!()
    }
}

pub fn shader_module(
    device: &GPUDevice,
    source: &ShaderSource,
    label: &str,
) -> &'static GPUShaderModule {
    device.create_shader_module(GPUShaderModuleDescriptor {
        label: Some(label),
        code: &source.wgsl(),
        compilation_hints: None,
    })
}

pub async fn compilation_errors(module: &GPUShaderModule) -> Vec<String> {
    module
        .get_compilation_info()
        .await
        .messages()
        .iter()
        .filter(|message| message.r#type() == "error")
        .map(|message| message.message())
        .collect()
}
