use std::collections::BTreeMap;

use super::values::value_as_i32;

const MUI_Z_INDEX_MOBILE_STEPPER: i32 = 1000;
const MUI_Z_INDEX_FAB: i32 = 1050;
const MUI_Z_INDEX_APP_BAR: i32 = 1100;
const MUI_Z_INDEX_DRAWER: i32 = 1200;
const MUI_Z_INDEX_MODAL: i32 = 1300;
const MUI_Z_INDEX_SNACKBAR: i32 = 1400;
const MUI_Z_INDEX_TOOLTIP: i32 = 1500;

pub(in super::super) fn projected_z_index(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    node_z_index: i32,
) -> i32 {
    attributes
        .get("z_index")
        .or_else(|| attributes.get("mui_z_index"))
        .or_else(|| attributes.get("zIndex"))
        .and_then(value_as_i32)
        .or_else(|| (node_z_index != 0).then_some(node_z_index))
        .unwrap_or_else(|| default_mui_z_index(component_role))
}

fn default_mui_z_index(component_role: &str) -> i32 {
    match component_role {
        "mobile-stepper" => MUI_Z_INDEX_MOBILE_STEPPER,
        "fab" | "floating-action-button" | "speed-dial" => MUI_Z_INDEX_FAB,
        "app-bar" => MUI_Z_INDEX_APP_BAR,
        "drawer" => MUI_Z_INDEX_DRAWER,
        // MUI Backdrop is normally nested under Modal; in the retained host it is a sibling,
        // so keep it immediately under modal surfaces while preserving the same global layer.
        "backdrop" => MUI_Z_INDEX_MODAL - 1,
        "modal" | "dialog" | "confirm-dialog" | "alert-dialog" | "popover" | "popper" | "menu" => {
            MUI_Z_INDEX_MODAL
        }
        "snackbar" | "snackbar-content" => MUI_Z_INDEX_SNACKBAR,
        "tooltip" => MUI_Z_INDEX_TOOLTIP,
        _ => 0,
    }
}
