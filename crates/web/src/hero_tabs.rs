//! Setup tabs from the upstream landing's hero-tabs.tsx.
use js::{TimeoutId, clear_timeout, promise, set_timeout, settle, spawn};
use react::{JSX, jsx, use_effect, use_ref, use_state, webapi::window};

use crate::hero_tab_state::{HeroTab, use_hero_tab};
use crate::inline_code::{InlineCode, strip_backticks};

/// `mono` distinguishes terminal commands from the natural-language prompt.
struct TabContent {
    text: &'static str,
    mono: bool,
    wrap: bool,
}

impl HeroTab {
    fn label(self) -> &'static str {
        match self {
            Self::Prompt => "Prompt",
            Self::Cli => "CLI",
            Self::Skill => "Skill",
            Self::Mcp => "MCP",
        }
    }

    fn content(self) -> TabContent {
        match self {
            Self::Prompt => TabContent {
                text: "Setup vgpu on my project, run `npx vgpu`",
                mono: false,
                wrap: false,
            },
            Self::Cli => TabContent {
                text: "`pnpm add vgpu`",
                mono: true,
                wrap: false,
            },
            Self::Skill => TabContent {
                text: "`npx skills add vercel-labs/vgpu`",
                mono: true,
                wrap: false,
            },
            Self::Mcp => TabContent {
                text: "`npx -y add-mcp https://vgpu.sh/api/mcp -g`",
                mono: true,
                wrap: true,
            },
        }
    }
}

const TABS: [HeroTab; 4] = [HeroTab::Prompt, HeroTab::Cli, HeroTab::Skill, HeroTab::Mcp];

/// How long the outgoing snippet takes to clear, in ms. The incoming one settles
/// over 200ms (`duration-200` below): the outgoing line leaving a touch faster
/// reads as a handover rather than a dissolve.
///
/// Keep this in step with the `duration-150` on the leaving layer — it is what
/// decides when that layer is re-parked below the line, and re-parking early
/// would cut the fade short.
const LEAVE_MS: f64 = 150.0;

struct IconProps<'a> {
    class_name: &'a str,
}

#[cfg_attr(rust_js, rust_js::link_name = "lucide-react#Check")]
fn Check(_props: IconProps) -> JSX::Element {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "lucide-react#Copy")]
fn Copy(_props: IconProps) -> JSX::Element {
    unreachable!()
}

