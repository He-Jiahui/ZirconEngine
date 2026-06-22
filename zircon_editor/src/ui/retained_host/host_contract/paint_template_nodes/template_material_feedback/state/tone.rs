use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;

pub(super) fn material_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    let tone = [node.validation_level.as_str(), node.text_tone.as_str()]
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("");
    match tone {
        "warning" => Some(PALETTE.warning),
        "error" | "danger" => Some(PALETTE.error),
        "success" => Some(PALETTE.success),
        "info" => Some(PALETTE.info),
        "accent" | "primary" => Some(PALETTE.accent),
        _ => None,
    }
}
