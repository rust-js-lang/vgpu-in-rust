//! Browser-owned device and swapchain. Scene attachments belong to the controller.
use crate::capabilities::optional_prism_features;
use js::JsError;
use react::webapi::{
    GPUCanvasConfiguration, GPUCanvasContext, GPUDevice, GPUDeviceDescriptor,
    GPURequestAdapterOptions, HTMLCanvasElement, Navigator, contexts, gpu_texture_usage, navigator,
};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

pub struct CanvasSurface {
    pub canvas: &'static HTMLCanvasElement,
    pub context: &'static GPUCanvasContext,
    pub format: String,
}
pub async fn request_prism_device(performance_sampling: bool) -> &'static GPUDevice {
    if !js::is_defined((navigator, Navigator::gpu)) {
        js::throw(js::error::new(Some("WebGPU is unavailable."), None).as_unknown());
    }
    let gpu = navigator.gpu();
    // The optional capability probe never prevents the normal adapter request.
    let probed = js::settle(gpu.request_adapter(None)).await.ok().flatten();
    let Some(adapter) = probed else {
        let adapter = gpu
            .request_adapter(Some(GPURequestAdapterOptions {
                power_preference: None,
                ..Default::default()
            }))
            .await;
        let Some(adapter) = adapter else {
            js::throw(
                js::error::new(Some("navigator.gpu.requestAdapter() returned null."), None)
                    .as_unknown(),
            );
        };
        return adapter.request_device(None).await;
    };
    let features = optional_prism_features(adapter.features(), performance_sampling);
    if features.is_empty() {
        return adapter.request_device(None).await;
    }
    let requested = js::settle(adapter.request_device(Some(GPUDeviceDescriptor {
        required_features: Some(&features),
        ..Default::default()
    })))
    .await;
    match requested {
        Ok(device) => device,
        // An optimistic optional feature probe can become stale; preserve fp16 fallback.
        Err(_) => adapter.request_device(None).await,
    }
}
pub fn create_canvas_surface(
    device: &'static GPUDevice,
    canvas: &'static HTMLCanvasElement,
    size: [f64; 2],
) -> Result<CanvasSurface, String> {
    let context = canvas
        .get_context(contexts::Webgpu, None)
        .ok_or_else(|| "Canvas WebGPU context is unavailable.".to_string())?;
    let format = navigator.gpu().get_preferred_canvas_format();
    let previous = [canvas.width(), canvas.height()];
    canvas.set_width(size[0]);
    canvas.set_height(size[1]);
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        context.configure(GPUCanvasConfiguration {
            device,
            format: format.clone(),
            alpha_mode: Some("premultiplied".into()),
            color_space: Some("srgb".into()),
            usage: Some(f64::from(
                gpu_texture_usage::RENDER_ATTACHMENT
                    | gpu_texture_usage::TEXTURE_BINDING
                    | gpu_texture_usage::COPY_SRC,
            )),
            view_formats: None,
            tone_mapping: None,
        })
    })) {
        let _ = catch_unwind(AssertUnwindSafe(|| context.unconfigure()));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            canvas.set_width(previous[0]);
            canvas.set_height(previous[1]);
        }));
        resume_unwind(payload);
    }
    Ok(CanvasSurface {
        canvas,
        context,
        format,
    })
}
impl CanvasSurface {
    pub fn size(&self) -> [f64; 2] {
        [self.canvas.width(), self.canvas.height()]
    }
    pub fn resize(&self, size: [f64; 2]) {
        if self.canvas.width() != size[0] {
            self.canvas.set_width(size[0]);
        }
        if self.canvas.height() != size[1] {
            self.canvas.set_height(size[1]);
        }
    }
    pub fn destroy(&self) {
        self.context.unconfigure();
    }
}
/// Synchronous browser failures keep their native payload at the reporting boundary.
pub(crate) fn catch_native<T>(f: impl FnOnce() -> T) -> Result<T, &'static JsError> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        *payload
            .downcast_ref::<&'static JsError>()
            .expect("browser unwind payload")
    })
}
pub(crate) fn checked<T>(result: Result<T, String>) -> T {
    match result {
        Ok(value) => value,
        Err(message) => js::throw(js::error::new(Some(&message), None).as_unknown()),
    }
}
