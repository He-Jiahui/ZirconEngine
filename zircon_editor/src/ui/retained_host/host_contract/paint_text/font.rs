use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

const FALLBACK_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/FiraMono-subset.ttf"
));
const UI_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/FiraSans-Regular.ttf"
));

pub(in crate::ui::retained_host::host_contract) fn fallback_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(UI_FONT_BYTES, FontSettings::default())
            .or_else(|_| Font::from_bytes(FALLBACK_FONT_BYTES, FontSettings::default()))
            .expect("embedded editor host font")
    })
}

pub(in crate::ui::retained_host::host_contract) fn measure_fallback_text_width(
    text: &str,
    font_size: f32,
) -> f32 {
    if text.is_empty() || !font_size.is_finite() || font_size <= 0.0 {
        return 0.0;
    }

    text.chars()
        .map(|glyph| fallback_font().metrics(glyph, font_size).advance_width)
        .filter(|advance| advance.is_finite() && *advance > 0.0)
        .sum()
}
