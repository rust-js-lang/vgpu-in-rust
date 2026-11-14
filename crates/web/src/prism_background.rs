//! React-owned landing canvas lifecycle from prism-background.tsx.
//! The optional debug graph and performance browser API remain separate port work.
use crate::browser_renderer::{BrowserRenderer, RendererOptions, create_renderer};
use crate::light_assets::preload_light_assets;
use js::{JsError, Promise};
use react::webapi::{
    HTMLCanvasElement, MutationObserverInit, console, document, html_element, mutation_observer,
};
use react::{JSX, jsx, use_callback, use_effect, use_ref};
use std::rc::Rc;
use vgpu_prism::controls::{PrismControls, PrismTheme};
use vgpu_prism::quality::QualityPreference;

pub fn current_prism_mode() -> PrismTheme {
    if document
        .document_element()
        .expect("The document has a root element.")
        .class_list()
        .contains("light")
    {
        PrismTheme::Light
    } else {
        PrismTheme::Dark
    }
}
fn wall_color(mode: PrismTheme) -> &'static str {
    match mode {
        PrismTheme::Dark => "#000000",
        PrismTheme::Light => "#d2ccc2",
    }
}
pub struct PrismBackgroundProps {
    pub enabled: bool,
}
pub fn PrismBackground(
    PrismBackgroundProps { enabled }: PrismBackgroundProps,
) -> Option<JSX::Element> {
    if !enabled {
        return None;
    }
    Some(jsx! {<PrismCanvas />})
}
fn PrismCanvas() -> JSX::Element {
    let canvas_ref = use_ref(None::<&'static HTMLCanvasElement>);
    let renderer_ref = use_ref(None::<Rc<BrowserRenderer>>);
    let controls_ref = use_ref(PrismControls::default());
    let report_error = use_callback(
        |error: &'static JsError| console::error(("Prism background failed to render.", error)),
        (),
    );
    let activate_mode = use_callback(
        move |mode: PrismTheme| {
            if mode == PrismTheme::Light {
                preload_light_assets();
            }
            let wall_color = wall_color(mode);
            if controls_ref.current().wall_color != wall_color {
                let next_controls = PrismControls {
                    wall_color: wall_color.into(),
                    ..controls_ref.current()
                };
                controls_ref.set_current(next_controls.clone());
                if let Some(renderer) = renderer_ref.current() {
                    renderer.set_controls(next_controls);
                }
            }
            if let Some(renderer) = renderer_ref.current() {
                // The renderer reports preparation failures through onError and
                // deliberately keeps the previously active scene usable.
                let _: Promise<()> = js::promise(renderer.set_mode(mode)).then_with(|_| (), |_| ());
            }
        },
        (),
    );
    use_effect(
        move || {
            let canvas = canvas_ref.current()?;
            let initial_mode = current_prism_mode();
            if initial_mode == PrismTheme::Light {
                preload_light_assets();
            }
            let hero = canvas.closest("[data-hero-theme]");
            let framing_element = hero
                .and_then(|hero| hero.query_selector("[data-triangle-container]"))
                .map(|element| html_element::unchecked_from(element));
            let initial_controls = PrismControls {
                wall_color: wall_color(initial_mode).into(),
                ..PrismControls::default()
            };
            controls_ref.set_current(initial_controls.clone());
            let renderer = create_renderer(RendererOptions {
                canvas,
                framing_element,
                initial_mode,
                initial_quality: QualityPreference::Auto,
                initial_controls: Some(initial_controls),
                on_error: Rc::new(report_error),
                quality_logger: None,
                load_auto_quality: None,
            });
            renderer_ref.set_current(Some(Rc::clone(&renderer)));
            let theme_observer =
                mutation_observer::new(Box::new(move |_, _| activate_mode(current_prism_mode())));
            theme_observer.observe(
                document
                    .document_element()
                    .expect("The document has a root element."),
                Some(MutationObserverInit {
                    attributes: Some(true),
                    attribute_filter: Some(&["class".into()]),
                    ..Default::default()
                }),
            );
            Some(move || {
                theme_observer.disconnect();
                renderer_ref.set_current(None);
                renderer.dispose();
            })
        },
        Some(&(activate_mode, report_error)),
    );
    jsx! {
        <div data-prism-background className="absolute inset-0 overflow-hidden">
            <canvas ref={canvas_ref} aria-hidden="true" className="block h-full w-full touch-none" />
        </div>
    }
}
