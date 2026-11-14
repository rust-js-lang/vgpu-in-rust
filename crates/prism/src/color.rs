//! Shared strict hexadecimal color parsing and the dark wall's sRGB transfer.
use crate::controls::PrismView;
pub fn hex_color(value: &str) -> Option<[f64; 3]> {
    let text = value.strip_prefix('#').unwrap_or(value);
    if text.len() != 6 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(std::array::from_fn(|index| {
        u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).expect("hex channel") as f64 / 255.0
    }))
}
pub fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
pub fn dark_wall_clear(wall_color: &str, view: PrismView) -> [f64; 4] {
    if view == PrismView::Caustic {
        return [0.0, 0.0, 0.0, 1.0];
    }
    match hex_color(wall_color) {
        Some([r, g, b]) => [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), 1.0],
        None => [0.0, 0.0, 0.0, 1.0],
    }
}
