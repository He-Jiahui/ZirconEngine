use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const CONSOLE_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/console.zui";

pub(crate) fn console_pane_nodes(status_text: &str, size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        "ConsoleTextPanel".to_string(),
        if status_text.is_empty() {
            "Console ready".to_string()
        } else {
            status_text.to_string()
        },
    );
    let node_patches = console_visual_state_patches(!status_text.is_empty());
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "console.template_projection",
        CONSOLE_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}

fn console_visual_state_patches(has_status: bool) -> BTreeMap<String, ViewTemplateNodePatch> {
    BTreeMap::from([
        (
            "ConsoleHeader".to_string(),
            ViewTemplateNodePatch::visual_state(false, false, "transparent", "default"),
        ),
        (
            "ConsoleBodySection".to_string(),
            ViewTemplateNodePatch::visual_state(false, false, "transparent", "muted"),
        ),
        (
            "ConsoleTextPanel".to_string(),
            ViewTemplateNodePatch::visual_state(
                false,
                false,
                "transparent",
                if has_status { "default" } else { "muted" },
            ),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projected_nodes(status_text: &str) -> Vec<ViewTemplateNodeData> {
        let pane = console_pane_nodes(status_text, UiSize::new(360.0, 240.0));
        (0..pane.row_count())
            .filter_map(|row| pane.row_data(row))
            .collect()
    }

    fn node_by_control_id<'a>(
        nodes: &'a [ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<&'a ViewTemplateNodeData> {
        nodes.iter().find(|node| node.control_id == control_id)
    }

    #[test]
    fn empty_console_projects_a_muted_output_surface_without_focus() {
        let nodes = projected_nodes("");

        assert!(nodes.iter().any(|node| node.control_id == "ConsoleHeader"));
        assert!(nodes
            .iter()
            .any(|node| node.control_id == "ConsoleBodySection"));

        let Some(header) = node_by_control_id(&nodes, "ConsoleHeader") else {
            return;
        };
        let Some(body) = node_by_control_id(&nodes, "ConsoleBodySection") else {
            return;
        };
        let Some(text) = node_by_control_id(&nodes, "ConsoleTextPanel") else {
            return;
        };

        assert_eq!(header.text.to_string(), "Console");
        assert_eq!(body.surface_variant.to_string(), "transparent");
        assert!(!body.selected);
        assert!(!body.focused);
        assert_eq!(text.text.to_string(), "Console ready");
        assert_eq!(text.text_tone.to_string(), "muted");
        assert!(!text.focused);
    }

    #[test]
    fn console_status_uses_the_active_output_surface_without_focus() {
        let nodes = projected_nodes("Build completed");

        let Some(body) = node_by_control_id(&nodes, "ConsoleBodySection") else {
            return;
        };
        let Some(text) = node_by_control_id(&nodes, "ConsoleTextPanel") else {
            return;
        };

        assert_eq!(body.surface_variant.to_string(), "transparent");
        assert!(!body.selected);
        assert!(!body.focused);
        assert_eq!(text.text.to_string(), "Build completed");
        assert_eq!(text.text_tone.to_string(), "default");
        assert!(!text.focused);
    }

    #[test]
    fn stable_console_projection_reuses_the_same_retained_rows() {
        let first = console_pane_nodes("Build completed", UiSize::new(360.0, 240.0));
        let stable = console_pane_nodes("Build completed", UiSize::new(360.0, 240.0));

        assert!(first.shares_values_with(&stable));
    }
}
