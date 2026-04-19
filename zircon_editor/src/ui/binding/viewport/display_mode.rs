use zircon_framework::render::DisplayMode;

pub(crate) fn symbol(mode: DisplayMode) -> &'static str {
    match mode {
        DisplayMode::Shaded => "Shaded",
        DisplayMode::WireOverlay => "WireOverlay",
        DisplayMode::WireOnly => "WireOnly",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<DisplayMode> {
    match symbol {
        "Shaded" => Some(DisplayMode::Shaded),
        "WireOverlay" => Some(DisplayMode::WireOverlay),
        "WireOnly" => Some(DisplayMode::WireOnly),
        _ => None,
    }
}
