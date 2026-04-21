use super::support::collect_rust_files;

#[test]
fn core_editor_event_stops_owning_ui_binding_and_projection_surfaces() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let core_event_root = crate_root.join("core").join("editor_event");
    let core_runtime_root = core_event_root.join("runtime");
    let ui_host_root = crate_root.join("ui").join("host");
    let ui_event_execution_root = ui_host_root.join("editor_event_execution");
    let ui_binding_core_root = crate_root.join("ui").join("binding").join("core");
    let ui_selection_dispatch_root = crate_root
        .join("ui")
        .join("binding_dispatch")
        .join("selection");
    let ui_reflection_root = crate_root.join("ui").join("workbench").join("reflection");
    let callback_dispatch_root = crate_root
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let core_mod_source =
        std::fs::read_to_string(core_event_root.join("mod.rs")).expect("core editor_event mod");
    let core_runtime_mod_source =
        std::fs::read_to_string(core_event_root.join("runtime.rs")).expect("core runtime mod");
    let core_runtime_type_source =
        std::fs::read_to_string(core_runtime_root.join("editor_event_runtime.rs"))
            .expect("core runtime type");
    let core_runtime_inner_source =
        std::fs::read_to_string(core_runtime_root.join("editor_event_runtime_inner.rs"))
            .expect("core runtime inner type");
    let core_event_types_source =
        std::fs::read_to_string(core_event_root.join("types.rs")).expect("core event types");
    let ui_host_mod_source =
        std::fs::read_to_string(ui_host_root.join("mod.rs")).expect("ui host mod");
    let ui_binding_core_mod_source =
        std::fs::read_to_string(ui_binding_core_root.join("mod.rs")).expect("ui binding core mod");
    let ui_selection_dispatch_mod_source =
        std::fs::read_to_string(ui_selection_dispatch_root.join("mod.rs"))
            .expect("ui selection dispatch mod");
    let ui_event_execution_mod_source =
        std::fs::read_to_string(ui_event_execution_root.join("mod.rs"))
            .expect("ui event execution mod");
    let reflection_mod_source =
        std::fs::read_to_string(ui_reflection_root.join("mod.rs")).expect("reflection mod");
    let menu_action_source = std::fs::read_to_string(
        callback_dispatch_root
            .join("workbench")
            .join("menu_action.rs"),
    )
    .expect("workbench menu action");

    assert!(
        !core_event_root.join("host_adapter.rs").exists(),
        "expected core/editor_event/host_adapter.rs to be deleted after slint adapter cutover"
    );
    assert!(
        !core_event_root.join("transient.rs").exists(),
        "expected core/editor_event/transient.rs to be deleted after transient ui cutover"
    );
    for deleted in [
        "control_requests.rs",
        "dispatch.rs",
        "execution_outcome.rs",
        "accessors.rs",
        "binding_normalization.rs",
        "reflection.rs",
        "runtime_inner.rs",
    ] {
        assert!(
            !core_runtime_root.join(deleted).exists(),
            "expected core/editor_event/runtime/{deleted} to be deleted after ui host cutover"
        );
    }
    assert!(
        !core_mod_source.contains("pub mod host_adapter;"),
        "expected core::editor_event mod wiring to stop exposing host_adapter"
    );
    assert!(
        !core_mod_source.contains("pub use transient::EditorTransientUiState;"),
        "expected core::editor_event mod wiring to stop owning EditorTransientUiState"
    );
    for required in [
        "mod inspector_field_change;",
        "mod selection_host_event;",
        "pub use inspector_field_change::InspectorFieldChange;",
        "pub use selection_host_event::SelectionHostEvent;",
    ] {
        assert!(
            core_mod_source.contains(required),
            "expected core::editor_event mod wiring to include `{required}`"
        );
    }
    for forbidden in [
        "mod control_requests;",
        "mod dispatch;",
        "mod execution;",
        "mod execution_outcome;",
        "mod accessors;",
        "mod binding_normalization;",
        "mod reflection;",
        "mod runtime_inner;",
    ] {
        assert!(
            !core_runtime_mod_source.contains(forbidden),
            "expected core::editor_event::runtime wiring to stop owning `{forbidden}`"
        );
    }
    assert!(
        core_runtime_mod_source.contains("mod editor_event_runtime;"),
        "expected core::editor_event::runtime wiring to keep a narrow editor_event_runtime declaration module"
    );
    assert!(
        core_runtime_mod_source.contains("mod editor_event_runtime_inner;"),
        "expected core::editor_event::runtime wiring to own editor_event_runtime_inner directly"
    );
    for forbidden in [
        "use crate::ui::binding::InspectorFieldChange;",
        "use crate::ui::binding_dispatch::SelectionHostEvent;",
        "use crate::ui::workbench::layout::LayoutCommand;",
        "use crate::ui::workbench::model::MenuAction;",
    ] {
        assert!(
            !core_event_types_source.contains(forbidden),
            "expected core/editor_event/types.rs to stop importing `{forbidden}`"
        );
    }
    for forbidden in [
        "EditorManager",
        "EditorState",
        "fn new(",
        "crate::ui::host::editor_event_runtime_inner::EditorEventRuntimeInner",
    ] {
        assert!(
            !core_runtime_type_source.contains(forbidden),
            "expected core/editor_event/runtime/editor_event_runtime.rs to stop owning `{forbidden}`"
        );
    }
    for required in [
        "pub(crate) state: EditorState",
        "pub(crate) manager: Arc<EditorManager>",
        "pub(crate) transient: EditorTransientUiState",
        "pub(crate) journal: EditorEventJournal",
        "pub(crate) control_service: EditorUiControlService",
    ] {
        assert!(
            core_runtime_inner_source.contains(required),
            "expected core/editor_event/runtime/editor_event_runtime_inner.rs to own `{required}`"
        );
    }
    for required in [
        "mod editor_event_control_requests;",
        "mod editor_event_dispatch;",
        "mod editor_event_execution;",
        "mod editor_event_runtime_bootstrap;",
        "mod editor_event_runtime_access;",
        "mod editor_event_runtime_reflection;",
    ] {
        assert!(
            ui_host_mod_source.contains(required),
            "expected ui::host mod wiring to include `{required}`"
        );
    }
    for required in [
        "editor_event_control_requests.rs",
        "editor_event_dispatch.rs",
        "editor_event_runtime_bootstrap.rs",
        "editor_event_runtime_access.rs",
        "editor_event_runtime_reflection.rs",
    ] {
        assert!(
            ui_host_root.join(required).exists(),
            "expected ui::host to own `{required}` directly"
        );
    }
    assert!(
        !ui_host_root.join("editor_event_runtime_inner.rs").exists(),
        "expected ui::host/editor_event_runtime_inner.rs to be deleted after core runtime inner cutover"
    );
    for required in [
        "mod dispatch;",
        "mod execution_outcome;",
        "mod menu_action;",
        "mod layout_command;",
        "mod selection_event;",
        "mod inspector_event;",
        "mod draft_event;",
        "mod viewport_event;",
        "mod asset_event;",
        "mod animation_event;",
        "mod common;",
        "mod undo_policy;",
    ] {
        assert!(
            ui_event_execution_mod_source.contains(required),
            "expected ui::host::editor_event_execution mod wiring to include `{required}`"
        );
    }
    for required in [
        "mod.rs",
        "dispatch.rs",
        "execution_outcome.rs",
        "menu_action.rs",
        "layout_command.rs",
        "selection_event.rs",
        "inspector_event.rs",
        "draft_event.rs",
        "viewport_event.rs",
        "asset_event.rs",
        "animation_event.rs",
        "common.rs",
        "undo_policy.rs",
    ] {
        assert!(
            ui_event_execution_root.join(required).exists(),
            "expected ui::host::editor_event_execution to own `{required}` directly"
        );
    }
    for deleted in [
        "mod.rs",
        "dispatch.rs",
        "animation_event.rs",
        "asset_event.rs",
        "common.rs",
        "draft_event.rs",
        "inspector_event.rs",
        "layout_command.rs",
        "menu_action.rs",
        "selection_event.rs",
        "undo_policy.rs",
        "viewport_event.rs",
    ] {
        assert!(
            !core_runtime_root.join("execution").join(deleted).exists(),
            "expected core/editor_event/runtime/execution/{deleted} to be deleted after ui host cutover"
        );
    }
    assert!(
        ui_reflection_root.join("transient_ui_state.rs").exists(),
        "expected transient UI projection owner file under {:?}",
        ui_reflection_root
    );
    assert!(
        !ui_binding_core_root
            .join("inspector_field_change.rs")
            .exists(),
        "expected ui/binding/core/inspector_field_change.rs to be deleted after cutover"
    );
    assert!(
        !ui_selection_dispatch_root.join("selection_host_event.rs").exists(),
        "expected ui/binding_dispatch/selection/selection_host_event.rs to be deleted after cutover"
    );
    for forbidden in [
        "mod inspector_field_change;",
        "pub use inspector_field_change::InspectorFieldChange;",
    ] {
        assert!(
            !ui_binding_core_mod_source.contains(forbidden),
            "expected ui::binding::core mod wiring to stop owning `{forbidden}`"
        );
    }
    for forbidden in [
        "mod selection_host_event;",
        "pub use selection_host_event::SelectionHostEvent;",
    ] {
        assert!(
            !ui_selection_dispatch_mod_source.contains(forbidden),
            "expected ui::binding_dispatch::selection mod wiring to stop owning `{forbidden}`"
        );
    }
    assert!(
        reflection_mod_source.contains("mod transient_ui_state;"),
        "expected ui::workbench::reflection mod wiring to include transient_ui_state"
    );
    assert!(
        reflection_mod_source.contains(
            "pub(crate) use transient_ui_state::{apply_transient_projection, EditorTransientUiState};"
        ),
        "expected ui::workbench::reflection to own transient projection helpers directly"
    );
    assert!(
        menu_action_source.contains("fn slint_menu_action("),
        "expected workbench menu action module to own Slint menu normalization directly"
    );

    for file in collect_rust_files(&callback_dispatch_root) {
        let source = std::fs::read_to_string(&file).expect("callback dispatch source");
        assert!(
            !source.contains("host_adapter"),
            "expected callback dispatch module {:?} to stop routing through core host_adapter",
            file
        );
    }
}
