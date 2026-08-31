pub(super) use std::collections::{BTreeMap, BTreeSet};
pub(super) use std::sync::Arc;

pub(super) use super::super::super::support::*;
pub(super) use super::super::support::{
    control_bool, control_float, control_string, control_visibility,
};
pub(super) use crate::core::asset::{
    AssetCreationTemplateDescriptor, AssetTypeContribution, AssetTypeId, AssetTypeRegistry,
};
pub(super) use crate::core::commands::{
    CommandEvalCtx, EditorCommandDescriptor, EditorCommandRegistry, EditorKeyChord, EditorKeymap,
};
pub(super) use crate::core::editor_extension::EditorExtensionRegistry;
pub(super) use crate::core::editor_operation::EditorOperationPath;
pub(super) use crate::core::extension::{CapabilitySet, ContributionSnapshot};
pub(super) use crate::core::play::{PlayKind, PlayMode};
pub(super) use crate::core::settings::EditorKeymapOverrides;
pub(super) use crate::ui::binding::AssetCommand;
pub(super) use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;
pub(super) use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
pub(super) use crate::ui::workbench::fixture::default_preview_fixture;
pub(super) use crate::ui::workbench::model::WorkbenchViewModel;
pub(super) use zircon_runtime_interface::resource::ResourceKind;
pub(super) use zircon_runtime_interface::ui::tree::UiVisibility;

#[derive(Clone, Copy)]
pub(super) enum ToolbarMenuAlign {
    Start,
    End,
}

pub(super) fn assert_toolbar_menu_anchor(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    trigger_id: &str,
    menu_id: &str,
    align: ToolbarMenuAlign,
) {
    let trigger = bridge
        .control_frame(trigger_id)
        .unwrap_or_else(|| panic!("{trigger_id} should expose a visible trigger frame"));
    let toolbar = bridge
        .control_frame("WorkbenchWindowTopToolbarRegion")
        .expect("workbench should expose the top toolbar frame");
    let root = bridge
        .control_frame("WorkbenchWindowRoot")
        .expect("workbench should expose the root frame");

    bridge
        .dispatch_control_state(trigger_id, UiEventKind::Click)
        .unwrap_or_else(|error| panic!("{trigger_id} should dispatch: {error:?}"))
        .unwrap_or_else(|| panic!("{trigger_id} should expose a click binding"));

    let arranged_menu = bridge
        .control_frame(menu_id)
        .unwrap_or_else(|| panic!("{menu_id} should open with a visible menu frame"));
    let menu = rendered_control_frame(bridge, menu_id);
    let authored_x = match align {
        ToolbarMenuAlign::Start => trigger.x,
        ToolbarMenuAlign::End => trigger.right() - arranged_menu.width,
    };
    let expected_x = authored_x.clamp(root.x, root.right() - arranged_menu.width);
    assert_near(&format!("{menu_id} x"), menu.x, expected_x);
    assert_near(&format!("{menu_id} y"), menu.y, toolbar.bottom());
    assert_near(&format!("{menu_id} width"), menu.width, arranged_menu.width);
    let node = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(menu_id)
        })
        .unwrap_or_else(|| panic!("{menu_id} should resolve to one runtime node"));
    let metadata = node
        .template_metadata
        .as_ref()
        .expect("menu node should retain template metadata");
    assert_eq!(
        metadata.widget.popup_anchor.control_id(),
        Some(trigger_id),
        "{menu_id} should resolve its live trigger through the runtime widget contract"
    );
    assert_eq!(
        control_string(bridge, menu_id, "placement").as_deref(),
        Some(match align {
            ToolbarMenuAlign::Start => "bottom-start",
            ToolbarMenuAlign::End => "bottom-end",
        })
    );
    assert_near(
        &format!("{menu_id} popup_offset_y"),
        control_float(bridge, menu_id, "popup_offset_y")
            .unwrap_or_else(|| panic!("{menu_id} should declare popup_offset_y")) as f32,
        toolbar.bottom() - trigger.bottom() - 4.0,
    );
    assert_eq!(control_float(bridge, menu_id, "popup_anchor_x"), None);
    assert_eq!(control_float(bridge, menu_id, "popup_anchor_y"), None);
}

pub(super) fn rendered_control_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiFrame {
    let node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|candidate| *candidate == control_id)
                .map(|_| node.node_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should resolve to one runtime node"));
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == node_id)
        .map(|command| command.frame)
        .max_by(|left, right| frame_area(*left).total_cmp(&frame_area(*right)))
        .unwrap_or_else(|| panic!("{control_id} should emit popup render commands"))
}

fn frame_area(frame: UiFrame) -> f32 {
    frame.width.max(0.0) * frame.height.max(0.0)
}

pub(super) fn assert_near(label: &str, actual: f32, expected: f32) {
    const EPSILON: f32 = 0.01;
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

pub(super) fn control_string_array(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Vec<String> {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(toml::Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
        })
        .unwrap_or_default()
}
