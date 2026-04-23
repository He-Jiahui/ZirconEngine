#[test]
fn editor_state_declaration_moves_under_ui_workbench_state() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_workbench_root = crate_root.join("ui").join("workbench");
    let ui_state_root = ui_workbench_root.join("state");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let core_editing_root = crate_root.join("core").join("editing");
    let workbench_mod =
        std::fs::read_to_string(ui_workbench_root.join("mod.rs")).expect("workbench mod");
    let state_mod = std::fs::read_to_string(ui_state_root.join("mod.rs")).expect("state mod");
    let state_source =
        std::fs::read_to_string(ui_state_root.join("editor_state.rs")).expect("editor state");
    let core_editing_mod =
        std::fs::read_to_string(core_editing_root.join("mod.rs")).expect("core editing mod");

    assert!(
        workbench_mod.contains("pub mod state;"),
        "expected ui::workbench mod wiring to expose the state subtree directly"
    );
    assert!(
        ui_state_root.join("editor_state.rs").exists(),
        "expected EditorState declaration owner file under {:?}",
        ui_state_root
    );
    assert!(
        state_mod.contains("mod editor_state;")
            && state_mod.contains("pub use editor_state::EditorState;"),
        "expected ui::workbench::state mod wiring to own EditorState directly"
    );
    assert!(
        state_mod.contains("pub(crate) mod editor_world_slot;"),
        "expected ui::workbench::state mod wiring to expose editor_world_slot directly"
    );
    assert!(
        ui_state_root.join("editor_world_slot.rs").exists(),
        "expected EditorWorldSlot owner file under {:?}",
        ui_state_root
    );
    assert!(
        !core_state_root.join("mod.rs").exists(),
        "expected core/editing/state/mod.rs to be deleted after ui workbench state cutover"
    );
    assert!(
        !core_state_root.join("editor_state.rs").exists(),
        "expected core/editing/state/editor_state.rs to be deleted after ui workbench state cutover"
    );
    assert!(
        !core_state_root.join("editor_world_slot.rs").exists(),
        "expected core/editing/state/editor_world_slot.rs to be deleted after ui workbench state cutover"
    );
    assert!(
        !core_editing_mod.contains("pub(crate) mod state;"),
        "expected core::editing mod wiring to stop exposing the state subtree"
    );
    assert!(
        state_source.contains("use crate::ui::workbench::project::AssetWorkspaceState;"),
        "expected EditorState to consume AssetWorkspaceState from ui::workbench::project"
    );
}

#[test]
fn editor_state_behavior_moves_under_ui_workbench_state() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_state_root = crate_root.join("ui").join("workbench").join("state");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let state_mod = std::fs::read_to_string(ui_state_root.join("mod.rs")).expect("state mod");

    for required in [
        "mod editor_state_apply_intent;",
        "mod editor_state_field_updates;",
        "mod editor_state_render;",
        "mod editor_state_selection;",
        "mod editor_state_viewport;",
    ] {
        assert!(
            state_mod.contains(required),
            "expected ui::workbench::state mod wiring to include `{required}`"
        );
    }

    for required in [
        "editor_state_apply_intent.rs",
        "editor_state_field_updates.rs",
        "editor_state_render.rs",
        "editor_state_selection.rs",
        "editor_state_viewport.rs",
    ] {
        assert!(
            ui_state_root.join(required).exists(),
            "expected ui::workbench::state to own `{required}` directly"
        );
    }

    for deleted in [
        "editor_state_apply_intent.rs",
        "editor_state_field_updates.rs",
        "editor_state_render.rs",
        "editor_state_selection.rs",
        "editor_state_viewport.rs",
        "no_project_open.rs",
        "parse_parent_field.rs",
    ] {
        assert!(
            !core_state_root.join(deleted).exists(),
            "expected core/editing/state/{deleted} to be deleted after ui workbench state cutover"
        );
    }
}

