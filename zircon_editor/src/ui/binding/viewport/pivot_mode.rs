use crate::scene::viewport::PivotMode;

pub(crate) fn symbol(mode: PivotMode) -> &'static str {
    match mode {
        PivotMode::Primary => "Primary",
        PivotMode::Centroid => "Centroid",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<PivotMode> {
    match symbol {
        "Primary" => Some(PivotMode::Primary),
        "Centroid" => Some(PivotMode::Centroid),
        _ => None,
    }
}
