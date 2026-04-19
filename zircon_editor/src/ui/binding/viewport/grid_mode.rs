use zircon_framework::render::GridMode;

pub(crate) fn symbol(mode: GridMode) -> &'static str {
    match mode {
        GridMode::Hidden => "Hidden",
        GridMode::VisibleNoSnap => "VisibleNoSnap",
        GridMode::VisibleAndSnap => "VisibleAndSnap",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<GridMode> {
    match symbol {
        "Hidden" => Some(GridMode::Hidden),
        "VisibleNoSnap" => Some(GridMode::VisibleNoSnap),
        "VisibleAndSnap" => Some(GridMode::VisibleAndSnap),
        _ => None,
    }
}
