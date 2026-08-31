use std::str::FromStr;

use zircon_runtime::scene::components::NodeKind;

use crate::core::editor_event::{
    ConsoleMessageFilter, ConsoleSourceFilter, EditorEvent, EditorEventTransient, LayoutCommand,
    MenuAction, ViewDescriptorId,
};
use crate::core::editor_operation::EditorOperationPath;

use super::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor, EditorCommandMenuPath,
    EditorKeyChord, PlayModePredicate, WhenClause,
};

pub(super) fn default_workbench_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = vec![command_palette_command()];
    commands.extend(file_commands());
    commands.extend(edit_commands());
    commands.extend(selection_commands());
    commands.extend(runtime_commands());
    commands.extend(view_commands());
    commands.extend(window_commands());
    commands.push(migrate_assets_commandlet());
    commands.push(plugin_list_commandlet());
    commands.push(authoring_automation_commandlet());
    commands.push(
        EditorCommandDescriptor::operation(path("view.editor.ui_asset.open"))
            .with_category(EditorCommandCategory::View)
            .with_callable_from_remote(false),
    );
    commands.extend(animation_asset_toolkit_open_operations());
    commands.push(
        EditorCommandDescriptor::operation(path("inspector.field.apply_batch"))
            .with_category(EditorCommandCategory::Edit)
            .with_menu_path(builtin_menu(
                "inspector.field.apply_batch",
                "inspector",
                &[],
            ))
            .with_callable_from_remote(false),
    );
    commands.push(command(
        "help.workbench.guide",
        EditorCommandCategory::Help,
        "help",
        &[],
        EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(
            "editor.asset_browser",
        ))),
        None,
        WhenClause::Always,
        ["help", "guide", "documentation"],
    ));
    commands
}

fn animation_asset_toolkit_open_operations() -> [EditorCommandDescriptor; 3] {
    [
        "timeline_sequence.authoring.open",
        "animation_graph.authoring.open_graph",
        "animation_graph.authoring.open_state_machine",
    ]
    .map(|operation| {
        EditorCommandDescriptor::operation(path(operation))
            .with_category(EditorCommandCategory::View)
            .with_callable_from_remote(false)
    })
}

fn migrate_assets_commandlet() -> EditorCommandDescriptor {
    EditorCommandDescriptor::new(
        path("asset.migration.migrate_assets"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessAssetMigration,
    )
    .with_keywords(["asset", "migration", "migrate-assets", "headless"])
    .with_payload_schema_id("editor.commandlet.migrate-assets")
    .with_headless_commandlet_route(path("commandlet.route.migrate_assets"))
    .with_headless_commandlet_name("migrate-assets")
    .with_callable_from_remote(true)
    .with_required_capabilities(["asset.migration"])
}

fn plugin_list_commandlet() -> EditorCommandDescriptor {
    EditorCommandDescriptor::new(
        path("plugin.catalog.list"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessPluginList,
    )
    .with_keywords(["plugin", "plugins", "catalog", "plugin-list", "headless"])
    .with_payload_schema_id("editor.commandlet.plugin-list")
    .with_headless_commandlet_route(path("commandlet.route.plugin_list"))
    .with_headless_commandlet_name("plugin-list")
    .with_callable_from_remote(true)
    .with_required_capabilities(["plugin.catalog.read"])
}

fn authoring_automation_commandlet() -> EditorCommandDescriptor {
    EditorCommandDescriptor::new(
        path("authoring.automation.run"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessAuthoringAutomation,
    )
    .with_keywords(["authoring", "automation", "headless", "retained-host"])
    .with_payload_schema_id("editor.commandlet.authoring-automation")
    .with_headless_commandlet_route(path("commandlet.route.authoring_automation"))
    .with_headless_commandlet_name("authoring-automation")
    .with_callable_from_remote(true)
}

fn command_palette_command() -> EditorCommandDescriptor {
    command(
        "editor.command.palette",
        EditorCommandCategory::Command,
        "help",
        &[],
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        Some("Ctrl+Shift+P"),
        WhenClause::Always,
        ["quick open", "search", "commands"],
    )
    .with_callable_from_remote(false)
}

fn file_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        command(
            "file.project.open",
            EditorCommandCategory::File,
            "file",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::OpenProject),
            Some("Ctrl+O"),
            WhenClause::Always,
            ["project", "open"],
        ),
        command(
            "file.project.save",
            EditorCommandCategory::File,
            "file",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
            Some("Ctrl+S"),
            WhenClause::ProjectOpen,
            ["project", "save"],
        ),
        command(
            "file.documents.save_all",
            EditorCommandCategory::File,
            "file",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::SaveAllDocuments),
            Some("Ctrl+Shift+S"),
            WhenClause::ProjectOpen,
            ["save", "all", "documents"],
        ),
        command(
            "file.project.close",
            EditorCommandCategory::File,
            "file",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::CloseProject),
            None,
            WhenClause::ProjectOpen,
            ["project", "close"],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

fn edit_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        command(
            "edit.history.undo",
            EditorCommandCategory::Edit,
            "edit",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::Undo),
            Some("Ctrl+Z"),
            WhenClause::UndoAvailable,
            ["history"],
        ),
        command(
            "edit.history.redo",
            EditorCommandCategory::Edit,
            "edit",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::Redo),
            Some("Ctrl+Shift+Z"),
            WhenClause::RedoAvailable,
            ["history"],
        ),
        command(
            "editor.settings.open",
            EditorCommandCategory::Edit,
            "edit",
            &[],
            EditorEvent::Transient(EditorEventTransient::OpenSettingsWindow),
            None,
            WhenClause::Always,
            ["settings", "preferences", "configuration"],
        ),
    ]
}

