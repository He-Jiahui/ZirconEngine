use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

const FALLBACK_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/FiraMono-subset.ttf"
));

pub(in crate::ui::retained_host::host_contract) fn fallback_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(FALLBACK_FONT_BYTES, FontSettings::default())
            .expect("embedded editor host font")
    })
}
