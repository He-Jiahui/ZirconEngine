use std::{collections::BTreeMap, sync::LazyLock};

use super::material_lab_template_bindings::material_lab_template_bindings;
use super::showcase_template_bindings::showcase_template_bindings;
use super::workbench_window_template_bindings::workbench_window_template_bindings;
use crate::core::editor_event::InspectorFieldChange;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    DisplayMode, GridMode, PivotMode, ProjectionMode, TransformSpace, ViewOrientation,
};
use crate::ui::binding::{
    AssetCommand, DockCommand, DraftCommand, EditorUiBinding, EditorUiBindingPayload,
    EditorUiEventKind, SelectionCommand, ViewportCommand, WelcomeCommand,
};
use zircon_runtime_interface::ui::binding::UiBindingValue;

const DYNAMIC_DOCUMENT_TAB_INSTANCE_ID: &str = "$document_tab_instance";
const DYNAMIC_MAIN_PAGE_ID: &str = "$main_page_id";

pub(crate) fn builtin_template_bindings() -> &'static BTreeMap<String, EditorUiBinding> {
    static BINDINGS: LazyLock<BTreeMap<String, EditorUiBinding>> =
        LazyLock::new(build_builtin_template_bindings);
    &BINDINGS
}

fn build_builtin_template_bindings() -> BTreeMap<String, EditorUiBinding> {
    let mut bindings = BTreeMap::from([
        (
            "WorkbenchMenuBar/OpenProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.project.open"),
            ),
        ),
        (
            "WorkbenchMenuBar/SaveProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.project.save"),
            ),
        ),
        (
            "WorkbenchMenuBar/ResetLayout".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "ResetLayout",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.layout.reset"),
            ),
        ),
        (
            "ActivityRail/HierarchyToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "HierarchyToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "left_top".to_string(),
                    instance_id: "editor.hierarchy#1".to_string(),
                }),
            ),
        ),
        (
            "ActivityRail/AssetsToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "AssetsToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "left_top".to_string(),
                    instance_id: "editor.assets#1".to_string(),
                }),
            ),
        ),
        (
            "ActivityRail/ConsoleToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "ConsoleToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "bottom_left".to_string(),
                    instance_id: "editor.console#1".to_string(),
                }),
            ),
        ),
        (
            "DocumentTabs/ActivateTab".to_string(),
            EditorUiBinding::new(
                "DocumentTabs",
                "ActivateTab",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "DocumentTabs/CloseTab".to_string(),
            EditorUiBinding::new(
                "DocumentTabs",
                "CloseTab",
                EditorUiEventKind::Submit,
                EditorUiBindingPayload::dock_command(DockCommand::CloseView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "Workbench/ActivateDocumentTab".to_string(),
            EditorUiBinding::new(
                "Workbench",
                "ActivateDocumentTab",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "Workbench/CloseDocumentTab".to_string(),
            EditorUiBinding::new(
                "Workbench",
                "CloseDocumentTab",
                EditorUiEventKind::Submit,
                EditorUiBindingPayload::dock_command(DockCommand::CloseView {
                    instance_id: DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string(),
                }),
            ),
        ),
        (
            "UiHostWindow/ActivateMainPage".to_string(),
            EditorUiBinding::new(
                "UiHostWindow",
                "ActivateMainPage",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateMainPage {
                    page_id: DYNAMIC_MAIN_PAGE_ID.to_string(),
                }),
            ),
        ),
        (
            "ViewportToolbar/ActivateSceneMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "ActivateSceneMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
                    SceneModeActivation::Select,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetTransformSpace".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTransformSpace",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTransformSpace(
                    TransformSpace::Local,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetPivotMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetPivotMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPivotMode(
                    PivotMode::Centroid,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetProjectionMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetProjectionMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetProjectionMode(
                    ProjectionMode::Perspective,
                )),
            ),
        ),
        (
            "ViewportToolbar/AlignView".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "AlignView",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::AlignView(
                    ViewOrientation::User,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetDisplayMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetDisplayMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
                    DisplayMode::Shaded,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetGridMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetGridMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
                    GridMode::Hidden,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetTranslateSnap".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTranslateSnap",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTranslateSnap(0.1)),
            ),
        ),
        (
            "ViewportToolbar/SetRotateSnapDegrees".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetRotateSnapDegrees",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetRotateSnapDegrees(
                    5.0,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetScaleSnap".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetScaleSnap",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetScaleSnap(0.05)),
            ),
        ),
        (
            "ViewportToolbar/SetPreviewLighting".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetPreviewLighting",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewLighting(
                    false,
                )),
            ),
        ),
        (
            "ViewportToolbar/SetPreviewSkybox".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetPreviewSkybox",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewSkybox(false)),
            ),
        ),
        (
            "ViewportToolbar/SetGizmosEnabled".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetGizmosEnabled",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGizmosEnabled(true)),
            ),
        ),
        (
            "ViewportToolbar/FrameSelection".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "FrameSelection",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::FrameSelection),
            ),
        ),
        (
            "ViewportToolbar/EnterPlayMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "EnterPlayMode",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.play_mode.enter"),
            ),
        ),
        (
            "ViewportToolbar/ExitPlayMode".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "ExitPlayMode",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.play_mode.exit"),
            ),
        ),
        (
            "AssetSurface/SelectFolder".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SelectFolder",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SelectFolder {
                    folder_id: "Assets".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SelectItem".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SelectItem",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SelectItem {
                    asset_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SearchEdited".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SearchEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetSearchQuery {
                    query: String::new(),
                }),
            ),
        ),
        (
            "AssetSurface/SetKindFilter".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetKindFilter",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetKindFilter {
                    kind: String::new(),
                }),
            ),
        ),
        (
            "AssetSurface/SetViewMode".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetViewMode",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
                    surface: "activity".to_string(),
                    view_mode: "list".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/SetUtilityTab".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "SetUtilityTab",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::asset_command(AssetCommand::SetUtilityTab {
                    surface: "activity".to_string(),
                    tab: "preview".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/ActivateReference".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "ActivateReference",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::ActivateReference {
                    asset_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                }),
            ),
        ),
        (
            "AssetSurface/OpenAssetBrowser".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "OpenAssetBrowser",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser),
            ),
        ),
        (
            "AssetSurface/LocateSelectedAsset".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "LocateSelectedAsset",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::LocateSelectedAsset),
            ),
        ),
        (
            "AssetSurface/ImportModel".to_string(),
            EditorUiBinding::new(
                "AssetSurface",
                "ImportModel",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
            ),
        ),
        (
            "WelcomeSurface/ProjectNameEdited".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "ProjectNameEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
                    value: String::new(),
                }),
            ),
        ),
        (
            "WelcomeSurface/LocationEdited".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "LocationEdited",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::SetLocation {
                    value: String::new(),
                }),
            ),
        ),
        (
            "WelcomeSurface/CreateProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "CreateProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::CreateProject),
            ),
        ),
        (
            "WelcomeSurface/OpenExistingProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "OpenExistingProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenExistingProject),
            ),
        ),
        (
            "WelcomeSurface/OpenRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "OpenRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "WelcomeSurface/RecoverRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "RecoverRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::RecoverRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "WelcomeSurface/SafeRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "SafeRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::SafeRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "WelcomeSurface/RemoveRecentProject".to_string(),
            EditorUiBinding::new(
                "WelcomeSurface",
                "RemoveRecentProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::welcome_command(WelcomeCommand::RemoveRecentProject {
                    path: "E:/Projects/Sandbox".to_string(),
                }),
            ),
        ),
        (
            "InspectorView/NameField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "NameField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "name".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/ParentField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "ParentField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "parent".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionXField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionXField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.x".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionYField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionYField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.y".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/PositionZField".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "PositionZField",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "transform.translation.z".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
            ),
        ),
        (
            "InspectorView/ApplyBatchButton".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "ApplyBatchButton",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::inspector_field_batch(
                    "entity://selected",
                    Vec::<InspectorFieldChange>::new(),
                ),
            ),
        ),
        (
            "InspectorView/DeleteSelected".to_string(),
            EditorUiBinding::new(
                "InspectorView",
                "DeleteSelected",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.selection.delete_selected"),
            ),
        ),
        (
            "ConsolePaneBody/ClearConsole".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "ClearConsole",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.clear"),
            ),
        ),
        (
            "ConsolePaneBody/FilterAll".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "FilterAll",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.filter.all"),
            ),
        ),
        (
            "ConsolePaneBody/FilterError".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "FilterError",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.filter.error"),
            ),
        ),
        (
            "ConsolePaneBody/FilterWarning".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "FilterWarning",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.filter.warning"),
            ),
        ),
        (
            "ConsolePaneBody/FilterInfo".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "FilterInfo",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.filter.info"),
            ),
        ),
        (
            "ConsolePaneBody/SourceAll".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourceAll",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.all"),
            ),
        ),
        (
            "ConsolePaneBody/SourceEditor".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourceEditor",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.editor"),
            ),
        ),
        (
            "ConsolePaneBody/SourceRuntime".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourceRuntime",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.runtime"),
            ),
        ),
        (
            "ConsolePaneBody/SourcePlay".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourcePlay",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.play"),
            ),
        ),
        (
            "ConsolePaneBody/SourcePlugin".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourcePlugin",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.plugin"),
            ),
        ),
        (
            "ConsolePaneBody/SourceImport".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourceImport",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.import"),
            ),
        ),
        (
            "ConsolePaneBody/SourceScriptBuild".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "SourceScriptBuild",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::editor_operation("view.console.source.script_build"),
            ),
        ),
        (
            "HierarchyPaneBody/SelectRoot".to_string(),
            EditorUiBinding::new(
                "HierarchyPaneBody",
                "SelectRoot",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
                    node_id: 0,
                }),
            ),
        ),
        (
            "RuntimeDiagnosticsPaneBody/FocusDiagnostics".to_string(),
            EditorUiBinding::new(
                "RuntimeDiagnosticsPaneBody",
                "FocusDiagnostics",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: "editor.runtime_diagnostics#1".to_string(),
                }),
            ),
        ),
        (
            "ModulePluginsPaneBody/FocusModulePlugins".to_string(),
            EditorUiBinding::new(
                "ModulePluginsPaneBody",
                "FocusModulePlugins",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: "editor.module_plugins#1".to_string(),
                }),
            ),
        ),
        (
            "BuildExportPaneBody/FocusBuildExport".to_string(),
            EditorUiBinding::new(
                "BuildExportPaneBody",
                "FocusBuildExport",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: "editor.build_export_desktop#1".to_string(),
                }),
            ),
        ),
    ]);
    bindings.extend(workbench_window_template_bindings());
    bindings.extend(showcase_template_bindings());
    bindings.extend(material_lab_template_bindings());
    bindings
}

#[cfg(test)]
mod performance_tests {
    use super::builtin_template_bindings;

    #[test]
    fn builtin_template_binding_registry_is_process_cached() {
        let first = builtin_template_bindings();
        let second = builtin_template_bindings();

        assert!(std::ptr::eq(first, second));
        assert!(first.contains_key("WorkbenchMenuBar/OpenProject"));
    }

    #[test]
    fn componentized_document_tabs_share_the_canonical_dynamic_payloads() {
        let bindings = builtin_template_bindings();

        for (workbench_id, canonical_id) in [
            ("Workbench/ActivateDocumentTab", "DocumentTabs/ActivateTab"),
            ("Workbench/CloseDocumentTab", "DocumentTabs/CloseTab"),
        ] {
            let workbench = bindings
                .get(workbench_id)
                .unwrap_or_else(|| panic!("{workbench_id} should be registered"));
            let canonical = bindings
                .get(canonical_id)
                .unwrap_or_else(|| panic!("{canonical_id} should be registered"));
            assert_eq!(workbench.path().event_kind, canonical.path().event_kind);
            assert_eq!(workbench.payload(), canonical.payload());
        }
    }
}
