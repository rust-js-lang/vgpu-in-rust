//! Read-only bindings to the WGSL asset compiler's actual reflection objects.
use js::JsObject;
use std::marker::PhantomData;

#[cfg_attr(rust_js, rust_js::types = "@vgpu/wgsl#ShaderReflection")]
pub struct ShaderReflection(PhantomData<JsObject>);

impl ShaderReflection {
    #[cfg_attr(rust_js, rust_js::link_name = "get bindings")]
    pub fn bindings(&self) -> Vec<&'static BindingInfo> {
        unreachable!()
    }
}

#[cfg_attr(rust_js, rust_js::types = "@vgpu/wgsl/reflect-source#BindingInfo")]
pub struct BindingInfo(PhantomData<JsObject>);

impl BindingInfo {
    #[cfg_attr(rust_js, rust_js::link_name = "get addressSpace")]
    pub fn address_space(&self) -> Option<String> {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get group")]
    pub fn group(&self) -> f64 {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get binding")]
    pub fn binding(&self) -> f64 {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get name")]
    pub fn name(&self) -> String {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get layout")]
    pub fn layout(&self) -> Option<&'static HostShareableLayout> {
        unreachable!()
    }
}

#[cfg_attr(
    rust_js,
    rust_js::types = "@vgpu/wgsl/reflect-source#HostShareableLayout"
)]
pub struct HostShareableLayout(PhantomData<JsObject>);

impl HostShareableLayout {
    #[cfg_attr(rust_js, rust_js::link_name = "get name")]
    pub fn name(&self) -> String {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get size")]
    pub fn size(&self) -> Option<f64> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get stride")]
    pub fn stride(&self) -> Option<f64> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get members")]
    pub fn members(&self) -> Option<Vec<&'static LayoutMember>> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get element")]
    pub fn element(&self) -> Option<&'static HostShareableLayout> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get runtimeSized")]
    pub fn runtime_sized(&self) -> Option<bool> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get type")]
    pub fn wgsl_type(&self) -> &'static WgslType {
        unreachable!()
    }
}

#[cfg_attr(rust_js, rust_js::types = "@vgpu/wgsl/reflect-source#LayoutMember")]
pub struct LayoutMember(PhantomData<JsObject>);

impl LayoutMember {
    #[cfg_attr(rust_js, rust_js::link_name = "get name")]
    pub fn name(&self) -> String {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get offset")]
    pub fn offset(&self) -> f64 {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get layout")]
    pub fn layout(&self) -> &'static HostShareableLayout {
        unreachable!()
    }
}

#[cfg_attr(rust_js, rust_js::types = "@vgpu/wgsl/reflect-source#WGSLType")]
pub struct WgslType(PhantomData<JsObject>);

impl WgslType {
    #[cfg_attr(rust_js, rust_js::link_name = "get kind")]
    pub fn kind(&self) -> String {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get name")]
    pub fn name(&self) -> Option<String> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get element")]
    pub fn element(&self) -> Option<&'static WgslType> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get width")]
    pub fn width(&self) -> Option<f64> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get columns")]
    pub fn columns(&self) -> Option<f64> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get rows")]
    pub fn rows(&self) -> Option<f64> {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get count")]
    pub fn count(&self) -> Option<f64> {
        unreachable!()
    }
}
