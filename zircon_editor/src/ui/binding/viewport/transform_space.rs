use crate::scene::viewport::TransformSpace;

pub(crate) fn symbol(space: TransformSpace) -> &'static str {
    match space {
        TransformSpace::Local => "Local",
        TransformSpace::Global => "Global",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<TransformSpace> {
    match symbol {
        "Local" => Some(TransformSpace::Local),
        "Global" => Some(TransformSpace::Global),
        _ => None,
    }
}
