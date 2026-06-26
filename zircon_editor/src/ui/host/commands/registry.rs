use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::ui::component::UiValue;

use crate::core::editor_event::{EditorEvent, EditorEventTransient, MenuAction, ViewDescriptorId};
use crate::core::editor_operation::{EditorOperationPath, EditorOperationRegistry};
use crate::ui::workbench::event::{editor_operation_binding, menu_action_binding};
use crate::ui::workbench::model::{
    operation_path_for_menu_action, MenuBarModel, MenuItemModel, MenuModel,
};

use super::{
    EditorCommandAction, EditorCommandCategory, EditorCommandContext, EditorCommandDescriptor,
    EditorCommandEnablement, EditorCommandPaletteEntry, EditorKeyChord,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandRegistry {
    commands: Vec<EditorCommandDescriptor>,
    by_id: BTreeMap<String, usize>,
}

impl EditorCommandRegistry {
    pub fn new(commands: Vec<EditorCommandDescriptor>) -> Result<Self, EditorCommandRegistryError> {
        let mut by_id = BTreeMap::new();
        for (index, command) in commands.iter().enumerate() {
            if by_id.insert(command.id().to_string(), index).is_some() {
                return Err(EditorCommandRegistryError::DuplicateCommand(
                    command.id().to_string(),
                ));
            }
        }
        Ok(Self { commands, by_id })
    }

    pub fn default_workbench() -> Self {
        Self::new(default_workbench_commands()).expect("default editor command ids are unique")
    }

    pub fn commands(&self) -> &[EditorCommandDescriptor] {
        &self.commands
    }

    pub fn command(&self, id: &str) -> Option<&EditorCommandDescriptor> {
        self.by_id
            .get(id)
            .and_then(|index| self.commands.get(*index))
    }

    pub fn event_for_command(&self, id: &str) -> Result<EditorEvent, EditorCommandDispatchError> {
        let descriptor = self
            .command(id)
            .ok_or_else(|| EditorCommandDispatchError::UnknownCommand(id.to_string()))?;
        match descriptor.action() {
            EditorCommandAction::Menu(action) => Ok(EditorEvent::WorkbenchMenu(action.clone())),
            EditorCommandAction::Operation(operation_id) => {
                event_for_operation_command(id, operation_id)
            }
            EditorCommandAction::OpenCommandPalette => Ok(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette,
            )),
        }
    }

    pub fn command_palette_entries(
        &self,
        context: EditorCommandContext,
    ) -> Vec<EditorCommandPaletteEntry> {
        self.commands
            .iter()
            .map(|descriptor| EditorCommandPaletteEntry::from_descriptor(descriptor, context))
            .collect()
    }

    pub fn command_palette_value(&self, context: EditorCommandContext) -> UiValue {
        UiValue::Array(
            self.command_palette_entries(context)
                .into_iter()
                .map(|entry| entry.to_ui_value())
                .collect(),
        )
    }

    pub fn menu_bar_model(&self, context: EditorCommandContext) -> MenuBarModel {
        const MENU_ORDER: [&str; 7] = [
            "File",
            "Edit",
            "Selection",
            "Play",
            "View",
            "Window",
            "Help",
        ];

        MenuBarModel {
            menus: MENU_ORDER
                .into_iter()
                .filter_map(|label| self.menu_model(label, context))
                .collect(),
        }
    }

    pub fn menu_model(&self, label: &str, context: EditorCommandContext) -> Option<MenuModel> {
        let items = self
            .commands
            .iter()
            .filter_map(|descriptor| command_menu_item(descriptor, label, context))
            .collect::<Vec<_>>();

        if items.is_empty() {
            None
        } else {
            Some(MenuModel {
                label: label.to_string(),
                items,
            })
        }
    }

    pub fn missing_default_keymap_bindings<'a>(
        &'a self,
        keymap: &'a super::EditorKeymap,
    ) -> Vec<&'a str> {
        let keymap_commands = keymap
            .bindings()
            .iter()
            .map(|binding| binding.command_id())
            .collect::<BTreeSet<_>>();
        self.commands
            .iter()
            .filter(|descriptor| descriptor.default_chord().is_some())
            .map(EditorCommandDescriptor::id)
            .filter(|id| !keymap_commands.contains(id))
            .collect()
    }
}