/// Hero setup snippet: a text-only tab switcher over a hairline rule.
///
/// Deliberately not a card — it sits directly on the shader, so the only chrome
/// is the divider. Copy is the whole line rather than a button: the icon is just
/// an affordance that fades in on hover, and confirmation is a swap to a check.
pub fn HeroTabs() -> JSX::Element {
    let state = use_hero_tab();
    let active_tab = state.active_tab;
    // The tab that is currently on its way out. Every other inactive tab is
    // "parked" below the line, which is what makes the next one rise from below.
    let (leaving_tab, set_leaving_tab) = use_state(|| None::<HeroTab>);
    let (copied, set_copied) = use_state(|| false);
    let park_timer = use_ref(None::<&'static TimeoutId>);
    let content = active_tab.content().text;

    let select_tab = move |tab: HeroTab| {
        if tab == active_tab {
            return;
        }
        set_leaving_tab.set(Some(active_tab));
        (state.set_active_tab)(tab);
        // Re-park the outgoing layer below the line once it has faded, so it rises
        // again on its next turn instead of dropping in from above. It is fully
        // transparent by then, so the jump back down is invisible. Restarting the
        // timer on every switch is also the guard against fast alternating clicks:
        // a layer is only ever re-parked after it has actually finished leaving.
        clear_timeout(park_timer.current());
        park_timer.set_current(Some(set_timeout(
            move || set_leaving_tab.set(None),
            Some(LEAVE_MS),
        )));
    };

    use_effect(
        move || move || clear_timeout(park_timer.current()),
        Some(&()),
    );

    let copy = move || {
        spawn(Box::new(async move {
            // Enclose the entire async operation: unavailable clipboard access can
            // throw before writeText returns its promise, as well as reject later.
            let _ = settle(promise(async move {
                window
                    .navigator()
                    .clipboard()
                    .write_text(&strip_backticks(content))
                    .await;
                set_copied.set(true);
                set_timeout(move || set_copied.set(false), Some(1600.0));
            }))
            .await;
            // Clipboard can be blocked (insecure origin, denied permission). The
            // snippet is on screen and selectable, so failing silently is fine.
        }));
    };

    // No `gap` on the column: the rule carries its own asymmetric margins (see
    // below), and a gap would add to both sides equally.
    jsx! {
        <div data-hero-tabs={true} className="flex w-full flex-col">
            <div data-hero-tabs-list={true} role="tablist" aria-label="Setup option"
                className="flex items-center justify-center text-[15px] leading-none lg:text-[16px]">
                {TABS.iter().copied().enumerate().map(|(index, tab)| jsx! {
                    <Fragment key={tab.label()}>
                        {if index > 0 { Some(jsx! { <span aria-hidden={true} className="px-2">"·"</span> }) } else { None }}
                        <button type="button" role="tab" aria-selected={active_tab == tab}
                            onClick={move |_| select_tab(tab)}
                            className={format!("text-current transition-opacity {}", if active_tab == tab { "opacity-100" } else { "opacity-50 hover:opacity-80" })}>
                            {tab.label()}
                        </button>
                    </Fragment>
                }).collect::<Vec<_>>()}
            </div>

            // Hairline rule that dissolves at both ends. A hard-edged 1px line reads as
            // a UI seam pinned over the shader; fading the ends lets it sit in the
            // image instead. Gradient rather than a border since a border can't taper.
            //
            // No flat middle on purpose: sampling the rule in the Figma reference gives
            // a symmetric triangle (alpha .57/.98/.56 at 25/50/75%), i.e. it peaks at
            // #4D4D4D dead centre and ramps straight down to both ends. A solid centre
            // band reads noticeably heavier than the reference.
            // Asymmetric margins, not a uniform column gap: the rule belongs to the
            // snippet below it, so it sits closer to that than to the tabs above.
            //
            // The two numbers are not the two visual gaps. The tabs run `leading-none`
            // (box hugs the glyphs) while the snippet runs `leading-relaxed` (~5px of
            // half-leading above its ink), so an equal margin already reads as a
            // bigger gap underneath. mt-5/mb-2 = 20/8px of box spacing lands at
            // roughly 25/14px of measured ink-to-ink spacing.
            <div data-hero-tabs-rule={true} aria-hidden={true} className="mb-2 mt-5 h-px w-full" />

            <button data-hero-tabs-copy={true} type="button" onClick={move |_| copy()}
                aria-label={if *copied { "Copied".to_string() } else { format!("Copy: {}", strip_backticks(content)) }}
                className="group relative w-full px-7 text-center text-[15px] leading-relaxed text-current opacity-90 transition-opacity hover:opacity-100 lg:text-[16px]">
                // Every snippet is rendered, stacked in one grid cell, and crossfaded.
                //
                // Stacking rather than swapping a single node is what keeps the block
                // rigid: the cell is always as tall as the LONGEST snippet, so neither
                // the rule above nor the tagline this is centred with can move — not
                // during the transition, and not between tabs with different content
                // lengths.
                //
                // Three states instead of active/inactive, because direction matters:
                // the one leaving lifts UP and the one arriving rises from BELOW, so a
                // tab that is merely idle has to wait below the line. Plain transitions
                // (no keyframes) so an interrupted switch interpolates from wherever it
                // got to instead of snapping back to 0% — mash the tabs and it stays
                // smooth.
                <span className="grid">
                    {TABS.iter().copied().map(|tab| {
                        let content = tab.content();
                        let transition = if tab == active_tab {
                            "translate-y-0 opacity-100 transition duration-200 ease-out"
                        } else if Some(tab) == *leaving_tab {
                            "-translate-y-1.5 select-none opacity-0 transition duration-150 ease-out"
                        } else {
                            // Parked below with no transition; re-parking is invisible.
                            "translate-y-1.5 select-none opacity-0"
                        };
                        jsx! {
                            // motion-reduce drops transitions, retaining the same end states.
                            <span key={tab.label()} aria-hidden={tab != active_tab}
                                className={format!("col-start-1 row-start-1 motion-reduce:transition-none {transition}")}>
                                <InlineCode text={content.text} mono={content.mono} wrap={content.wrap} />
                            </span>
                        }
                    }).collect::<Vec<_>>()}
                </span>
                <span aria-hidden={true}
                    className={format!("absolute right-0 top-1/2 -translate-y-1/2 transition-opacity {}", if *copied { "opacity-90" } else { "opacity-0 group-hover:opacity-50" })}>
                    {if *copied { jsx! { <Check className="h-3.5 w-3.5" /> } } else { jsx! { <Copy className="h-3.5 w-3.5" /> } }}
                </span>
            </button>
        </div>
    }
}
