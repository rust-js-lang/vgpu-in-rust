//! Inline literal text rendering, from the upstream landing's inline-code.tsx.
use react::{JSX, jsx};

/// Strips backtick markers, giving the plain text to put on the clipboard.
pub fn strip_backticks(text: &str) -> String {
    text.replace('`', "")
}

pub struct InlineCodeProps<'a> {
    pub text: &'a str,
    #[cfg_attr(rust_js, rust_js::default = true)]
    pub mono: bool,
    #[cfg_attr(rust_js, rust_js::default = false)]
    pub wrap: bool,
}

/// Renders a string where backtick-wrapped spans are set in mono.
///
/// Commands quoted mid-sentence must read as literal without breaking the
/// line. Geist Mono runs slightly wider and taller than Geist Sans at the
/// same px size, so code is nudged to 0.95em. Long standalone commands may
/// opt into wrapping when the containing layout is intentionally narrow.
///
/// Fence only what is literal: mono tells the reader to type the characters
/// exactly. A command merely named in prose need not be presented as code.
pub fn InlineCode(InlineCodeProps { text, mono, wrap }: InlineCodeProps) -> JSX::Element {
    // Odd indices are the fenced spans: "a `b` c" -> ["a ", "b", " c"].
    jsx! {
        <>
            {text.split('`').enumerate().map(|(index, part)| {
                if index % 2 == 0 {
                    jsx! { <Fragment key={index}>{part}</Fragment> }
                } else if mono {
                    // Fencing marks an atomic run; mono decides whether it
                    // is also presented as code. Prompt needs the former only.
                    // Long standalone commands may wrap at their own spaces.
                    jsx! {
                        <code key={index} className={format!("{} font-mono text-[0.95em]", if wrap { "whitespace-normal" } else { "whitespace-nowrap" })}>
                            {part}
                        </code>
                    }
                } else {
                    jsx! {
                        <span key={index} className={if wrap { "whitespace-normal" } else { "whitespace-nowrap" }}>
                            {part}
                        </span>
                    }
                }
            }).collect::<Vec<_>>()}
        </>
    }
}
