//! Deferred image activation from deferred-image.tsx. Responsive candidates use
//! Next's own image props; static derivatives replace its server endpoint.
use js::{Dict, dict};
use next::image::{
    Image, ImageLoader, ImageLoaderProps, ImageProps, IntoImageSrc, get_image_props,
};
use react::webapi::{HTMLSpanElement, IntersectionObserverInit, intersection_observer, window};
use react::{CSSProperties, JSX, jsx, use_effect, use_ref, use_state};
struct ImageFormat {
    name: &'static str,
    mime: &'static str,
}
struct ImageVariants {
    variants: &'static Dict<&'static Dict<&'static str>>,
}
struct ImageManifest {
    formats: &'static [ImageFormat],
    fallback: &'static str,
    images: &'static Dict<ImageVariants>,
}
unsafe extern "Rust" {
    #[link_name = "../../../assets/generated/static-images.json#default"]
    safe static IMAGES: &'static ImageManifest;
}
fn asset_loader(format: &'static str) -> ImageLoader {
    Box::new(move |props: ImageLoaderProps| {
        let image = dict::get(IMAGES.images, props.src.as_str())
            .expect("The image must have generated static variants.");
        let width = js::number::to_string(props.width, None);
        let formats = dict::get(image.variants, &width)
            .expect("The image width must have a generated static variant.");
        dict::get(formats, format)
            .expect("The image format must have a generated static variant.")
            .to_string()
    })
}
// Only geometry and quality affect candidate URLs. ImageProps contains owned
// callbacks, so create this small descriptor without copying callback ownership.
fn candidate_props<'a, S: IntoImageSrc + Copy>(
    props: &ImageProps<'a, S>,
    format: &'static str,
) -> ImageProps<'a, S> {
    ImageProps {
        src: props.src,
        alt: props.alt,
        width: props.width,
        height: props.height,
        fill: props.fill,
        loader: Some(asset_loader(format)),
        quality: props.quality,
        sizes: props.sizes,
        preload: None,
        priority: None,
        loading: None,
        placeholder: None,
        blur_data_url: None,
        unoptimized: None,
        override_src: None,
        on_loading_complete: None,
        layout: None,
        object_fit: None,
        object_position: None,
        lazy_boundary: None,
        lazy_root: None,
        class_name: None,
        id: None,
        title: None,
        img: Default::default(),
    }
}
/// Keeps below-the-fold image URLs out of the initial document fetch queue.
/// Native lazy loading has a generous look-ahead window, too early while the
/// WebGPU hero is still initializing.
pub fn DeferredImage<S: IntoImageSrc + Copy>(mut props: ImageProps<'_, S>) -> JSX::Element {
    let marker_ref = use_ref(None::<&'static HTMLSpanElement>);
    let (should_load, set_should_load) = use_state(|| false);
    use_effect(
        move || {
            let marker = marker_ref.current()?;
            if !js::has(window, "IntersectionObserver") {
                set_should_load.set(true);
                return None;
            }
            let observer = intersection_observer::new(
                Box::new(move |entries, observer| {
                    if !entries.first().is_some_and(|entry| entry.is_intersecting()) {
                        return;
                    }
                    set_should_load.set(true);
                    observer.disconnect();
                }),
                Some(IntersectionObserverInit {
                    root_margin: Some("256px 0px"),
                    ..Default::default()
                }),
            );
            observer.observe(marker);
            Some(move || observer.disconnect())
        },
        Some(&()),
    );
    let image = if *should_load {
        if props.unoptimized == Some(true) || props.loader.is_some() {
            Some(jsx! {<Image {...props} />})
        } else {
            let sources=IMAGES.formats.iter().filter(|format|format.name!=IMAGES.fallback).map(|format| {
                let candidates=get_image_props(candidate_props(&props,format.name));
                jsx! {<source key={format.name} type={format.mime} srcSet={candidates.props.src_set} sizes={candidates.props.sizes} />}
            }).collect::<Vec<_>>();
            let style = if props.fill == Some(true) {
                CSSProperties::new().position("absolute").inset(0)
            } else {
                CSSProperties::new()
            };
            props.loader = Some(asset_loader(IMAGES.fallback));
            Some(jsx! {<picture style={style}>{sources}<Image {...props} /></picture>})
        }
    } else {
        None
    };
    jsx! {<span ref={marker_ref} className="absolute inset-0 block">{image}</span>}
}
