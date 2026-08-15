pub(super) use std::collections::BTreeSet;
pub(super) use std::sync::Arc;

pub(super) use super::super::super::support::*;
pub(super) use super::super::support::{
    control_bool, control_float, control_string, control_visibility,
};
pub(super) use crate::core::asset::{
    AssetCreationTemplateDescriptor, AssetTypeContribution, AssetTypeId, AssetTypeRegistry,
};
pub(super) use crate::core::commands::EditorCommandDescriptor;
pub(super) use crate::core::editor_extension::EditorExtensionRegistry;
pub(super) use crate::core::editor_operation::EditorOperationPath;
pub(super) use crate::core::play::{PlayKind, PlayMode};
pub(super) use crate::ui::binding::AssetCommand;
pub(super) use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;
pub(super) use crate::ui::retained_host::popup_anchor_metrics::{
    clamp_popup_x_to_bounds, toolbar_popup_render_gap,
};
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

    let menu = bridge
        .control_frame(menu_id)
        .unwrap_or_else(|| panic!("{menu_id} should open with a visible menu frame"));
    let authored_x = match align {
        ToolbarMenuAlign::Start => trigger.x,
        ToolbarMenuAlign::End => trigger.right() - menu.width,
    };
    let expected_x = clamped_toolbar_menu_x(authored_x, menu.width, root.width);
    assert_near(&format!("{menu_id} x"), menu.x, expected_x);
    assert_near(&format!("{menu_id} y"), menu.y, toolbar.bottom());
    assert_near(
        &format!("{menu_id} popup_anchor_x"),
        control_float(bridge, menu_id, "popup_anchor_x")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_x")) as f32,
        menu.x,
    );
    assert_near(
        &format!("{menu_id} popup_anchor_y"),
        control_float(bridge, menu_id, "popup_anchor_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_y")) as f32,
        menu.y,
    );
    assert_near(
        &format!("{menu_id} popup_offset_y"),
        control_float(bridge, menu_id, "popup_offset_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_offset_y")) as f32,
        -toolbar_popup_render_gap(),
    );
    assert_eq!(
        control_string(bridge, menu_id, "placement").as_deref(),
        Some("bottom-start")
    );
}

pub(super) fn clamped_toolbar_menu_x(authored_x: f32, menu_width: f32, root_width: f32) -> f32 {
    clamp_popup_x_to_bounds(authored_x, 0.0, root_width, menu_width)
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
