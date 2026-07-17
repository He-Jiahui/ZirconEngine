use std::str::FromStr;

use zircon_runtime::scene::components::NodeKind;

use crate::core::editor_event::{
    EditorEvent, EditorEventTransient, LayoutCommand, MenuAction, ViewDescriptorId,
};
use crate::core::editor_operation::EditorOperationPath;

use super::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor, EditorKeyChord,
    PlayModePredicate, WhenClause,
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
    commands.push(
        EditorCommandDescriptor::operation(
            path("inspector.field.apply_batch"),
            "Apply Inspector Changes",
        )
        .with_category(EditorCommandCategory::Edit)
        .with_menu_path("Inspector/Apply Changes")
        .with_callable_from_remote(false),
    );
    commands.push(command(
        "help.workbench.guide",
        "Workbench Guide",
        EditorCommandCategory::Help,
        "Help/Workbench Guide",
        EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(
            "editor.asset_browser",
        ))),
        None,
        WhenClause::Always,
        ["help", "guide", "documentation"],
    ));
    commands
}

fn migrate_assets_commandlet() -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(
        path("asset.migration.migrate_assets"),
        "Migrate Project Assets",
    )
    .with_category(EditorCommandCategory::Command)
    .with_description("Migrate authoring assets through the headless migrate-assets commandlet")
    .with_keywords(["asset", "migration", "migrate-assets", "headless"])
    .with_payload_schema_id("editor.commandlet.migrate-assets")
    .with_required_capabilities(["asset.migration"])
}

fn command_palette_command() -> EditorCommandDescriptor {
    command(
        "editor.command.palette",
        "Command Palette",
        EditorCommandCategory::Command,
        "Help/Command Palette",
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        Some("Ctrl+Shift+P"),
        WhenClause::Always,
        ["quick open", "search", "commands"],
    )
    .with_description("Open global command search")
    .with_callable_from_remote(false)
}

fn file_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        command(
            "file.project.open",
            "Open Project",
            EditorCommandCategory::File,
            "File/Open Project",
            EditorEvent::WorkbenchMenu(MenuAction::OpenProject),
            Some("Ctrl+O"),
            WhenClause::Always,
            ["project", "open"],
        ),
        command(
            "file.project.save",
            "Save Project",
            EditorCommandCategory::File,
            "File/Save Project",
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
            Some("Ctrl+S"),
            WhenClause::ProjectOpen,
            ["project", "save"],
        ),
    ]
}

fn edit_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        command(
            "edit.history.undo",
            "Undo",
            EditorCommandCategory::Edit,
            "Edit/Undo",
            EditorEvent::WorkbenchMenu(MenuAction::Undo),
            Some("Ctrl+Z"),
            WhenClause::UndoAvailable,
            ["history"],
        ),
        command(
            "edit.history.redo",
            "Redo",
            EditorCommandCategory::Edit,
            "Edit/Redo",
            EditorEvent::WorkbenchMenu(MenuAction::Redo),
            Some("Ctrl+Shift+Z"),
            WhenClause::RedoAvailable,
            ["history"],
        ),
    ]
}

fn selection_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        ("scene.node.create_cube", "Create Cube", NodeKind::Cube),
        (
            "scene.node.create_camera",
            "Create Camera",
            NodeKind::Camera,
        ),
        (
            "scene.node.create_ambient_light",
            "Create Ambient Light",
            NodeKind::AmbientLight,
        ),
        (
            "scene.node.create_directional_light",
            "Create Directional Light",
            NodeKind::DirectionalLight,
        ),
        (
            "scene.node.create_point_light",
            "Create Point Light",
            NodeKind::PointLight,
        ),
        (
            "scene.node.create_rect_light",
            "Create Rect Light",
            NodeKind::RectLight,
        ),
        (
            "scene.node.create_spot_light",
            "Create Spot Light",
            NodeKind::SpotLight,
        ),
    ]
    .into_iter()
    .map(|(id, display_name, kind)| {
        command(
            id,
            display_name,
            EditorCommandCategory::Selection,
            format!("Selection/{display_name}"),
            EditorEvent::WorkbenchMenu(MenuAction::CreateNode(kind)),
            None,
            WhenClause::Always,
            ["scene", "node", "create"],
        )
    })
    .collect::<Vec<_>>();
    commands.push(command(
        "scene.node.delete_selected",
        "Delete Selection",
        EditorCommandCategory::Selection,
        "Selection/Delete Selection",
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
            "Enter Play Mode",
            EditorCommandCategory::Runtime,
            "Play/Enter Play Mode",
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
            Some("F5"),
            WhenClause::All(vec![
                WhenClause::ProjectOpen,
                WhenClause::PlayMode(PlayModePredicate::Edit),
            ]),
            ["play", "run"],
        ),
        command(
            "runtime.play_mode.exit",
            "Exit Play Mode",
            EditorCommandCategory::Runtime,
            "Play/Exit Play Mode",
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
            Some("Shift+F5"),
            WhenClause::PlayMode(PlayModePredicate::Playing),
            ["play", "stop"],
        ),
    ]
}

