use zircon_framework::render::ViewOrientation;

pub(crate) fn symbol(orientation: ViewOrientation) -> &'static str {
    match orientation {
        ViewOrientation::User => "User",
        ViewOrientation::PosX => "PosX",
        ViewOrientation::NegX => "NegX",
        ViewOrientation::PosY => "PosY",
        ViewOrientation::NegY => "NegY",
        ViewOrientation::PosZ => "PosZ",
        ViewOrientation::NegZ => "NegZ",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<ViewOrientation> {
    match symbol {
        "User" => Some(ViewOrientation::User),
        "PosX" => Some(ViewOrientation::PosX),
        "NegX" => Some(ViewOrientation::NegX),
        "PosY" => Some(ViewOrientation::PosY),
        "NegY" => Some(ViewOrientation::NegY),
        "PosZ" => Some(ViewOrientation::PosZ),
        "NegZ" => Some(ViewOrientation::NegZ),
        _ => None,
    }
}
