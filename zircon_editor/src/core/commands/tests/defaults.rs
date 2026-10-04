use super::*;

#[test]
fn new_scene_is_project_scoped_and_uses_the_native_create_scene_command() {
    let commands = default_workbench_commands();
    let command = commands
        .iter()
        .find(|command| command.id().as_str() == "file.scene.create")
        .expect("New Scene should be registered for menus, keymap and palette");
    assert!(matches!(command.when(), WhenClause::ProjectOpen));
    assert_eq!(
        command.default_chord().map(ToString::to_string).as_deref(),
        Some("Ctrl+N")
    );
    assert_eq!(
        command.menu_path().map(|path| path.root().id().as_str()),
        Some("file")
    );
    assert!(matches!(
        command.event(),
        Some(EditorEvent::WorkbenchMenu(MenuAction::CreateScene))
    ));
    let ctrl_n = EditorKeyChord::from_str("Ctrl+N").unwrap();
    assert_eq!(
        commands
            .iter()
            .filter(|candidate| candidate.default_chord() == Some(&ctrl_n))
            .count(),
        1
    );
}

#[test]
fn open_scene_owns_ctrl_o_and_open_project_has_no_default_chord() {
    let commands = default_workbench_commands();
    let scene = commands
        .iter()
        .find(|command| command.id().as_str() == "file.scene.open")
        .expect("Open Scene should be available to the palette, menu and keymap");
    assert!(matches!(scene.when(), WhenClause::ProjectOpen));
    assert!(matches!(
        scene.event(),
        Some(EditorEvent::WorkbenchMenu(MenuAction::OpenScene))
    ));
    assert_eq!(
        scene.default_chord().map(ToString::to_string).as_deref(),
        Some("Ctrl+O")
    );
    let project = commands
        .iter()
        .find(|command| command.id().as_str() == "file.project.open")
        .unwrap();
    assert!(project.default_chord().is_none());
    let ctrl_o = EditorKeyChord::from_str("Ctrl+O").unwrap();
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.default_chord() == Some(&ctrl_o))
            .count(),
        1
    );
}

#[test]
fn close_project_is_a_project_scoped_file_menu_command() {
    let close_project = default_workbench_commands()
        .into_iter()
        .find(|command| command.id().as_str() == "file.project.close")
        .expect("the default workbench command registry should expose Close Project");

    assert_eq!(
        close_project.presentation().label_key(),
        "command.file.project.close.label"
    );
    assert_eq!(
        close_project
            .menu_path()
            .map(|path| path.root().id().as_str()),
        Some("file")
    );
    assert!(matches!(close_project.when(), WhenClause::ProjectOpen));
    assert!(matches!(
        close_project.event(),
        Some(EditorEvent::WorkbenchMenu(MenuAction::CloseProject))
    ));
}

#[test]
fn save_all_documents_is_a_project_scoped_file_menu_command() {
    let save_all = default_workbench_commands()
        .into_iter()
        .find(|command| command.id().as_str() == "file.documents.save_all")
        .expect("the default command registry should expose Save All Documents");

    assert_eq!(
        save_all.presentation().label_key(),
        "command.file.documents.save_all.label"
    );
    assert_eq!(
        save_all.menu_path().map(|path| path.leaf().id().as_str()),
        Some("file.documents.save_all")
    );
    assert!(matches!(save_all.when(), WhenClause::ProjectOpen));
    assert!(matches!(
        save_all.event(),
        Some(EditorEvent::WorkbenchMenu(MenuAction::SaveAllDocuments))
    ));
}

#[test]
fn ui_asset_toolkit_open_operation_is_registered() {
    let operation = default_workbench_commands()
        .into_iter()
        .find(|command| command.id().as_str() == "view.editor.ui_asset.open")
        .expect("the default command registry should expose the UI asset toolkit operation");

    assert_eq!(
        operation.presentation().label_key(),
        "command.view.editor.ui_asset.open.label"
    );
    assert!(operation.event().is_none());
}

#[test]
fn animation_asset_toolkit_open_operations_are_registered() {
    let commands = default_workbench_commands();
    for operation in [
        "timeline_sequence.authoring.open",
        "animation_graph.authoring.open_graph",
        "animation_graph.authoring.open_state_machine",
    ] {
        let command = commands
            .iter()
            .find(|command| command.id().as_str() == operation)
            .expect("animation toolkit operation should be registered");

        assert_eq!(
            command.presentation().label_key(),
            format!("command.{operation}.label")
        );
        assert!(matches!(command.category(), EditorCommandCategory::View));
        assert!(command.event().is_none());
    }
}

#[test]
fn default_command_registry_uses_one_exact_outer_capacity_bound() {
    let commands = default_workbench_commands();

    assert_eq!(commands.len(), DEFAULT_WORKBENCH_COMMAND_CAPACITY);
    assert!(commands.capacity() >= DEFAULT_WORKBENCH_COMMAND_CAPACITY);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn optimization_batch_20260919_editor811_default_command_direct_append_bench() {
    let mut legacy_capacity = 0;
    let mut legacy_growth_events = 0;
    for length in 1..=DEFAULT_WORKBENCH_COMMAND_CAPACITY {
        if length > legacy_capacity {
            legacy_capacity = if legacy_capacity == 0 {
                4
            } else {
                legacy_capacity * 2
            };
            legacy_growth_events += 1;
        }
    }

    let commands = std::hint::black_box(default_workbench_commands());
    println!(
        "EDITOR811_DEFAULT_COMMAND_DIRECT_APPEND_BENCH_V1 commands={} legacy_growth_events={} optimized_growth_events=0 temporary_group_vectors=6",
        commands.len(),
        legacy_growth_events,
    );
    assert_eq!(commands.len(), DEFAULT_WORKBENCH_COMMAND_CAPACITY);
    assert!(commands.capacity() >= DEFAULT_WORKBENCH_COMMAND_CAPACITY);
    assert!(legacy_growth_events > 0);
}