fn view_commands() -> Vec<EditorCommandDescriptor> {
    [
        ("view.project.open", "Project", "editor.project"),
        ("view.hierarchy.open", "Hierarchy", "editor.hierarchy"),
        ("view.inspector.open", "Inspector", "editor.inspector"),
        ("view.scene.open", "Scene", "editor.scene"),
        ("view.game.open", "Game", "editor.game"),
        ("view.assets.open", "Assets", "editor.assets"),
        ("view.console.open", "Console", "editor.console"),
        (
            "view.runtime_diagnostics.open",
            "Runtime Diagnostics",
            "editor.runtime_diagnostics",
        ),
        (
            "view.performance_timeline.open",
            "Performance Timeline",
            "editor.performance_timeline",
        ),
        (
            "view.plugin_manager.open",
            "Plugin Manager",
            "editor.module_plugins",
        ),
        (
            "view.build_export.open",
            "Desktop Export",
            "editor.build_export_desktop",
        ),
        ("view.prefab.open", "Prefab Editor", "editor.prefab"),
        (
            "view.asset_browser.open",
            "Asset Browser",
            "editor.asset_browser",
        ),
    ]
    .into_iter()
    .map(|(id, label, view_id)| {
        command(
            id,
            format!("Open {label}"),
            EditorCommandCategory::View,
            format!("View/{label}"),
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view_id))),
            None,
            WhenClause::Always,
            ["view", "panel"],
        )
    })
    .collect()
}

fn window_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        (
            "window.prefab_editor.open",
            "Prefab Editor",
            "editor.prefab_editor_window",
        ),
        (
            "window.material_editor.open",
            "Material Editor",
            "editor.material_editor_window",
        ),
        (
            "window.ui_component_showcase.open",
            "UI Component Showcase",
            "editor.ui_component_showcase",
        ),
        (
            "window.material_demo.open",
            "Material Demo",
            "editor.material_demo_window",
        ),
        (
            "window.material_component_lab.open",
            "Material Component Lab",
            "editor.material_component_lab",
        ),
        (
            "window.ui_asset_editor.open",
            "UI Asset Editor",
            "editor.ui_asset_editor_window",
        ),
        (
            "window.animation_editor.open",
            "Animation Editor",
            "editor.animation_editor_window",
        ),
        (
            "window.asset_browser.open",
            "Asset Browser",
            "editor.asset_browser_window",
        ),
        (
            "window.diagnostics.open",
            "Diagnostics",
            "editor.diagnostics_window",
        ),
        (
            "window.debug_observatory.open",
            "Debug Observatory",
            "editor.debug_observatory",
        ),
    ]
    .into_iter()
    .map(|(id, label, view_id)| {
        command(
            id,
            format!("Open {label}"),
            EditorCommandCategory::Window,
            format!("Window/{label}"),
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
            "Save Layout",
            EditorCommandCategory::Window,
            "Window/Save Layout",
            EditorEvent::WorkbenchMenu(MenuAction::SaveLayout),
            None,
            WhenClause::Always,
            ["layout", "workspace"],
        ),
        command(
            "window.layout.reset",
            "Reset Layout",
            EditorCommandCategory::Window,
            "Window/Reset Layout",
            EditorEvent::WorkbenchMenu(MenuAction::ResetLayout),
            Some("Ctrl+Alt+0"),
            WhenClause::Always,
            ["layout", "workspace", "reset"],
        ),
        command(
            "window.layout.default",
            "Load Default Layout",
            EditorCommandCategory::Window,
            "Window/Layout/Default",
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
    display_name: impl Into<String>,
    category: EditorCommandCategory,
    menu_path: impl Into<String>,
    event: EditorEvent,
    default_chord: Option<&str>,
    when: WhenClause,
    keywords: I,
) -> EditorCommandDescriptor
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut descriptor = EditorCommandDescriptor::new(
        path(id),
        display_name,
        category,
        EditorCommandAction::Emit(event),
    )
    .with_menu_path(menu_path)
    .with_when(when)
    .with_keywords(keywords);
    if let Some(chord_value) = default_chord {
        descriptor = descriptor.with_default_chord(
            EditorKeyChord::from_str(chord_value).expect("built-in command chord is valid"),
        );
    }
    descriptor
}

fn path(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("built-in command path is valid")
}