#[test]
fn asset_workspace_state_moves_under_ui_workbench_project() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_project_root = crate_root.join("ui").join("workbench").join("project");
    let core_editing_root = crate_root.join("core").join("editing");
    let project_mod =
        std::fs::read_to_string(ui_project_root.join("mod.rs")).expect("workbench project mod");

    assert!(
        ui_project_root.join("asset_workspace_state.rs").exists(),
        "expected asset workspace owner file under {:?}",
        ui_project_root
    );
    assert!(
        project_mod.contains("mod asset_workspace_state;")
            && project_mod.contains("pub(crate) use asset_workspace_state::AssetWorkspaceState;"),
        "expected ui::workbench::project mod wiring to own AssetWorkspaceState directly"
    );
    assert!(
        !core_editing_root.join("asset_workspace.rs").exists(),
        "expected core/editing asset workspace owner file to be deleted after ui workbench project cutover"
    );
}

#[test]
fn editor_state_asset_workspace_accessors_move_under_ui_workbench_project() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_project_root = crate_root.join("ui").join("workbench").join("project");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let project_mod =
        std::fs::read_to_string(ui_project_root.join("mod.rs")).expect("workbench project mod");

    assert!(
        ui_project_root
            .join("editor_state_asset_workspace.rs")
            .exists(),
        "expected EditorState asset workspace accessors under {:?}",
        ui_project_root
    );
    assert!(
        project_mod.contains("mod editor_state_asset_workspace;"),
        "expected ui::workbench::project mod wiring to own EditorState asset workspace accessors directly"
    );
    assert!(
        !core_state_root
            .join("editor_state_asset_workspace.rs")
            .exists(),
        "expected core/editing/state/editor_state_asset_workspace.rs to be deleted after ui workbench project cutover"
    );
}

#[test]
fn editor_state_session_transitions_move_under_ui_workbench_startup() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_startup_root = crate_root.join("ui").join("workbench").join("startup");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let startup_mod =
        std::fs::read_to_string(ui_startup_root.join("mod.rs")).expect("workbench startup mod");

    assert!(
        ui_startup_root.join("editor_state_project.rs").exists(),
        "expected EditorState session transition owner file under {:?}",
        ui_startup_root
    );
    assert!(
        startup_mod.contains("mod editor_state_project;"),
        "expected ui::workbench::startup mod wiring to own EditorState session transition methods directly"
    );
    assert!(
        !core_state_root.join("editor_state_project.rs").exists(),
        "expected core/editing/state/editor_state_project.rs to be deleted after ui workbench startup cutover"
    );
}

#[test]
fn editor_state_construction_moves_under_ui_workbench_startup() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_startup_root = crate_root.join("ui").join("workbench").join("startup");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let startup_mod =
        std::fs::read_to_string(ui_startup_root.join("mod.rs")).expect("workbench startup mod");

    assert!(
        ui_startup_root
            .join("editor_state_construction.rs")
            .exists(),
        "expected EditorState construction owner file under {:?}",
        ui_startup_root
    );
    assert!(
        startup_mod.contains("mod editor_state_construction;"),
        "expected ui::workbench::startup mod wiring to own EditorState construction methods directly"
    );
    assert!(
        !core_state_root
            .join("editor_state_construction.rs")
            .exists(),
        "expected core/editing/state/editor_state_construction.rs to be deleted after ui workbench startup cutover"
    );
}

#[test]
fn editor_state_snapshot_moves_under_ui_workbench_snapshot() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_snapshot_root = crate_root
        .join("ui")
        .join("workbench")
        .join("snapshot")
        .join("data");
    let core_state_root = crate_root.join("core").join("editing").join("state");
    let snapshot_mod =
        std::fs::read_to_string(ui_snapshot_root.join("mod.rs")).expect("snapshot data mod");

    assert!(
        ui_snapshot_root
            .join("editor_state_snapshot_build.rs")
            .exists(),
        "expected EditorState::snapshot owner file under {:?}",
        ui_snapshot_root
    );
    assert!(
        snapshot_mod.contains("mod editor_state_snapshot_build;"),
        "expected ui::workbench::snapshot::data mod wiring to own EditorState snapshot building directly"
    );
    assert!(
        !core_state_root.join("editor_state_snapshot.rs").exists(),
        "expected core/editing/state/editor_state_snapshot.rs to be deleted after ui workbench snapshot cutover"
    );
}
