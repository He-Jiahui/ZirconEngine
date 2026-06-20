use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::material_primitives::component_variant_contains;
use super::super::template_style_color::resolved_style_color;

const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_material_progress_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    ) || matches!(
        node.role.as_str(),
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner"
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_circular(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "circular-progress" | "spinner"
    ) || matches!(node.role.as_str(), "CircularProgress" | "Spinner")
        || component_variant_contains(node, "circular")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_indeterminate(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.component_role.as_str(), "spinner")
        || component_variant_contains(node, "indeterminate")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_percent(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MATERIAL_PROGRESS_TRACK)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node))
        .unwrap_or(PALETTE.accent)
}

fn material_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
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
