use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const INSPECTOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/inspector.zui";
const INSPECTOR_EMPTY_STATE_CONTROL_ID: &str = "InspectorEmptyState";
const INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID: &str = "InspectorEmptyStateMessage";
const INSPECTOR_NAME_VALUE_CONTROL_ID: &str = "InspectorNameValue";
const INSPECTOR_PARENT_VALUE_CONTROL_ID: &str = "InspectorParentValue";
const INSPECTOR_POSITION_VALUE_CONTROL_ID: &str = "InspectorPositionValue";
const INSPECTOR_COMPONENTS_VALUE_CONTROL_ID: &str = "InspectorComponentsValue";

pub(crate) fn inspector_pane_nodes(
    inspector: Option<&InspectorSnapshot>,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        INSPECTOR_NAME_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.name.clone())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_PARENT_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.parent.clone())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_POSITION_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| {
                format!(
                    "{}, {}, {}",
                    inspector.translation[0], inspector.translation[1], inspector.translation[2]
                )
            })
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_COMPONENTS_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.plugin_components.len().to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID.to_string(),
        inspector
            .map(|_| String::new())
            .unwrap_or_else(|| "No object selected".to_string()),
    );

    let node_patches = inspector_visual_state_patches(inspector.is_some());
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "inspector.template_projection",
        INSPECTOR_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}

fn inspector_visual_state_patches(has_selection: bool) -> BTreeMap<String, ViewTemplateNodePatch> {
    let mut patches = BTreeMap::new();
    mark_readout(&mut patches, INSPECTOR_NAME_VALUE_CONTROL_ID, has_selection);
    mark_readout(
        &mut patches,
        INSPECTOR_PARENT_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_readout(
        &mut patches,
        INSPECTOR_POSITION_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_readout(
        &mut patches,
        INSPECTOR_COMPONENTS_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_empty_state(&mut patches, has_selection);
    patches
}

fn mark_empty_state(patches: &mut BTreeMap<String, ViewTemplateNodePatch>, has_selection: bool) {
    patches.insert(
        INSPECTOR_EMPTY_STATE_CONTROL_ID.to_string(),
        ViewTemplateNodePatch {
            selected: Some(false),
            focused: Some(false),
            surface_variant: Some(
                if has_selection {
                    "transparent"
                } else {
                    "inset"
                }
                .to_string(),
            ),
            ..ViewTemplateNodePatch::default()
        },
    );
}

fn mark_readout(
    patches: &mut BTreeMap<String, ViewTemplateNodePatch>,
    control_id: &str,
    active: bool,
) {
    patches.insert(
        control_id.to_string(),
        ViewTemplateNodePatch {
            selected: Some(false),
            text_tone: Some(if active { "default" } else { "muted" }.to_string()),
            ..ViewTemplateNodePatch::default()
        },
    );
}

#[cfg(test)]
#[path = "tests/inspector.rs"]
mod tests;