fn command_menu_item(
    descriptor: &EditorCommandDescriptor,
    menu_label: &str,
    context: EditorCommandContext,
) -> Option<MenuItemModel> {
    let menu_path = descriptor.menu_path()?;
    let (top_level, item_label) = menu_path.split_once('/')?;
    if top_level != menu_label {
        return None;
    }

    let label = item_label
        .rsplit('/')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(descriptor.label());
    let shortcut = descriptor.default_chord().map(ToString::to_string);
    let enabled = context.is_enabled(descriptor);

    match descriptor.action() {
        EditorCommandAction::Menu(action) => Some(MenuItemModel::leaf(
            label,
            Some(action.clone()),
            menu_action_binding(action),
            operation_path_for_menu_action(action),
            shortcut,
            enabled,
        )),
        EditorCommandAction::Operation(operation_id) => Some(MenuItemModel::leaf(
            label,
            None,
            editor_operation_binding(operation_id),
            Some(operation_id.clone()),
            shortcut,
            enabled,
        )),
        EditorCommandAction::OpenCommandPalette => None,
    }
}

impl Default for EditorCommandRegistry {
    fn default() -> Self {
        Self::default_workbench()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandRegistryError {
    DuplicateCommand(String),
}

impl std::fmt::Display for EditorCommandRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCommand(id) => write!(formatter, "editor command `{id}` already exists"),
        }
    }
}

impl std::error::Error for EditorCommandRegistryError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandDispatchError {
    UnknownCommand(String),
    OperationNotRegistered {
        command_id: String,
        operation_id: String,
    },
    OperationHasNoHandler {
        command_id: String,
        operation_id: String,
    },
}

impl std::fmt::Display for EditorCommandDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand(id) => write!(formatter, "editor command `{id}` does not exist"),
            Self::OperationNotRegistered {
                command_id,
                operation_id,
            } => write!(
                formatter,
                "editor command `{command_id}` resolves to operation `{operation_id}`, but that operation is not registered"
            ),
            Self::OperationHasNoHandler {
                command_id,
                operation_id,
            } => write!(
                formatter,
                "editor command `{command_id}` resolves to operation `{operation_id}`, but that operation has no event handler"
            ),
        }
    }
}

impl std::error::Error for EditorCommandDispatchError {}

fn default_workbench_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = vec![command_palette_command()];
    commands.extend(file_commands());
    commands.extend(edit_commands());
    commands.extend(selection_commands());
    commands.extend(runtime_commands());
    commands.extend(view_commands());
    commands.extend(window_commands());
    commands.extend(help_commands());
    commands
}

fn command_palette_command() -> EditorCommandDescriptor {
    EditorCommandDescriptor::new(
        "editor.command_palette",
        "Command Palette",
        EditorCommandCategory::Command,
        EditorCommandAction::OpenCommandPalette,
    )
    .with_description("Open global command search")
    .with_menu_path("Help/Command Palette")
    .with_default_chord(chord("Ctrl+Shift+P"))
    .with_keywords(["quick open", "search", "commands"])
}

fn file_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        menu_command(
            "Open Project",
            EditorCommandCategory::File,
            MenuAction::OpenProject,
            Some("Ctrl+O"),
            EditorCommandEnablement::Always,
            ["project", "open"],
        ),
        menu_command(
            "Save Project",
            EditorCommandCategory::File,
            MenuAction::SaveProject,
            Some("Ctrl+S"),
            EditorCommandEnablement::ProjectOpen,
            ["project", "save"],
        ),
        menu_command(
            "Save Layout",
            EditorCommandCategory::File,
            MenuAction::SaveLayout,
            None,
            EditorCommandEnablement::Always,
            ["layout", "workspace"],
        ),
        menu_command(
            "Reset Layout",
            EditorCommandCategory::File,
            MenuAction::ResetLayout,
            Some("Ctrl+Alt+0"),
            EditorCommandEnablement::Always,
            ["layout", "workspace", "reset"],
        ),
    ]
}

