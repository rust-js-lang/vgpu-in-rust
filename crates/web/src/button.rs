//! Original landing button: CVA classes, Tailwind merging and Radix Slot.
use react::attributes::ButtonHTMLAttributes;
use react::{ElementType, JSX, ReactNode, element_type, jsx};
#[derive(Clone, Copy)]
pub enum ButtonVariant {
    #[cfg_attr(rust_js, rust_js::name = "default")]
    Default,
    #[cfg_attr(rust_js, rust_js::name = "destructive")]
    Destructive,
    #[cfg_attr(rust_js, rust_js::name = "outline")]
    Outline,
    #[cfg_attr(rust_js, rust_js::name = "secondary")]
    Secondary,
    #[cfg_attr(rust_js, rust_js::name = "ghost")]
    Ghost,
    #[cfg_attr(rust_js, rust_js::name = "link")]
    Link,
}
#[derive(Clone, Copy)]
pub enum ButtonSize {
    #[cfg_attr(rust_js, rust_js::name = "default")]
    Default,
    #[cfg_attr(rust_js, rust_js::name = "sm")]
    Sm,
    #[cfg_attr(rust_js, rust_js::name = "lg")]
    Lg,
    #[cfg_attr(rust_js, rust_js::name = "icon")]
    Icon,
    #[cfg_attr(rust_js, rust_js::name = "icon-sm")]
    IconSm,
    #[cfg_attr(rust_js, rust_js::name = "icon-lg")]
    IconLg,
}
pub struct ButtonVariantValues<'a> {
    pub variant: Option<ButtonVariant>,
    pub size: Option<ButtonSize>,
    pub class_name: Option<&'a str>,
}
struct VariantClasses {
    default: &'static str,
    destructive: &'static str,
    outline: &'static str,
    secondary: &'static str,
    ghost: &'static str,
    link: &'static str,
}
struct SizeClasses {
    default: &'static str,
    sm: &'static str,
    lg: &'static str,
    icon: &'static str,
    #[cfg_attr(rust_js, rust_js::name = "icon-sm")]
    icon_sm: &'static str,
    #[cfg_attr(rust_js, rust_js::name = "icon-lg")]
    icon_lg: &'static str,
}
struct VariantMap {
    variant: VariantClasses,
    size: SizeClasses,
}
struct VariantDefaults {
    variant: ButtonVariant,
    size: ButtonSize,
}
struct CvaConfig {
    variants: VariantMap,
    default_variants: VariantDefaults,
}
#[cfg_attr(rust_js, rust_js::link_name = "class-variance-authority#cva")]
fn cva(_base: &str, _config: CvaConfig) -> &'static dyn Fn(ButtonVariantValues<'_>) -> String {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "tailwind-merge#twMerge")]
fn tw_merge(_classes: &str) -> String {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "get radix-ui#Slot.Slot")]
fn slot_component() -> ElementType {
    unreachable!()
}
thread_local! {
    pub static BUTTON_VARIANTS: &'static dyn Fn(ButtonVariantValues<'_>)->String = cva(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-gray-600 focus-visible:ring-gray-600/50 focus-visible:ring-[3px] aria-invalid:ring-red-800/20 dark:aria-invalid:ring-red-800/40 aria-invalid:border-red-800",
        CvaConfig {
            variants:VariantMap {
                variant:VariantClasses {
                    default:"bg-gray-1000 text-background-100 hover:bg-gray-1000/90",
                    destructive:"bg-red-800 text-white hover:bg-red-800/90 focus-visible:ring-red-800/20 dark:focus-visible:ring-red-800/40 dark:bg-red-800/60",
                    outline:"border bg-background-100 shadow-xs hover:bg-gray-100 hover:text-gray-1000 dark:bg-gray-alpha-400/30 dark:border-gray-alpha-400 dark:hover:bg-gray-alpha-400/50",
                    secondary:"bg-gray-100 text-gray-1000 hover:bg-gray-100/80",
                    ghost:"hover:bg-gray-100 hover:text-gray-1000 dark:hover:bg-gray-100/50",
                    link:"text-gray-1000 underline-offset-4 hover:underline",
                },
                size:SizeClasses {
                    default:"h-9 px-4 py-2 has-[>svg]:px-3",sm:"h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5",lg:"h-10 rounded-md px-6 has-[>svg]:px-4",icon:"size-9",icon_sm:"size-8",icon_lg:"size-10",
                },
            },
            default_variants:VariantDefaults {variant:ButtonVariant::Default,size:ButtonSize::Default},
        },
    );
}
pub struct ButtonProps<'a, C> {
    pub class_name: Option<&'a str>,
    pub variant: Option<ButtonVariant>,
    pub size: Option<ButtonSize>,
    #[cfg_attr(rust_js, rust_js::default = false)]
    pub as_child: bool,
    pub children: C,
    #[cfg_attr(rust_js, rust_js::flatten)]
    pub attrs: ButtonHTMLAttributes<'a>,
}
pub fn Button<C: ReactNode>(
    ButtonProps {
        class_name,
        variant,
        size,
        as_child,
        children,
        attrs,
    }: ButtonProps<C>,
) -> JSX::Element {
    let Comp = if as_child {
        slot_component()
    } else {
        element_type("button")
    };
    let classes = BUTTON_VARIANTS.with(|button_variants| {
        tw_merge(&button_variants(ButtonVariantValues {
            variant,
            size,
            class_name,
        }))
    });
    jsx! {<Comp data-slot="button" className={classes} {...attrs}>{children}</Comp>}
}
