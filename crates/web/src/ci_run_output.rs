//! Animated CI transcript from ci-run-output.tsx.
use react::webapi::{HTMLDivElement, IntersectionObserverInit, intersection_observer, window};
use react::{JSX, jsx, use_effect, use_ref, use_state};
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Command,
    Step,
    Success,
}
struct CiLine {
    label: &'static str,
    tone: Tone,
}
const CI_LINES: [CiLine; 5] = [
    CiLine {
        label: "$ pnpm test:render",
        tone: Tone::Command,
    },
    CiLine {
        label: "compile eve.wgsl",
        tone: Tone::Step,
    },
    CiLine {
        label: "render headless frame",
        tone: Tone::Step,
    },
    CiLine {
        label: "compare snapshot",
        tone: Tone::Step,
    },
    CiLine {
        label: "1 test passed · 842ms",
        tone: Tone::Success,
    },
];
const STEP_DELAY_MS: f64 = 650.0;
const COMPLETE_DELAY_MS: f64 = 1800.0;
pub fn CiRunOutput() -> JSX::Element {
    let root_ref = use_ref(None::<&'static HTMLDivElement>);
    let (active, set_active) = use_state(|| false);
    let (visible_count, set_visible_count) = use_state(|| 1_usize);
    use_effect(
        move || {
            if window
                .match_media("(prefers-reduced-motion: reduce)")
                .matches()
            {
                set_visible_count.set(CI_LINES.len());
                return None;
            }
            let root = root_ref.current();
            if root.is_none() || !js::is_defined(intersection_observer::new) {
                set_active.set(true);
                return None;
            }
            let observer = intersection_observer::new(
                Box::new(move |entries, _| set_active.set(entries[0].is_intersecting())),
                Some(IntersectionObserverInit {
                    threshold: Some(0.35.into()),
                    ..Default::default()
                }),
            );
            observer.observe(root.expect("The root was checked above."));
            Some(move || observer.disconnect())
        },
        Some(&()),
    );
    use_effect(
        move || {
            if !*active {
                return None;
            }
            let complete = *visible_count == CI_LINES.len();
            let timeout = js::set_timeout(
                move || {
                    set_visible_count.update(|current| {
                        if *current == CI_LINES.len() {
                            1
                        } else {
                            current + 1
                        }
                    })
                },
                Some(if complete {
                    COMPLETE_DELAY_MS
                } else {
                    STEP_DELAY_MS
                }),
            );
            Some(move || js::clear_timeout(Some(timeout)))
        },
        Some(&(active, visible_count)),
    );
    jsx! {
        <div className="flex aspect-video items-center bg-background-100 px-4 font-mono text-[9px] leading-[1.65] sm:text-[10px]" ref={root_ref}>
            <p className="sr-only">{"CI runs the render test, compiles the shader, renders a headless frame, compares the snapshot, and passes."}</p>
            <div aria-hidden="true" className="w-full space-y-0.5">
                {CI_LINES.iter().enumerate().map(|(index,line)| {
                    let visible=index<*visible_count;
                    let success=line.tone==Tone::Success;
                    jsx! {
                        <p className={format!("truncate transition-opacity duration-300 {} {}",if visible {"opacity-100"} else {"opacity-0"},if success {"text-green-800 dark:text-[#00ca52]"} else if line.tone==Tone::Command {"text-gray-1000"} else {"text-gray-900"})} key={line.label}>
                            {if line.tone==Tone::Step {Some("✓ ")} else {None}}
                            {if success {Some("● ")} else {None}}
                            {line.label}
                        </p>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