fn edit_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        menu_command(
            "Undo",
            EditorCommandCategory::Edit,
            MenuAction::Undo,
            Some("Ctrl+Z"),
            EditorCommandEnablement::UndoAvailable,
            ["history"],
        ),
        menu_command(
            "Redo",
            EditorCommandCategory::Edit,
            MenuAction::Redo,
            Some("Ctrl+Shift+Z"),
            EditorCommandEnablement::RedoAvailable,
            ["history"],
        ),
    ]
}

fn selection_commands() -> Vec<EditorCommandDescriptor> {
    let mut commands = [
        ("Create Cube", NodeKind::Cube),
        ("Create Camera", NodeKind::Camera),
        ("Create Ambient Light", NodeKind::AmbientLight),
        ("Create Directional Light", NodeKind::DirectionalLight),
        ("Create Point Light", NodeKind::PointLight),
        ("Create Rect Light", NodeKind::RectLight),
        ("Create Spot Light", NodeKind::SpotLight),
    ]
    .into_iter()
    .map(|(label, kind)| {
        menu_command(
            label,
            EditorCommandCategory::Selection,
            MenuAction::CreateNode(kind),
            None,
            EditorCommandEnablement::Always,
            ["scene", "node", "create"],
        )
    })
    .collect::<Vec<_>>();
    commands.push(menu_command(
        "Delete Selection",
        EditorCommandCategory::Selection,
        MenuAction::DeleteSelected,
        Some("Delete"),
        EditorCommandEnablement::SelectionPresent,
        ["scene", "node", "delete"],
    ));
    commands
}

fn runtime_commands() -> Vec<EditorCommandDescriptor> {
    vec![
        menu_command(
            "Enter Play Mode",
            EditorCommandCategory::Runtime,
            MenuAction::EnterPlayMode,
            Some("F5"),
            EditorCommandEnablement::CanEnterPlayMode,
            ["play", "run"],
        ),
        menu_command(
            "Exit Play Mode",
            EditorCommandCategory::Runtime,
            MenuAction::ExitPlayMode,
            Some("Shift+F5"),
            EditorCommandEnablement::CanExitPlayMode,
            ["play", "stop"],
        ),
    ]
}

fn view_commands() -> Vec<EditorCommandDescriptor> {
    [
        ("Project", "editor.project"),
        ("Hierarchy", "editor.hierarchy"),
        ("Inspector", "editor.inspector"),
        ("Scene", "editor.scene"),
        ("Game", "editor.game"),
        ("Assets", "editor.assets"),
        ("Console", "editor.console"),
        ("Runtime Diagnostics", "editor.runtime_diagnostics"),
        ("Performance Timeline", "editor.performance_timeline"),
        ("Plugin Manager", "editor.module_plugins"),
        ("Desktop Export", "editor.build_export_desktop"),
        ("Prefab Editor", "editor.prefab"),
        ("Asset Browser", "editor.asset_browser"),
    ]
    .into_iter()
    .map(|(label, descriptor_id)| {
        menu_command(
            format!("Open {label}"),
            EditorCommandCategory::View,
            MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)),
            None,
            EditorCommandEnablement::Always,
            ["view", "panel"],
        )
        .with_menu_path(format!("View/{label}"))
    })
    .collect()
}

