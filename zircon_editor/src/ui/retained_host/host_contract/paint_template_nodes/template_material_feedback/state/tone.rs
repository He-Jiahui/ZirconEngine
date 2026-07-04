use super::super::super::super::data::TemplatePaneNodeData;
use super::super::palette::MaterialFeedbackPalette;

pub(super) fn material_tone_color(
    node: &TemplatePaneNodeData,
    palette: &MaterialFeedbackPalette,
) -> Option<[u8; 4]> {
    let tone = [node.validation_level.as_str(), node.text_tone.as_str()]
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("");
    match tone {
        "warning" => Some(palette.warning),
        "error" | "danger" => Some(palette.error),
        "success" => Some(palette.success),
        "info" => Some(palette.info),
        "accent" | "primary" => Some(palette.accent),
        _ => None,
    }
}
