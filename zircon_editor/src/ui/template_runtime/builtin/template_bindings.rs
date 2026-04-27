use std::collections::BTreeMap;

use crate::core::editor_event::InspectorFieldChange;
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use crate::ui::binding::{
    AnimationCommand, AssetCommand, DockCommand, DraftCommand, EditorUiBinding,
    EditorUiBindingPayload, EditorUiEventKind, SelectionCommand, ViewportCommand, WelcomeCommand,
};
use zircon_runtime::ui::binding::{UiBindingCall, UiBindingValue};

const DYNAMIC_DOCUMENT_TAB_INSTANCE_ID: &str = "$document_tab_instance";
const DYNAMIC_MAIN_PAGE_ID: &str = "$main_page_id";

pub(crate) fn builtin_template_bindings() -> BTreeMap<String, EditorUiBinding> {
    BTreeMap::from([
        (
            "WorkbenchMenuBar/OpenProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        ),
        (
            "WorkbenchMenuBar/SaveProject".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("SaveProject"),
            ),
        ),
        (
            "WorkbenchMenuBar/ResetLayout".to_string(),
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "ResetLayout",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("ResetLayout"),
            ),
        ),
        (
            "ActivityRail/ProjectToggle".to_string(),
            EditorUiBinding::new(
                "ActivityRail",
                "ProjectToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
                    slot: "left_top".to_string(),
                    instance_id: "editor.project#1".to_string(),
                }),
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
            "ViewportToolbar/SetTool".to_string(),
            EditorUiBinding::new(
                "ViewportToolbar",
                "SetTool",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
                    SceneViewportTool::Drag,
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
                EditorUiBindingPayload::menu_action("DeleteSelected"),
            ),
        ),
        (
            "PaneSurface/TriggerAction".to_string(),
            EditorUiBinding::new(
                "PaneSurface",
                "TriggerAction",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        ),
        (
            "ConsolePaneBody/FocusConsole".to_string(),
            EditorUiBinding::new(
                "ConsolePaneBody",
                "FocusConsole",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::FocusView {
                    instance_id: "editor.console#1".to_string(),
                }),
            ),
        ),
        (
            "InspectorPaneBody/ApplyDraft".to_string(),
            EditorUiBinding::new(
                "InspectorPaneBody",
                "ApplyDraft",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                    subject_path: "entity://selected".to_string(),
                    field_id: "name".to_string(),
                    value: UiBindingValue::string(String::new()),
                }),
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
            "AnimationSequencePaneBody/ScrubTimeline".to_string(),
            EditorUiBinding::new(
                "AnimationSequencePaneBody",
                "ScrubTimeline",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::ScrubTimeline {
                    frame: 0,
                }),
            ),
        ),
        (
            "AnimationGraphPaneBody/AddNode".to_string(),
            EditorUiBinding::new(
                "AnimationGraphPaneBody",
                "AddNode",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::AddGraphNode {
                    graph_path: "animation://selected/graph".to_string(),
                    node_id: "new_state".to_string(),
                    node_kind: "State".to_string(),
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
        showcase_binding_entry(
            "UiComponentShowcase/ShowVisualCategory",
            "ShowVisualCategory",
            EditorUiEventKind::Click,
            "SelectCategory.Visual",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ShowInputCategory",
            "ShowInputCategory",
            EditorUiEventKind::Click,
            "SelectCategory.Input",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ShowDataCategory",
            "ShowDataCategory",
            EditorUiEventKind::Click,
            "SelectCategory.Collections",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ButtonCommit",
            "ButtonDemo",
            EditorUiEventKind::Click,
            "Commit.Button",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/IconButtonCommit",
            "IconButtonDemo",
            EditorUiEventKind::Click,
            "Commit.IconButton",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ToggleButtonChanged",
            "ToggleButtonDemo",
            EditorUiEventKind::Toggle,
            "Change.ToggleButton",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/CheckboxChanged",
            "CheckboxDemo",
            EditorUiEventKind::Toggle,
            "Change.Checkbox",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/RadioChanged",
            "RadioDemo",
            EditorUiEventKind::Change,
            "Change.Radio",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/SegmentedControlChanged",
            "SegmentedControlDemo",
            EditorUiEventKind::Change,
            "SelectOption.SegmentedControl",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/InputFieldChanged",
            "InputFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.InputField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/InputFieldCommitted",
            "InputFieldDemo",
            EditorUiEventKind::Submit,
            "Commit.InputField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/TextFieldChanged",
            "TextFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.TextField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/TextFieldCommitted",
            "TextFieldDemo",
            EditorUiEventKind::Submit,
            "Commit.TextField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldDragBegin",
            "NumberFieldDemo",
            EditorUiEventKind::DragBegin,
            "BeginDrag.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldDragUpdate",
            "NumberFieldDemo",
            EditorUiEventKind::DragUpdate,
            "DragDelta.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldLargeDragUpdate",
            "NumberFieldDemo",
            EditorUiEventKind::DragUpdate,
            "LargeDragDelta.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldDragEnd",
            "NumberFieldDemo",
            EditorUiEventKind::DragEnd,
            "EndDrag.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldChanged",
            "NumberFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/NumberFieldCommitted",
            "NumberFieldDemo",
            EditorUiEventKind::Submit,
            "Commit.NumberField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/RangeFieldDragUpdate",
            "RangeFieldDemo",
            EditorUiEventKind::DragUpdate,
            "DragDelta.RangeField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/RangeFieldChanged",
            "RangeFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.RangeField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/RangeFieldCommitted",
            "RangeFieldDemo",
            EditorUiEventKind::Submit,
            "Commit.RangeField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/DropdownChanged",
            "DropdownDemo",
            EditorUiEventKind::Change,
            "SelectOption.Dropdown",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/DropdownOpenPopup",
            "DropdownDemo",
            EditorUiEventKind::Click,
            "OpenPopup.Dropdown",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/DropdownClosePopup",
            "DropdownDemo",
            EditorUiEventKind::Click,
            "ClosePopup.Dropdown",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ComboBoxChanged",
            "ComboBoxDemo",
            EditorUiEventKind::Change,
            "SelectOption.ComboBox",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ComboBoxOpenPopup",
            "ComboBoxDemo",
            EditorUiEventKind::Click,
            "OpenPopup.ComboBox",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ComboBoxClosePopup",
            "ComboBoxDemo",
            EditorUiEventKind::Click,
            "ClosePopup.ComboBox",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/EnumFieldChanged",
            "EnumFieldDemo",
            EditorUiEventKind::Change,
            "SelectOption.EnumField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/EnumFieldOpenPopup",
            "EnumFieldDemo",
            EditorUiEventKind::Click,
            "OpenPopup.EnumField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/EnumFieldClosePopup",
            "EnumFieldDemo",
            EditorUiEventKind::Click,
            "ClosePopup.EnumField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/FlagsFieldChanged",
            "FlagsFieldDemo",
            EditorUiEventKind::Change,
            "SelectOption.FlagsField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/FlagsFieldOpenPopup",
            "FlagsFieldDemo",
            EditorUiEventKind::Click,
            "OpenPopup.FlagsField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/FlagsFieldClosePopup",
            "FlagsFieldDemo",
            EditorUiEventKind::Click,
            "ClosePopup.FlagsField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/SearchSelectChanged",
            "SearchSelectDemo",
            EditorUiEventKind::Change,
            "SelectOption.SearchSelect",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/SearchSelectOpenPopup",
            "SearchSelectDemo",
            EditorUiEventKind::Click,
            "OpenPopup.SearchSelect",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/SearchSelectClosePopup",
            "SearchSelectDemo",
            EditorUiEventKind::Click,
            "ClosePopup.SearchSelect",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/AssetFieldDropped",
            "AssetFieldDemo",
            EditorUiEventKind::Drop,
            "DropReference.AssetField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/AssetFieldClear",
            "AssetFieldDemo",
            EditorUiEventKind::Click,
            "ClearReference.AssetField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/AssetFieldLocate",
            "AssetFieldDemo",
            EditorUiEventKind::Click,
            "LocateReference.AssetField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/AssetFieldOpen",
            "AssetFieldDemo",
            EditorUiEventKind::Click,
            "OpenReference.AssetField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/InstanceFieldDropped",
            "InstanceFieldDemo",
            EditorUiEventKind::Drop,
            "DropReference.InstanceField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ObjectFieldDropped",
            "ObjectFieldDemo",
            EditorUiEventKind::Drop,
            "DropReference.ObjectField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/GroupToggled",
            "GroupDemo",
            EditorUiEventKind::Toggle,
            "ToggleExpanded.Group",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/FoldoutToggled",
            "FoldoutDemo",
            EditorUiEventKind::Toggle,
            "ToggleExpanded.Foldout",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ArrayFieldChanged",
            "ArrayFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.ArrayField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ArrayFieldAddElement",
            "ArrayFieldDemo",
            EditorUiEventKind::Submit,
            "AddElement.ArrayField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ArrayFieldSetElement",
            "ArrayFieldDemo",
            EditorUiEventKind::Submit,
            "SetElement.ArrayField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ArrayFieldRemoveElement",
            "ArrayFieldDemo",
            EditorUiEventKind::Submit,
            "RemoveElement.ArrayField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ArrayFieldMoveElement",
            "ArrayFieldDemo",
            EditorUiEventKind::DragEnd,
            "MoveElement.ArrayField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/MapFieldChanged",
            "MapFieldDemo",
            EditorUiEventKind::Change,
            "ValueChanged.MapField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/MapFieldAddEntry",
            "MapFieldDemo",
            EditorUiEventKind::Submit,
            "AddMapEntry.MapField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/MapFieldSetEntry",
            "MapFieldDemo",
            EditorUiEventKind::Submit,
            "SetMapEntry.MapField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/MapFieldRemoveEntry",
            "MapFieldDemo",
            EditorUiEventKind::Submit,
            "RemoveMapEntry.MapField",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ListRowClicked",
            "ListRowDemo",
            EditorUiEventKind::Click,
            "Select.ListRow",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/TreeRowToggled",
            "TreeRowDemo",
            EditorUiEventKind::Toggle,
            "ToggleExpanded.TreeRow",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ContextActionMenuChanged",
            "ContextActionMenuDemo",
            EditorUiEventKind::Change,
            "SelectOption.ContextActionMenu",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ContextActionMenuOpenPopup",
            "ContextActionMenuDemo",
            EditorUiEventKind::Click,
            "OpenPopup.ContextActionMenu",
        ),
        showcase_binding_entry(
            "UiComponentShowcase/ContextActionMenuClosePopup",
            "ContextActionMenuDemo",
            EditorUiEventKind::Click,
            "ClosePopup.ContextActionMenu",
        ),
    ])
}

fn showcase_binding_entry(
    binding_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
    demo_action: &str,
) -> (String, EditorUiBinding) {
    (
        binding_id.to_string(),
        EditorUiBinding::new(
            "UiComponentShowcase",
            control_id,
            event_kind,
            EditorUiBindingPayload::Custom(
                UiBindingCall::new("UiComponentShowcase")
                    .with_argument(UiBindingValue::string(demo_action))
                    .with_argument(UiBindingValue::string(control_id)),
            ),
        ),
    )
}
