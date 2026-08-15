use super::super::support::*;
use crate::core::extension::FieldEditorInstance;
use crate::ui::retained_host::callback_dispatch::load_startup_builtin_template_runtime;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::retained_host::{
    to_host_contract_workbench_window_nodes, TemplatePaneMenuItemData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reference::EditorWorkbenchTemplateControlIds;
use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot, InspectorSnapshot,
    SceneEntries, SceneEntry,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::layout::UiMargin;
use zircon_runtime_interface::ui::style::UiStyleColor;
use zircon_runtime_interface::ui::tree::UiVisibility;
use zircon_runtime_interface::ui::v2::{
    UI_V2_REPEAT_ATTRIBUTE, UI_V2_REPEAT_FIELD_AUTHORED_COUNT, UI_V2_REPEAT_FIELD_KIND,
    UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE, UI_V2_REPEAT_FIELD_PROTOTYPE,
    UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX, UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
};

mod document_module;
mod interaction;
mod popup_projection;
mod scene_fragment;
mod scene_snapshot;
mod shell_layout;
mod support;
mod surface_contract;

use support::*;

#[test]
fn workbench_projection_test_owners_stay_within_budget() {
    for (path, source) in [
        ("mod.rs", include_str!("mod.rs")),
        ("shell_layout.rs", include_str!("shell_layout.rs")),
        ("document_module.rs", include_str!("document_module.rs")),
        ("scene_fragment.rs", include_str!("scene_fragment.rs")),
        ("scene_snapshot.rs", include_str!("scene_snapshot.rs")),
        ("interaction.rs", include_str!("interaction.rs")),
        ("popup_projection.rs", include_str!("popup_projection.rs")),
        ("support.rs", include_str!("support.rs")),
        (
            "surface_contract/mod.rs",
            include_str!("surface_contract/mod.rs"),
        ),
        (
            "surface_contract/viewport.rs",
            include_str!("surface_contract/viewport.rs"),
        ),
        (
            "surface_contract/buttons.rs",
            include_str!("surface_contract/buttons.rs"),
        ),
        (
            "surface_contract/input_feedback.rs",
            include_str!("surface_contract/input_feedback.rs"),
        ),
        (
            "surface_contract/chrome_routes.rs",
            include_str!("surface_contract/chrome_routes.rs"),
        ),
    ] {
        assert!(
            source.lines().count() <= 800,
            "workbench projection test owner `{path}` exceeds the 800-line budget"
        );
    }
}
