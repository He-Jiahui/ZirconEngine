use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const CONSOLE_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/console.v2.ui.toml";
const CONSOLE_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.v2.ui.toml";
const CONSOLE_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.v2.ui.toml";

pub(crate) fn console_pane_nodes(status_text: &str, size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    let resolved_status = if status_text.is_empty() {
        "Console ready".to_string()
    } else {
        status_text.to_string()
    };
    text_overrides.insert("ConsoleTextPanel".to_string(), resolved_status.clone());
    text_overrides.insert("ConsoleHeader".to_string(), "Console".to_string());
    text_overrides.insert("FocusConsole".to_string(), "Focus Console".to_string());

    let mut nodes = build_view_template_nodes(
        "console.template_projection",
        CONSOLE_LAYOUT_ASSET_PATH,
        &[(CONSOLE_STYLE_ASSET_ID, CONSOLE_STYLE_ASSET_PATH)],
        size,
        &text_overrides,
    )
    .unwrap_or_default();
    apply_console_visual_state(&mut nodes, !status_text.is_empty());
    model_rc(nodes)
}

fn apply_console_visual_state(nodes: &mut [ViewTemplateNodeData], has_status: bool) {
    mark_console_node(nodes, "ConsoleHeader", has_status, "panel", "default");
    mark_console_node(nodes, "FocusConsole", has_status, "panel", "default");
    mark_console_node(
        nodes,
        "ConsoleBodySection",
        has_status,
        if has_status { "panel" } else { "inset" },
        if has_status { "default" } else { "muted" },
    );
    mark_console_node(
        nodes,
        "ConsoleTextPanel",
        has_status,
        if has_status { "panel" } else { "inset" },
        if has_status { "default" } else { "muted" },
    );
}

fn mark_console_node(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
    surface_variant: &str,
    text_tone: &str,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = active;
        node.surface_variant = surface_variant.into();
        node.text_tone = text_tone.into();
    }
}
