//! Interactive source editor and deferred live compiler output, from shader-code-scales-demo.tsx.
use crate::shader_preview::{Dispose, render_shader_preview};
use js::Promise;
use react::event::KeyboardEvent;
use react::webapi::{
    EventTargetExt, HTMLButtonElement, HTMLCanvasElement, HTMLDivElement, HTMLPreElement,
    IntersectionObserverInit, document, events, html_element, intersection_observer, listener,
};
use react::{CSSProperties, JSX, ReactNode, RefObject, jsx, use_effect, use_ref, use_state};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
const SOURCE_FILES: [&str; 3] = ["index.ts", "main.wgsl", "color.wgsl"];
const COMPILED_SOURCE: &str = "struct U{r:vec2f}@group(0)@binding(0)var<uniform>u:U;\n@fragment fn fs_main(@location(0)b:vec2f)->@location(0)vec4f{\nlet c=(b-0.5)*(u.r/min(u.r.x,u.r.y));let d=length(c);let e=atan2(c.y,c.x)/6.28318+0.5;\nlet f=exp(-95.0*pow(abs(d-0.31),2.0));let h=exp(-18.0*pow(abs(d-0.31),2.0));\nlet i=0.82+0.18*sin(e*25.13272+d*42.0);let g=a(e+d*0.65);\nreturn vec4f(g*(f*i*2.0+h*0.22),1.0)}fn a(b:f32)->vec3f{let c=vec3f(0.0,0.33,0.67);\nreturn 0.55+0.45*cos(6.28318*(b+c))}";
struct CodeLineProps<'a> {
    children: Option<&'a dyn ReactNode>,
    highlight: Option<bool>,
    number: u32,
}
fn CodeLine(
    CodeLineProps {
        children,
        highlight,
        number,
    }: CodeLineProps,
) -> JSX::Element {
    jsx! {
        <span className={format!("relative block min-w-max pr-5 {}", if highlight.unwrap_or(false) { "-mx-3 bg-[rgba(0,202,82,0.08)] px-3 before:absolute before:inset-y-0 before:left-0 before:w-px before:bg-[#00a844] dark:bg-[rgba(0,202,82,0.12)] dark:before:bg-[#00ca52] sm:-mx-5 sm:px-5" } else { "" })}>
            <span className="mr-4 inline-block w-5 select-none text-right text-gray-700">{number}</span>
            {children.unwrap_or(&"\u{a0}")}
        </span>
    }
}
fn TypeScriptSource() -> JSX::Element {
    jsx! {
        <>
            <CodeLine number={1}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"import"}
                </span>
                {" { effect, frame, init, surface } "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"from"}
                </span>
                {" "}
                <span className="text-green-800 dark:text-[#00ca52]">
                    {"\"vgpu\""}
                </span>
                {";"}
            </CodeLine>
            <CodeLine highlight={Some(true)} number={2}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"import"}
                </span>
                {" shaderSource "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"from"}
                </span>
                {" "}
                <span className="text-green-800 dark:text-[#00ca52]">
                    {"\"./main.wgsl\""}
                </span>
                {";"}
            </CodeLine>
            <CodeLine number={3} />
            <CodeLine number={4}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"export async function"}
                </span>
                {" "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"render"}
                </span>
                {"(canvas: HTMLCanvasElement) {"}
            </CodeLine>
            <CodeLine number={5}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"const"}
                </span>
                {" gpu = "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"await"}
                </span>
                {" "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"init"}
                </span>
                {"();"}
            </CodeLine>
            <CodeLine number={6}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"const"}
                </span>
                {" output = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"surface"}
                </span>
                {"(gpu, canvas, { dpr: 1 });"}
            </CodeLine>
            <CodeLine number={7}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"const"}
                </span>
                {" shader = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"effect"}
                </span>
                {"(gpu, shaderSource);"}
            </CodeLine>
            <CodeLine number={8} />
            <CodeLine number={9}>
                {"  "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"shader.set"}
                </span>
                {"({ uniforms: { resolution: output.size } });"}
            </CodeLine>
            <CodeLine number={10}>
                {"  "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"frame"}
                </span>
                {"(gpu, (f) => f.pass(output, shader));"}
            </CodeLine>
            <CodeLine number={11}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"return"}
                </span>
                {" () => gpu."}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"dispose"}
                </span>
                {"();"}
            </CodeLine>
            <CodeLine number={12}>
                {"}"}
            </CodeLine>
        </>

    }
}
fn MainShaderSource() -> JSX::Element {
    jsx! {
        <>
            <CodeLine highlight={Some(true)} number={1}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"import"}
                </span>
                {" { palette } "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"from"}
                </span>
                {" "}
                <span className="text-green-800 dark:text-[#00ca52]">
                    {"\"./color.wgsl\""}
                </span>
                {";"}
            </CodeLine>
            <CodeLine number={2} />
            <CodeLine number={3}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"struct"}
                </span>
                {" Uniforms {"}
            </CodeLine>
            <CodeLine number={4}>
                {"  resolution: vec2f,"}
            </CodeLine>
            <CodeLine number={5}>
                {"};"}
            </CodeLine>
            <CodeLine number={6} />
            <CodeLine number={7}>
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"@group(0) @binding(0)"}
                </span>
                {" var<uniform> uniforms: Uniforms;"}
            </CodeLine>
            <CodeLine number={8} />
            <CodeLine number={9}>
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"@fragment"}
                </span>
            </CodeLine>
            <CodeLine number={10}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"fn"}
                </span>
                {" "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"fs_main"}
                </span>
                {"(@location(0) uv: vec2f) -> @location(0) vec4f {"}
            </CodeLine>
            <CodeLine number={11}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" shortest = min(uniforms.resolution.x, uniforms.resolution.y);"}
            </CodeLine>
            <CodeLine number={12}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" aspect = uniforms.resolution / shortest;"}
            </CodeLine>
            <CodeLine number={13}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" p = (uv - 0.5) * aspect;"}
            </CodeLine>
            <CodeLine number={14}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" radius = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"length"}
                </span>
                {"(p);"}
            </CodeLine>
            <CodeLine number={15}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" angle = atan2(p.y, p.x) / 6.28318 + 0.5;"}
            </CodeLine>
            <CodeLine number={16}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" ring = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"exp"}
                </span>
                {"(-95.0 * pow(abs(radius - 0.31), 2.0));"}
            </CodeLine>
            <CodeLine number={17}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" halo = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"exp"}
                </span>
                {"(-18.0 * pow(abs(radius - 0.31), 2.0));"}
            </CodeLine>
            <CodeLine number={18}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" bands = 0.82 + 0.18 * sin(angle * 25.13272 + radius * 42.0);"}
            </CodeLine>
            <CodeLine number={19}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" color = "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"palette"}
                </span>
                {"(angle + radius * 0.65);"}
            </CodeLine>
            <CodeLine number={20}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"return"}
                </span>
                {" vec4f(color * (ring * bands * 2.0 + halo * 0.22), 1.0);"}
            </CodeLine>
            <CodeLine number={21}>
                {"}"}
            </CodeLine>
        </>

    }
}
fn ColorShaderSource() -> JSX::Element {
    jsx! {
        <>
            <CodeLine number={1}>
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"export fn"}
                </span>
                {" "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"palette"}
                </span>
                {"(t: f32) -> vec3f {"}
            </CodeLine>
            <CodeLine number={2}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"let"}
                </span>
                {" phase = vec3f(0.0, 0.33, 0.67);"}
            </CodeLine>
            <CodeLine number={3}>
                {"  "}
                <span className="text-red-800 dark:text-[#ff518d]">
                    {"return"}
                </span>
                {" 0.55 + 0.45 * "}
                <span className="text-purple-800 dark:text-[#c472fb]">
                    {"cos"}
                </span>
                {"(6.28318 * (t + phase));"}
            </CodeLine>
            <CodeLine number={4}>
                {"}"}
            </CodeLine>
        </>

    }
}
fn SourceEditor() -> JSX::Element {
    let (active_file, set_active_file) = use_state(|| "index.ts");
    let code_ref = use_ref(None::<&'static HTMLPreElement>);
    use_effect(
        move || {
            if let Some(code) = code_ref.current() {
                code.set_scroll_top(0.0);
            }
        },
        Some(&(active_file,)),
    );
    let handle_tab_key = move |event: &KeyboardEvent<HTMLButtonElement>, index: usize| {
        let next_index = match event.key().as_str() {
            "ArrowRight" => (index + 1) % SOURCE_FILES.len(),
            "ArrowLeft" => (index + SOURCE_FILES.len() - 1) % SOURCE_FILES.len(),
            "Home" => 0,
            "End" => SOURCE_FILES.len() - 1,
            _ => return,
        };
        event.prevent_default();
        let next_file = SOURCE_FILES[next_index];
        set_active_file.set(next_file);
        if let Some(element) = document.get_element_by_id(&format!("shader-source-tab-{next_file}"))
        {
            html_element::unchecked_from(element).focus(None);
        }
    };
    jsx! {
        <section className="flex min-h-[28rem] min-w-0 flex-col bg-background-100 lg:min-h-[34rem]">
            <div aria-label="TypeScript and shader source files" className="flex h-12 shrink-0 items-stretch border-b border-gray-alpha-400 bg-background-200" role="tablist">
                {SOURCE_FILES.iter().copied().enumerate().map(|(index,file)| {
                    let active = file == *active_file;
                    jsx! {
                        <button aria-controls="shader-source-panel" aria-selected={active}
                            className={format!("interactive-tab relative shrink-0 border-r border-gray-alpha-400 px-3 font-mono text-xs transition-colors focus-visible:outline focus-visible:outline-1 focus-visible:outline-gray-1000 focus-visible:outline-offset-[-2px] sm:px-4 {}", if active {"text-gray-1000 after:absolute after:inset-x-0 after:bottom-0 after:h-px after:bg-gray-1000"} else {"text-gray-800 hover:text-gray-1000"})}
                            id={format!("shader-source-tab-{file}")} key={file} onClick={move |_| set_active_file.set(file)} onKeyDown={move |event| handle_tab_key(event,index)} role="tab" tabIndex={if active {0} else {-1}} type="button">{file}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <pre aria-labelledby={format!("shader-source-tab-{}",*active_file)} className="min-h-0 flex-1 overflow-auto px-3 py-5 font-mono text-[12px] leading-[1.65] text-gray-1000 sm:px-5 sm:text-[13px]" id="shader-source-panel" ref={code_ref} role="tabpanel">
                {if *active_file=="index.ts" {Some(jsx!{<TypeScriptSource />})} else {None}}
                {if *active_file=="main.wgsl" {Some(jsx!{<MainShaderSource />})} else {None}}
                {if *active_file=="color.wgsl" {Some(jsx!{<ColorShaderSource />})} else {None}}
            </pre>
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-gray-alpha-400 px-4 py-3 font-mono text-[10px] uppercase tracking-[0.12em] text-gray-800">
                <span>{"resolve imports"}</span><span aria-hidden="true">{"→"}</span><span>{"remove unused"}</span><span aria-hidden="true">{"→"}</span><span>{"minify"}</span>
            </div>
        </section>
    }
}
fn ResultPoster() -> JSX::Element {
    jsx! {
        <div aria-hidden="true" className="absolute inset-0 overflow-hidden bg-black">
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,#2b1740_0%,#11121d_40%,#000_72%)]" />
            <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-90 blur-[0.3px]" style={CSSProperties::new()
                .background("conic-gradient(from 20deg, #ff518d, #ffb347, #00ca52, #29b6f6, #8b5cf6, #ff518d)")
                .height("min(68cqw, 68cqh)")
                .mask_image("radial-gradient(circle, transparent 0 49%, black 55% 64%, transparent 70%)")
                .webkit_mask_image("radial-gradient(circle, transparent 0 49%, black 55% 64%, transparent 70%)")
                .width("min(68cqw, 68cqh)")} />
        </div>
    }
}
struct CompilerOutputProps {
    root_ref: RefObject<Option<&'static HTMLDivElement>>,
}
fn CompilerOutput(CompilerOutputProps { root_ref }: CompilerOutputProps) -> JSX::Element {
    let canvas_ref = use_ref(None::<&'static HTMLCanvasElement>);
    let (is_near, set_is_near) = use_state(|| false);
    let (is_page_visible, set_is_page_visible) = use_state(|| true);
    let (preview_state, set_preview_state) = use_state(|| "poster");
    let is_active = *is_near && *is_page_visible;
    use_effect(
        move || {
            let root = root_ref.current();
            if root.is_none() || !js::is_defined(intersection_observer::new) {
                set_is_near.set(true);
                return None;
            }
            let observer = intersection_observer::new(
                Box::new(move |entries, _| set_is_near.set(entries[0].is_intersecting())),
                Some(IntersectionObserverInit {
                    root_margin: Some("320px 0px"),
                    ..Default::default()
                }),
            );
            observer.observe(root.expect("The root was checked."));
            Some(move || observer.disconnect())
        },
        Some(&(root_ref,)),
    );
    use_effect(
        move || {
            let sync_visibility =
                listener(move |_| set_is_page_visible.set(document.visibility_state() != "hidden"));
            set_is_page_visible.set(document.visibility_state() != "hidden");
            document.add_event_listener(events::Visibilitychange, sync_visibility, None);
            move || document.remove_event_listener(events::Visibilitychange, sync_visibility, None)
        },
        Some(&()),
    );
    use_effect(
        move || {
            let canvas = canvas_ref.current()?;
            if !is_active {
                return None;
            }
            let disposed = Rc::new(Cell::new(false));
            let stop: Rc<RefCell<Option<Dispose>>> = Rc::new(RefCell::new(None));
            set_preview_state.set("loading");
            let initialized = Rc::clone(&disposed);
            let failed = Rc::clone(&disposed);
            let retained = Rc::clone(&stop);
            let loading: Promise<Dispose> =
                js::import!(render_shader_preview).then(move |render| js::promise(render(canvas)));
            let task: Promise<()> = loading.then(move |dispose_renderer| {
                if initialized.get() {
                    dispose_renderer();
                } else {
                    *retained.borrow_mut() = Some(dispose_renderer);
                    set_preview_state.set("running");
                }
            });
            let _: Promise<()> = task.catch(move |_| {
                if !failed.get() {
                    set_preview_state.set("unavailable");
                }
            });
            Some(move || {
                disposed.set(true);
                if let Some(stop) = stop.borrow().as_ref() {
                    stop();
                }
            })
        },
        Some(&(is_active,)),
    );
    jsx! {
        <section className="relative flex min-w-0 flex-col border-t border-gray-alpha-400 bg-background-100 lg:border-l lg:border-t-0">
            <span aria-hidden="true" className="absolute left-1/2 top-0 z-20 grid size-7 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full border border-gray-alpha-500 bg-background-100 font-mono text-xs text-gray-900 lg:left-0 lg:top-1/2"><span className="lg:hidden">{"↓"}</span><span className="hidden lg:inline">{"→"}</span></span>
            <div className="flex h-12 shrink-0 items-center border-b border-gray-alpha-400 px-4"><span className="font-mono text-xs text-gray-1000">{"result"}</span></div>
            <div aria-label="Rainbow ring rendered from the compiled shader modules" className="relative aspect-[16/10] min-h-[17rem] overflow-hidden bg-black [container-type:size]" role="img">
                <ResultPoster />
                <canvas aria-hidden="true" className={format!("absolute inset-0 size-full transition-opacity duration-500 {}",if *preview_state=="running" {"opacity-100"} else {"opacity-0"})} ref={canvas_ref} />
            </div>
            <div className="flex h-12 items-center justify-between border-y border-gray-alpha-400 px-4"><span className="font-mono text-xs text-gray-1000">{"compiled.wgsl"}</span><span className="font-mono text-[10px] uppercase tracking-[0.12em] text-gray-800">{"397 B"}</span></div>
            <pre className="min-h-36 flex-1 overflow-auto whitespace-pre-wrap break-all px-4 py-4 font-mono text-[10px] leading-5 text-gray-900 sm:text-[11px]"><code>{COMPILED_SOURCE}</code></pre>
        </section>
    }
}
pub fn ShaderCodeScalesDemo() -> JSX::Element {
    let root_ref = use_ref(None::<&'static HTMLDivElement>);
    jsx! {
        <div className="relative grid overflow-hidden rounded-card border border-gray-alpha-400 bg-background-100 shadow-[0_32px_100px_rgba(0,0,0,0.12)] lg:grid-cols-[1.15fr_0.85fr]" ref={root_ref}>
            <SourceEditor /><CompilerOutput rootRef={root_ref} />
        </div>
    }
}
