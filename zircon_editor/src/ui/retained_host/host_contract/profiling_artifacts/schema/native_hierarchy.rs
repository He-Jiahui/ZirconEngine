use serde::Serialize;

use super::frame::UiProfileFrame;

/// Source-bound native hierarchy state emitted only in the profile artifact.
/// Rows are copied from the retained `SceneNodeData` model; no pixels or
/// inferred rows are used.
#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNativeHierarchy {
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) viewport_frame: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) scroll_offset: f32,
    pub(in crate::ui::retained_host::host_contract) rows: Vec<UiProfileNativeHierarchyRow>,
    pub(in crate::ui::retained_host::host_contract) selected_node_ids: Vec<String>,
    pub(in crate::ui::retained_host::host_contract) selected_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) focused_control_id: Option<String>,
    pub(in crate::ui::retained_host::host_contract) popup_focus: UiProfileNativePopupFocus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) inline_rename:
        Option<UiProfileNativeHierarchyRename>,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNativePopupFocus {
    pub(in crate::ui::retained_host::host_contract) menu_open: bool,
    pub(in crate::ui::retained_host::host_contract) page_overflow_open: bool,
    pub(in crate::ui::retained_host::host_contract) dock_overflow_open: bool,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNativeHierarchyRow {
    pub(in crate::ui::retained_host::host_contract) index: usize,
    pub(in crate::ui::retained_host::host_contract) node_id: String,
    pub(in crate::ui::retained_host::host_contract) name: String,
    pub(in crate::ui::retained_host::host_contract) depth: i32,
    pub(in crate::ui::retained_host::host_contract) selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) frame: Option<UiProfileFrame>,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNativeHierarchyRename {
    pub(in crate::ui::retained_host::host_contract) control_id: String,
    pub(in crate::ui::retained_host::host_contract) node_id: String,
    pub(in crate::ui::retained_host::host_contract) value_text: String,
    pub(in crate::ui::retained_host::host_contract) edit_frame: UiProfileFrame,
}
