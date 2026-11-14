//! On-demand rainbow-ring renderer from shader-code-scales-renderer.ts.
use crate::bindings::{BoundPipeline, UniformInput};
use crate::canvas_surface::{CanvasSurface, checked, create_canvas_surface};
use crate::fullscreen::{encode_fullscreen, fullscreen_pipeline_sync, fullscreen_vertex};
use crate::shader::{ShaderSource, shader_module};
use react::webapi::{
    GPUDevice, GPUDeviceDescriptor, GPURenderPassColorAttachment, GPURenderPassDescriptor,
    GPURequestAdapterOptions, HTMLCanvasElement, Navigator, navigator, resize_observer, window,
};
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::rc::Rc;

unsafe extern "Rust" {
    #[link_name = "../../../assets/landing/shader-code-scales/main.wgsl#default"]
    safe static SOURCE: &'static ShaderSource;
}
pub type Dispose = Box<dyn Fn()>;
const LABEL: &str = "homepage-shader-code-scales-preview";
struct Uniforms {
    resolution: [f64; 2],
}
struct Preview {
    device: &'static GPUDevice,
    surface: CanvasSurface,
    effect: RefCell<Option<BoundPipeline>>,
    resize_frame: Cell<Option<f64>>,
    disposed: Cell<bool>,
}
impl Preview {
    fn render(&self) {
        self.resize_frame.set(None);
        let size = [
            js::math::max((1.0, js::math::round(self.surface.canvas.client_width()))),
            js::math::max((1.0, js::math::round(self.surface.canvas.client_height()))),
        ];
        if self.surface.size() != size {
            self.surface.resize(size);
        }
        let mut retained = self.effect.borrow_mut();
        if retained.is_none() {
            let vertex = fullscreen_vertex(self.device);
            let fragment = shader_module(self.device, SOURCE, LABEL);
            let pipeline = fullscreen_pipeline_sync(
                self.device,
                vertex,
                fragment,
                &self.surface.format,
                1,
                LABEL,
            );
            *retained = Some(checked(BoundPipeline::prepare(
                self.device,
                SOURCE,
                pipeline,
                &[UniformInput::new("uniforms", Uniforms { resolution: size })],
                &[],
                LABEL,
            )));
        } else {
            checked(
                retained
                    .as_ref()
                    .expect("prepared effect")
                    .values
                    .write_value(
                        self.device.queue(),
                        "uniforms",
                        Uniforms { resolution: size },
                    ),
            );
        }
        let effect = retained.as_ref().expect("prepared effect");
        let encoder = self.device.create_command_encoder(None);
        let view = self.surface.context.get_current_texture().create_view(None);
        let pass = encoder.begin_render_pass(GPURenderPassDescriptor {
            label: None,
            color_attachments: &[GPURenderPassColorAttachment {
                view: view.into(),
                resolve_target: None,
                depth_slice: None,
                clear_value: Some((&[0.0, 0.0, 0.0, 1.0][..]).into()),
                load_op: "clear",
                store_op: "store",
            }],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            max_draw_count: None,
        });
        encode_fullscreen(pass, effect.pipeline, effect.group);
        pass.end();
        self.device.queue().submit(&[encoder.finish(None)]);
    }
    fn resize(self: &Rc<Self>) {
        if self.resize_frame.get().is_none() {
            let preview = Rc::clone(self);
            self.resize_frame.set(Some(
                window.request_animation_frame(Box::new(move |_| preview.render())),
            ));
        }
    }
    fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        if let Some(effect) = self.effect.borrow().as_ref() {
            effect.destroy();
        }
        self.surface.destroy();
        self.device.destroy();
    }
}
/// Render once, then coalesce layout changes into one frame. There is no idle loop.
pub async fn render_shader_preview(canvas: &'static HTMLCanvasElement) -> Dispose {
    let adapter = if js::is_defined((navigator, Navigator::gpu)) {
        navigator
            .gpu()
            .request_adapter(Some(GPURequestAdapterOptions::default()))
            .await
    } else {
        None
    };
    let Some(adapter) = adapter else {
        js::throw(
            js::error::new(Some("navigator.gpu.requestAdapter() returned null."), None)
                .as_unknown(),
        );
    };
    let device = adapter
        .request_device(Some(GPUDeviceDescriptor::default()))
        .await;
    let mut prepared = None;
    let created = catch_unwind(AssertUnwindSafe(|| {
        let size = [
            js::math::max((1.0, js::math::round(canvas.client_width()))),
            js::math::max((1.0, js::math::round(canvas.client_height()))),
        ];
        let surface = checked(create_canvas_surface(device, canvas, size));
        let preview = Rc::new(Preview {
            device,
            surface,
            effect: RefCell::new(None),
            resize_frame: Cell::new(None),
            disposed: Cell::new(false),
        });
        prepared = Some(Rc::clone(&preview));
        let observed = Rc::clone(&preview);
        let observer = resize_observer::new(Box::new(move |_, _| observed.resize()));
        observer.observe(canvas, None);
        preview.render();
        move || {
            observer.disconnect();
            if let Some(frame) = preview.resize_frame.get() {
                window.cancel_animation_frame(frame);
            }
            preview.dispose();
        }
    }));
    match created {
        Ok(dispose) => Box::new(dispose),
        Err(error) => {
            if let Some(preview) = prepared {
                preview.dispose();
            } else {
                device.destroy();
            }
            resume_unwind(error)
        }
    }
}