fn selection_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        ("scene.node.create_cube", NodeKind::Cube),
        ("scene.node.create_camera", NodeKind::Camera),
        ("scene.node.create_ambient_light", NodeKind::AmbientLight),
        (
            "scene.node.create_directional_light",
            NodeKind::DirectionalLight,
        ),
        ("scene.node.create_point_light", NodeKind::PointLight),
        ("scene.node.create_rect_light", NodeKind::RectLight),
        ("scene.node.create_spot_light", NodeKind::SpotLight),
    ]
    .into_iter()
    .map(|(id, kind)| {
        command(
            id,
            EditorCommandCategory::Selection,
            "selection",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::CreateNode(kind)),
            None,
            WhenClause::Always,
            ["scene", "node", "create"],
        )
    })
    .collect::<Vec<_>>();
    commands.push(command(
        "scene.node.delete_selected",
        EditorCommandCategory::Selection,
        "selection",
        &[],
        EditorEvent::WorkbenchMenu(MenuAction::DeleteSelected),
        Some("Delete"),
        WhenClause::SelectionNonEmpty,
        ["scene", "node", "delete"],
    ));
    commands
}

fn runtime_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        command(
            "runtime.play_mode.enter",
            EditorCommandCategory::Runtime,
            "play",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
            Some("F5"),
            WhenClause::All(vec![
                WhenClause::ProjectOpen,
                WhenClause::PlayMode(PlayModePredicate::Edit),
            ]),
            ["play", "run"],
        ),
        command(
            "runtime.play_mode.keep_changes",
            EditorCommandCategory::Runtime,
            "play",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::KeepPlayChanges),
            None,
            WhenClause::All(vec![
                WhenClause::ProjectOpen,
                WhenClause::PlayMode(PlayModePredicate::Playing),
                WhenClause::SelectionNonEmpty,
                WhenClause::AssetWritable,
            ]),
            ["play", "keep", "properties", "authoring"],
        ),
        command(
            "runtime.play_mode.exit",
            EditorCommandCategory::Runtime,
            "play",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
            Some("Shift+F5"),
            WhenClause::Any(vec![
                WhenClause::PlayMode(PlayModePredicate::Playing),
                WhenClause::PlayMode(PlayModePredicate::CleanupFailed),
            ]),
            ["play", "stop"],
        ),
    ]
}

