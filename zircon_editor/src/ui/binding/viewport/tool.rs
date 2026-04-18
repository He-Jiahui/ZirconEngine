use zircon_scene::SceneViewportTool;

pub(crate) fn symbol(tool: SceneViewportTool) -> &'static str {
    match tool {
        SceneViewportTool::Drag => "Drag",
        SceneViewportTool::Move => "Move",
        SceneViewportTool::Rotate => "Rotate",
        SceneViewportTool::Scale => "Scale",
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<SceneViewportTool> {
    match symbol {
        "Drag" => Some(SceneViewportTool::Drag),
        "Move" => Some(SceneViewportTool::Move),
        "Rotate" => Some(SceneViewportTool::Rotate),
        "Scale" => Some(SceneViewportTool::Scale),
        _ => None,
    }
}
