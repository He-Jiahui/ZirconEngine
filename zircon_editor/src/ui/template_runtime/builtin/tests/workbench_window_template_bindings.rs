use super::*;

#[test]
fn component_lab_search_input_has_edit_and_commit_bindings() {
    let bindings = workbench_window_template_bindings();

    assert_menu_binding(
        &bindings,
        "ComponentLab/InputSearchEdit",
        EditorUiEventKind::Change,
        "component_lab.input_search.edit",
    );
    assert_menu_binding(
        &bindings,
        "ComponentLab/InputSearchCommit",
        EditorUiEventKind::Submit,
        "component_lab.input_search.commit",
    );
}

#[test]
fn scene_search_and_component_lab_numeric_fields_have_edit_and_commit_bindings() {
    let bindings = workbench_window_template_bindings();

    for (binding_id, event_kind, action) in [
        (
            "Workbench/SceneSearchEdit",
            EditorUiEventKind::Change,
            "workbench.hierarchy.search.edit",
        ),
        (
            "Workbench/SceneSearchCommit",
            EditorUiEventKind::Submit,
            "workbench.hierarchy.search.commit",
        ),
        (
            "Workbench/SceneSearchEditFromShell",
            EditorUiEventKind::Change,
            "workbench.hierarchy.search.edit",
        ),
        (
            "Workbench/SceneSearchCommitFromShell",
            EditorUiEventKind::Submit,
            "workbench.hierarchy.search.commit",
        ),
    ] {
        assert_menu_binding(&bindings, binding_id, event_kind, action);
    }
    for (control, action) in [
        ("InputStepper", "input_stepper"),
        ("InputSlider", "input_slider"),
        ("InputRangeSlider", "input_range_slider"),
        ("InputStepsSlider", "input_steps_slider"),
    ] {
        assert_menu_binding(
            &bindings,
            &format!("ComponentLab/{control}Edit"),
            EditorUiEventKind::Change,
            &format!("component_lab.{action}.edit"),
        );
        assert_menu_binding(
            &bindings,
            &format!("ComponentLab/{control}Commit"),
            EditorUiEventKind::Submit,
            &format!("component_lab.{action}.commit"),
        );
    }
}

#[test]
fn inspector_search_field_has_edit_and_commit_bindings() {
    let bindings = workbench_window_template_bindings();

    assert_menu_binding(
        &bindings,
        "Workbench/InspectorSearchEdit",
        EditorUiEventKind::Change,
        "workbench.inspector.search.edit",
    );
    assert_menu_binding(
        &bindings,
        "Workbench/InspectorSearchCommit",
        EditorUiEventKind::Submit,
        "workbench.inspector.search.commit",
    );
}

#[test]
fn component_lab_icon_button_samples_have_click_bindings() {
    let bindings = workbench_window_template_bindings();

    for (binding_id, action) in [
        ("ComponentLab/MiniAdd", "component_lab.icon_button.add"),
        ("ComponentLab/MiniOpen", "component_lab.icon_button.open"),
        ("ComponentLab/MiniSave", "component_lab.icon_button.save"),
        (
            "ComponentLab/MiniDelete",
            "component_lab.icon_button.delete",
        ),
        ("ComponentLab/MiniShow", "component_lab.icon_button.show"),
        ("ComponentLab/MiniHide", "component_lab.icon_button.hide"),
        ("ComponentLab/MiniLock", "component_lab.icon_button.lock"),
        ("ComponentLab/MiniMore", "component_lab.icon_button.more"),
    ] {
        assert_menu_binding(&bindings, binding_id, EditorUiEventKind::Click, action);
    }
}

#[test]
fn workbench_command_palette_commit_binding_is_registered() {
    let bindings = workbench_window_template_bindings();
    let binding = bindings
        .get("CommandPalette/Commit")
        .expect("command palette commit binding should be registered");

    assert_eq!(binding.path().event_kind, EditorUiEventKind::Submit);
    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::editor_command("editor.command.palette")
    );

    let query_binding = bindings
        .get("CommandPalette/QueryChanged")
        .expect("command palette query binding should be registered");
    assert_eq!(query_binding.path().event_kind, EditorUiEventKind::Change);
    assert_eq!(
        query_binding.payload(),
        &EditorUiBindingPayload::menu_action("editor.command_palette.query_changed")
    );

    let window_binding = bindings
        .get("CommandPalette/WindowRequested")
        .expect("command palette window request binding should be registered");
    assert_eq!(window_binding.path().event_kind, EditorUiEventKind::Change);
    assert_eq!(
        window_binding.payload(),
        &EditorUiBindingPayload::menu_action("editor.command_palette.window_requested")
    );
}

#[test]
fn top_toolbar_registered_commands_dispatch_through_the_command_registry() {
    let bindings = workbench_window_template_bindings();

    for (binding_id, command_id) in [
        ("MenuAction/OpenProject", "file.project.open"),
        ("MenuAction/SaveProject", "file.project.save"),
        ("MenuAction/ResetLayout", "window.layout.reset"),
        ("Run/Play", "runtime.play_mode.enter"),
        ("Run/Stop", "runtime.play_mode.exit"),
    ] {
        let binding = bindings
            .get(binding_id)
            .unwrap_or_else(|| panic!("{binding_id} should be registered"));
        assert_eq!(binding.path().event_kind, EditorUiEventKind::Click);
        assert_eq!(
            binding.payload(),
            &EditorUiBindingPayload::editor_command(command_id)
        );
    }
}

#[test]
fn status_shortcuts_have_unique_bindings_with_canonical_viewport_payloads() {
    let bindings = workbench_window_template_bindings();

    for (binding_id, expected_payload) in [
        (
            "Workbench/ToggleSnapFromStatus",
            EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
                GridMode::VisibleAndSnap,
            )),
        ),
        (
            "Workbench/FrameSelectionFromStatus",
            EditorUiBindingPayload::viewport_command(ViewportCommand::FrameSelection),
        ),
    ] {
        let binding = bindings
            .get(binding_id)
            .unwrap_or_else(|| panic!("{binding_id} should be registered"));
        assert_eq!(binding.path().event_kind, EditorUiEventKind::Click);
        assert_eq!(binding.payload(), &expected_payload);
    }
}

#[test]
fn top_toolbar_asset_browser_uses_the_canonical_asset_command() {
    let bindings = workbench_window_template_bindings();
    let binding = bindings
        .get("Workbench/OpenAssetBrowserFromToolbar")
        .expect("top-toolbar asset browser binding should be registered");

    assert_eq!(binding.path().event_kind, EditorUiEventKind::Click);
    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser)
    );
}

fn assert_menu_binding(
    bindings: &BTreeMap<String, EditorUiBinding>,
    binding_id: &str,
    event_kind: EditorUiEventKind,
    action_id: &str,
) {
    let binding = bindings
        .get(binding_id)
        .unwrap_or_else(|| panic!("{binding_id} should be registered"));

    assert_eq!(binding.path().event_kind, event_kind);
    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::menu_action(action_id)
    );
}