fn view_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        ("view.project.open", "editor.project"),
        ("view.hierarchy.open", "editor.hierarchy"),
        ("view.inspector.open", "editor.inspector"),
        ("view.scene.open", "editor.scene"),
        ("view.game.open", "editor.game"),
        ("view.assets.open", "editor.assets"),
        ("view.console.open", "editor.console"),
        (
            "view.runtime_diagnostics.open",
            "editor.runtime_diagnostics",
        ),
        (
            "view.performance_timeline.open",
            "editor.performance_timeline",
        ),
        ("view.plugin_manager.open", "editor.module_plugins"),
        ("view.build_export.open", "editor.build_export_desktop"),
        ("view.prefab.open", "editor.prefab"),
        ("view.asset_browser.open", "editor.asset_browser"),
    ]
    .into_iter()
    .map(|(id, view_id)| {
        command(
            id,
            EditorCommandCategory::View,
            "view",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view_id))),
            None,
            WhenClause::Always,
            ["view", "panel"],
        )
    })
    .collect::<Vec<_>>();
    commands.push(command(
        "view.console.clear",
        EditorCommandCategory::View,
        "view",
        &[],
        EditorEvent::WorkbenchMenu(MenuAction::ClearConsole),
        None,
        WhenClause::Always,
        ["console", "clear", "output", "log"],
    ));
    for (id, filter) in [
        ("view.console.filter.all", ConsoleMessageFilter::All),
        ("view.console.filter.info", ConsoleMessageFilter::Info),
        ("view.console.filter.warning", ConsoleMessageFilter::Warning),
        ("view.console.filter.error", ConsoleMessageFilter::Error),
    ] {
        commands.push(command(
            id,
            EditorCommandCategory::View,
            "view",
            &["console"],
            EditorEvent::WorkbenchMenu(MenuAction::SetConsoleMessageFilter(filter)),
            None,
            WhenClause::Always,
            ["console", "filter", filter.as_str()],
        ));
    }
    for (id, filter) in [
        ("view.console.source.all", ConsoleSourceFilter::All),
        ("view.console.source.editor", ConsoleSourceFilter::Editor),
        ("view.console.source.runtime", ConsoleSourceFilter::Runtime),
        ("view.console.source.play", ConsoleSourceFilter::Play),
        ("view.console.source.plugin", ConsoleSourceFilter::Plugin),
        ("view.console.source.import", ConsoleSourceFilter::Import),
        (
            "view.console.source.script_build",
            ConsoleSourceFilter::ScriptBuild,
        ),
    ] {
        commands.push(command(
            id,
            EditorCommandCategory::View,
            "view",
            &["activity_log"],
            EditorEvent::WorkbenchMenu(MenuAction::SetConsoleSourceFilter(filter)),
            None,
            WhenClause::Always,
            ["activity", "log", "source", filter.as_str()],
        ));
    }
    commands
}

fn window_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        ("window.prefab_editor.open", "editor.prefab_editor_window"),
        (
            "window.material_editor.open",
            "editor.material_editor_window",
        ),
        (
            "window.ui_component_showcase.open",
            "editor.ui_component_showcase",
        ),
        ("window.material_demo.open", "editor.material_demo_window"),
        (
            "window.material_component_lab.open",
            "editor.material_component_lab",
        ),
        (
            "window.ui_asset_editor.open",
            "editor.ui_asset_editor_window",
        ),
        (
            "window.animation_editor.open",
            "editor.animation_editor_window",
        ),
        ("window.asset_browser.open", "editor.asset_browser_window"),
        ("window.diagnostics.open", "editor.diagnostics_window"),
        ("window.debug_observatory.open", "editor.debug_observatory"),
    ]
    .into_iter()
    .map(|(id, view_id)| {
        command(
            id,
            EditorCommandCategory::Window,
            "window",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view_id))),
            None,
            WhenClause::Always,
            ["window", "tool"],
        )
    })
    .collect::<Vec<_>>();
    commands.extend([
        command(
            "window.layout.save",
            EditorCommandCategory::Window,
            "window",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::SaveLayout),
            None,
            WhenClause::Always,
            ["layout", "workspace"],
        ),
        command(
            "window.layout.reset",
            EditorCommandCategory::Window,
            "window",
            &[],
            EditorEvent::WorkbenchMenu(MenuAction::ResetLayout),
            Some("Ctrl+Alt+0"),
            WhenClause::Always,
            ["layout", "workspace", "reset"],
        ),
        command(
            "window.layout.default",
            EditorCommandCategory::Window,
            "window",
            &["layout"],
            EditorEvent::Layout(LayoutCommand::ResetToDefault),
            None,
            WhenClause::Always,
            ["layout", "default"],
        ),
    ]);
    commands
}

fn command<I, S>(
    id: &str,
    category: EditorCommandCategory,
    menu_root: &str,
    menu_groups: &[&str],
    event: EditorEvent,
    default_chord: Option<&str>,
    when: WhenClause,
    keywords: I,
) -> EditorCommandDescriptor
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut descriptor =
        EditorCommandDescriptor::new(path(id), category, EditorCommandAction::Emit(event))
            .with_menu_path(builtin_menu(id, menu_root, menu_groups))
            .with_when(when)
            .with_keywords(keywords);
    if let Some(chord_value) = default_chord {
        descriptor = descriptor.with_default_chord(
            EditorKeyChord::from_str(chord_value).expect("built-in command chord is valid"),
        );
    }
    descriptor
}

fn builtin_menu(id: &str, root: &str, groups: &[&str]) -> EditorCommandMenuPath {
    EditorCommandMenuPath::builtin(&path(id), root, groups)
}

fn path(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("built-in command path is valid")
}
