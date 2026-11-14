//! Shared hero-tab state from components/hero/hero-tab-state.tsx.
use react::{Context, JSX, ReactNode, create_context, jsx, use_context, use_memo, use_state};

#[derive(Clone, Copy, PartialEq)]
pub enum HeroTab {
    Prompt,
    #[cfg_attr(rust_js, rust_js::name = "CLI")]
    Cli,
    Skill,
    #[cfg_attr(rust_js, rust_js::name = "MCP")]
    Mcp,
}

pub struct HeroTabState {
    pub active_tab: HeroTab,
    pub set_active_tab: &'static dyn Fn(HeroTab),
}

thread_local! {
    static HERO_TAB_CONTEXT: Context<Option<&'static HeroTabState>> = create_context(None);
}

pub struct HeroTabProviderProps<C> {
    pub children: C,
}

pub fn HeroTabProvider<C: ReactNode>(
    HeroTabProviderProps { children }: HeroTabProviderProps<C>,
) -> JSX::Element {
    let (active_tab, set_active_tab) = use_state(|| HeroTab::Prompt);
    let value = use_memo(
        move || HeroTabState {
            active_tab: *active_tab,
            set_active_tab: &**set_active_tab,
        },
        (active_tab,),
    );
    jsx! {
        <HERO_TAB_CONTEXT.Provider value={Some(value)}>{children}</HERO_TAB_CONTEXT.Provider>
    }
}

pub fn use_hero_tab() -> &'static HeroTabState {
    use_context(&HERO_TAB_CONTEXT).expect("useHeroTab must be used inside HeroTabProvider.")
}