fn window_commands() -> Vec<EditorCommandDescriptor> {
    [
        ("Prefab Editor", "editor.prefab_editor_window"),
        ("Material Editor", "editor.material_editor_window"),
        ("UI Component Showcase", "editor.ui_component_showcase"),
        ("Material Demo", "editor.material_demo_window"),
        ("UI Asset Editor", "editor.ui_asset_editor_window"),
        ("Animation Editor", "editor.animation_editor_window"),
        ("Asset Browser", "editor.asset_browser_window"),
        ("Diagnostics", "editor.diagnostics_window"),
        ("Debug Observatory", "editor.debug_observatory"),
    ]
    .into_iter()
    .map(|(label, descriptor_id)| {
        menu_command(
            format!("Open {label}"),
            EditorCommandCategory::Window,
            MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)),
            None,
            EditorCommandEnablement::Always,
            ["window", "tool"],
        )
        .with_menu_path(format!("Window/{label}"))
    })
    .chain(std::iter::once(menu_command_with_id(
        "workbench.window.layout.reset",
        "Reset Layout",
        EditorCommandCategory::Window,
        MenuAction::ResetLayout,
        None,
        EditorCommandEnablement::Always,
        ["layout", "window"],
    )))
    .collect()
}

fn help_commands() -> Vec<EditorCommandDescriptor> {
    vec![menu_command_with_id(
        "workbench.help.guide",
        "Workbench Guide",
        EditorCommandCategory::Help,
        MenuAction::OpenView(ViewDescriptorId::new("editor.asset_browser")),
        None,
        EditorCommandEnablement::Always,
        ["help", "guide", "documentation"],
    )]
}

fn menu_command<I, S>(
    label: impl Into<String>,
    category: EditorCommandCategory,
    action: MenuAction,
    default_chord: Option<&str>,
    enablement: EditorCommandEnablement,
    keywords: I,
) -> EditorCommandDescriptor
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let id = menu_action_id(&action);
    menu_command_with_id(
        id,
        label,
        category,
        action,
        default_chord,
        enablement,
        keywords,
    )
}

fn menu_command_with_id<I, S>(
    id: impl Into<String>,
    label: impl Into<String>,
    category: EditorCommandCategory,
    action: MenuAction,
    default_chord: Option<&str>,
    enablement: EditorCommandEnablement,
    keywords: I,
) -> EditorCommandDescriptor
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let label = label.into();
    let operation_path = operation_path_for_menu_action(&action);
    let command_action = operation_path
        .clone()
        .map(EditorCommandAction::Operation)
        .unwrap_or_else(|| EditorCommandAction::Menu(action.clone()));
    let mut descriptor = EditorCommandDescriptor::new(id, label.clone(), category, command_action)
        .with_menu_path(format!("{}/{}", category.as_str(), label))
        .with_enablement(enablement)
        .with_keywords(keywords);
    if let Some(path) = operation_path.as_ref() {
        descriptor = descriptor.with_description(format!("Invoke editor operation {path}"));
    }
    if let Some(chord_value) = default_chord {
        descriptor = descriptor.with_default_chord(chord(chord_value));
    }
    descriptor
}

fn event_for_operation_command(
    command_id: &str,
    operation_id: &EditorOperationPath,
) -> Result<EditorEvent, EditorCommandDispatchError> {
    let registry = EditorOperationRegistry::with_builtin_operations();
    let descriptor = registry.descriptor(operation_id).ok_or_else(|| {
        EditorCommandDispatchError::OperationNotRegistered {
            command_id: command_id.to_string(),
            operation_id: operation_id.to_string(),
        }
    })?;
    descriptor
        .event()
        .cloned()
        .ok_or_else(|| EditorCommandDispatchError::OperationHasNoHandler {
            command_id: command_id.to_string(),
            operation_id: operation_id.to_string(),
        })
}

fn menu_action_id(action: &MenuAction) -> String {
    let binding = menu_action_binding(action);
    match binding.payload() {
        crate::ui::binding::EditorUiBindingPayload::MenuAction { action_id } => action_id.clone(),
        _ => unreachable!("menu_action_binding always returns a MenuAction payload"),
    }
}

