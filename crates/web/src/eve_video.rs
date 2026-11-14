//! Deferred video and media progress from eve-video.tsx.
use react::event::SyntheticEvent;
use react::webapi::{
    HTMLDivElement, HTMLVideoElement, IntersectionObserverInit, intersection_observer, window,
};
use react::{CSSProperties, JSX, jsx, use_effect, use_ref, use_state};
const EVE_VIDEO: &str = "./examples/eve/eve-hero-1080p.mp4";
const EVE_VIDEO_POSTER: &str = "./examples/eve/eve-billboard.png";
pub fn format_time(value: f64) -> String {
    if !value.is_finite() || value < 0.0 {
        return "00:00".into();
    }
    let seconds = value.floor();
    let minutes = js::number::to_string((seconds / 60.0).floor(), None);
    let remainder = js::number::to_string(seconds % 60.0, None);
    format!("{minutes:0>2}:{remainder:0>2}")
}
pub fn EveVideo() -> JSX::Element {
    let container_ref = use_ref(None::<&'static HTMLDivElement>);
    let (should_load, set_should_load) = use_state(|| false);
    let (current_time, set_current_time) = use_state(|| 0.0);
    let (duration, set_duration) = use_state(|| 0.0);
    let progress = if *duration > 0.0 {
        js::math::min((100.0, current_time / duration * 100.0))
    } else {
        0.0
    };
    use_effect(
        move || {
            let container = container_ref.current()?;
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
            observer.observe(container);
            Some(move || observer.disconnect())
        },
        Some(&()),
    );
    jsx! {
        <div ref={container_ref} className="flex aspect-video flex-col bg-black">
            <video aria-label="Eve shader rendered as video" autoPlay={*should_load} className="min-h-0 w-full flex-1 object-contain" loop muted
                onDurationChange={move |event:&SyntheticEvent<HTMLVideoElement>|set_duration.set(event.current_target().duration())}
                onLoadedMetadata={move |event:&SyntheticEvent<HTMLVideoElement>|set_duration.set(event.current_target().duration())}
                onTimeUpdate={move |event:&SyntheticEvent<HTMLVideoElement>| {set_current_time.set(event.current_target().current_time());set_duration.set(event.current_target().duration());}}
                playsInline poster={if *should_load {Some(EVE_VIDEO_POSTER)} else {None}} preload="none" src={if *should_load {Some(EVE_VIDEO)} else {None}}
            />
            <div className="flex h-8 shrink-0 items-center gap-2 border-t border-white/10 bg-black px-2.5">
                <span className="block size-1.5 shrink-0 rounded-full bg-white" />
                <span className="relative h-px flex-1 bg-white/20">
                    <span className="absolute inset-y-0 left-0 bg-white/80" style={CSSProperties::new().width(format!("{}%", js::number::to_string(progress,None)))} />
                    <span className="absolute top-1/2 size-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white" style={CSSProperties::new().left(format!("{progress}%"))} />
                </span>
                <span className="font-mono text-[9px] tabular-nums text-white/55">{format_time(*current_time)}{" / "}{format_time(*duration)}</span>
            </div>
        </div>
    }
}
