use crate::scene::viewport::ProjectionMode;

pub(crate) fn symbol(mode: ProjectionMode) -> &'static str {
    match mode {
        ProjectionMode::Perspective => "Perspective",
        ProjectionMode::Orthographic => "Orthographic",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<ProjectionMode> {
    match symbol {
        "Perspective" => Some(ProjectionMode::Perspective),
        "Orthographic" => Some(ProjectionMode::Orthographic),
        _ => None,
    }
}