fn chord(value: &str) -> EditorKeyChord {
    EditorKeyChord::from_str(value).expect("built-in command chord is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::host::module::{EDITOR_COMMAND_REGISTRY_NAME, EDITOR_KEYMAP_NAME};
    use crate::ui::host::EditorKeymap;
    use crate::ui::{
        binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind},
        binding_dispatch::editor_event_normalization::normalize_editor_event_binding,
    };
    use zircon_runtime::core::runtime::CoreRuntime;
    use zircon_runtime_interface::ui::dispatch::{
        UiInputEventMetadata, UiInputModifiers, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState,
    };

    #[test]
    fn default_editor_command_registry_covers_menu_and_palette_commands() {
        let registry = EditorCommandRegistry::default_workbench();

        assert!(registry.command("editor.command_palette").is_some());
        assert!(registry.command("workbench.project.open").is_some());
        assert!(registry.command("workbench.history.undo").is_some());
        assert!(registry
            .command("workbench.selection.delete_selected")
            .is_some());
        assert!(registry
            .command("workbench.view.open.editor.hierarchy")
            .is_some());
        assert!(registry
            .command("workbench.view.open.editor.ui_component_showcase")
            .is_some());
        assert!(registry.command("workbench.help.guide").is_some());

        let mut ids = BTreeSet::new();
        for command in registry.commands() {
            assert!(
                ids.insert(command.id()),
                "duplicate command {}",
                command.id()
            );
        }
    }

    #[test]
    fn keymap_resolves_unconsumed_chord_to_command() {
        let keymap = EditorKeymap::default_workbench();

        assert_eq!(
            keymap.resolve(&chord("Ctrl+Shift+P")),
            Some("editor.command_palette")
        );
        assert_eq!(
            keymap.resolve(&chord("Ctrl+S")),
            Some("workbench.project.save")
        );
        assert_eq!(
            keymap.resolve(&chord("Delete")),
            Some("workbench.selection.delete_selected")
        );
        assert_eq!(
            keymap.resolve(&chord("Shift+F5")),
            Some("workbench.play_mode.exit")
        );
        assert_eq!(
            keymap.resolve_keyboard_input(&keyboard_input(
                "o",
                79,
                modifiers(true, false, false, false),
                UiKeyboardInputState::Pressed,
            )),
            Some("workbench.project.open")
        );
        assert_eq!(
            keymap.resolve_keyboard_input(&keyboard_input(
                "F5",
                0,
                modifiers(false, true, false, false),
                UiKeyboardInputState::Pressed,
            )),
            Some("workbench.play_mode.exit")
        );

        let registry = EditorCommandRegistry::default_workbench();
        assert!(
            registry.missing_default_keymap_bindings(&keymap).is_empty(),
            "default keymap must include every command with a default chord"
        );
    }

    #[test]
    fn key_chord_normalizes_runtime_keyboard_input() {
        assert_eq!(
            EditorKeyChord::from_keyboard_input(&keyboard_input(
                "p",
                80,
                modifiers(true, true, false, false),
                UiKeyboardInputState::Pressed,
            )),
            Some(chord("Ctrl+Shift+P"))
        );
        assert_eq!(
            EditorKeyChord::from_keyboard_input(&keyboard_input(
                "Unidentified",
                46,
                UiInputModifiers::default(),
                UiKeyboardInputState::Pressed,
            )),
            Some(chord("Delete"))
        );
        assert_eq!(
            EditorKeyChord::from_keyboard_input(&keyboard_input(
                "o",
                79,
                modifiers(true, false, false, false),
                UiKeyboardInputState::Released,
            )),
            None
        );
        assert_eq!(
            EditorKeyChord::from_keyboard_input(&keyboard_input(
                "Control",
                17,
                modifiers(true, false, false, false),
                UiKeyboardInputState::Pressed,
            )),
            None
        );
    }

    #[test]
    fn palette_filters_commands_by_query_source_and_enablement_metadata() {
        let registry = EditorCommandRegistry::default_workbench();
        let context = EditorCommandContext {
            project_open: false,
            can_undo: false,
            can_redo: true,
            selection_present: false,
            play_mode_active: false,
        };
        let entries = registry.command_palette_entries(context);

        let save_project = entries
            .iter()
            .find(|entry| entry.id == "workbench.project.save")
            .expect("save command should be projected");
        assert!(save_project.disabled);
        assert_eq!(save_project.shortcut, "Ctrl+S");
        assert_eq!(save_project.source, "file");

        let redo = entries
            .iter()
            .find(|entry| entry.id == "workbench.history.redo")
            .expect("redo command should be projected");
        assert!(!redo.disabled);
        assert!(redo.keywords.iter().any(|keyword| keyword == "history"));

        match registry.command_palette_value(context) {
            UiValue::Array(values) => assert!(values.iter().any(|value| {
                matches!(value, UiValue::Map(map) if map.get("id") == Some(&UiValue::String("editor.command_palette".to_string())))
            })),
            other => panic!("expected command palette array, got {other:?}"),
        }
    }

    #[test]
    fn menu_bar_projects_registry_commands_with_contextual_enablement() {
        let registry = EditorCommandRegistry::default_workbench();
        let menu_bar = registry.menu_bar_model(EditorCommandContext {
            project_open: false,
            can_undo: false,
            can_redo: true,
            selection_present: false,
            play_mode_active: false,
        });

        let file = menu(&menu_bar, "File");
        let save_project = item(file, "Save Project");
        assert_eq!(save_project.shortcut.as_deref(), Some("Ctrl+S"));
        assert!(!save_project.enabled);

        let edit = menu(&menu_bar, "Edit");
        assert!(!item(edit, "Undo").enabled);
        assert!(item(edit, "Redo").enabled);
        assert_eq!(item(edit, "Redo").shortcut.as_deref(), Some("Ctrl+Shift+Z"));

        let selection = menu(&menu_bar, "Selection");
        assert!(!item(selection, "Delete Selection").enabled);

        let play = menu(&menu_bar, "Play");
        assert!(!item(play, "Enter Play Mode").enabled);
        assert!(!item(play, "Exit Play Mode").enabled);

        let view = menu(&menu_bar, "View");
        assert!(view.items.iter().any(|item| item.label == "Hierarchy"));
        assert!(!view.items.iter().any(|item| item.label == "Open Hierarchy"));

        let help = menu(&menu_bar, "Help");
        assert!(help
            .items
            .iter()
            .any(|item| item.label == "Workbench Guide"));
        assert!(!help
            .items
            .iter()
            .any(|item| item.label == "Command Palette"));
    }

    #[test]
    fn menu_commands_project_operation_backed_bindings_when_operation_paths_exist() {
        let registry = EditorCommandRegistry::default_workbench();
        let create_cube = registry
            .command("workbench.scene.node.create.cube")
            .expect("create cube command should be registered");
        assert!(matches!(
            create_cube.action(),
            EditorCommandAction::Operation(operation) if operation.as_str() == "scene.node.create_cube"
        ));

        let menu_bar = registry.menu_bar_model(EditorCommandContext {
            project_open: true,
            can_undo: false,
            can_redo: false,
            selection_present: true,
            play_mode_active: false,
        });
        let selection = menu(&menu_bar, "Selection");
        let create_cube = item(selection, "Create Cube");
        assert!(create_cube.action.is_none());
        assert_eq!(
            create_cube.binding.path().control_id,
            "scene.node.create_cube"
        );
        assert_eq!(
            create_cube
                .operation_path
                .as_ref()
                .map(|operation| operation.as_str()),
            Some("scene.node.create_cube")
        );
        assert!(matches!(
            create_cube.binding.payload(),
            EditorUiBindingPayload::EditorOperation { operation_id, .. }
                if operation_id == "scene.node.create_cube"
        ));

        let delete = item(selection, "Delete Selection");

        assert!(delete.action.is_none());
        assert_eq!(
            delete
                .operation_path
                .as_ref()
                .map(|operation| operation.as_str()),
            Some("scene.node.delete_selected")
        );
        assert!(matches!(
            delete.binding.payload(),
            EditorUiBindingPayload::EditorOperation { operation_id, .. }
                if operation_id == "scene.node.delete_selected"
        ));
    }

    #[test]
    fn command_registry_maps_menu_command_ids_to_editor_events() {
        let registry = EditorCommandRegistry::default_workbench();

        assert_eq!(
            registry.event_for_command("workbench.project.save"),
            Ok(EditorEvent::WorkbenchMenu(MenuAction::SaveProject))
        );
        assert_eq!(
            registry.event_for_command("workbench.history.redo"),
            Ok(EditorEvent::WorkbenchMenu(MenuAction::Redo))
        );
        assert_eq!(
            registry.event_for_command("editor.command_palette"),
            Ok(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette
            ))
        );
        assert!(matches!(
            registry.event_for_command("missing.command"),
            Err(EditorCommandDispatchError::UnknownCommand(id)) if id == "missing.command"
        ));
    }

    #[test]
    fn editor_command_binding_normalizes_through_command_registry() {
        let binding = EditorUiBinding::new(
            "CommandPalette",
            "CommandPaletteCommit",
            EditorUiEventKind::Submit,
            EditorUiBindingPayload::editor_command("workbench.history.redo"),
        );

        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding())
                .expect("EditorCommand binding should parse")
                .payload(),
            binding.payload()
        );
        assert_eq!(
            normalize_editor_event_binding(&binding),
            Ok(EditorEvent::WorkbenchMenu(MenuAction::Redo))
        );
    }

    #[test]
    fn editor_module_registers_command_registry_and_keymap_managers() {
        let core = CoreRuntime::default();
        core.register_module(crate::ui::host::module::module_descriptor())
            .expect("editor module descriptor should register");

        let registry = core
            .resolve_manager::<EditorCommandRegistry>(EDITOR_COMMAND_REGISTRY_NAME)
            .expect("command registry manager should resolve");
        let keymap = core
            .resolve_manager::<EditorKeymap>(EDITOR_KEYMAP_NAME)
            .expect("keymap manager should resolve");

        assert!(registry.command("editor.command_palette").is_some());
        assert_eq!(
            keymap.resolve(&chord("Ctrl+Shift+P")),
            Some("editor.command_palette")
        );
    }

    fn menu<'a>(menu_bar: &'a MenuBarModel, label: &str) -> &'a MenuModel {
        menu_bar
            .menus
            .iter()
            .find(|menu| menu.label == label)
            .unwrap_or_else(|| panic!("expected {label} menu"))
    }

    fn item<'a>(menu: &'a MenuModel, label: &str) -> &'a MenuItemModel {
        menu.items
            .iter()
            .find(|item| item.label == label)
            .unwrap_or_else(|| panic!("expected {label} menu item"))
    }

    fn keyboard_input(
        logical_key: &str,
        key_code: u32,
        modifiers: UiInputModifiers,
        state: UiKeyboardInputState,
    ) -> UiKeyboardInputEvent {
        let mut metadata =
            UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
        metadata.modifiers = modifiers;
        UiKeyboardInputEvent {
            metadata,
            state,
            key_code,
            scan_code: None,
            physical_key: logical_key.to_string(),
            logical_key: logical_key.to_string(),
            text: None,
        }
    }

    fn modifiers(ctrl: bool, shift: bool, alt: bool, meta: bool) -> UiInputModifiers {
        UiInputModifiers {
            control: ctrl,
            shift,
            alt,
            super_key: meta,
            ..UiInputModifiers::default()
        }
    }
}
