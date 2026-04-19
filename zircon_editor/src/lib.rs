//! Editor host UI built on Slint, with viewport frames coming from core graphics.

mod core;
mod scene;
mod ui;

use zircon_runtime::engine_module::{EngineModule, ModuleDescriptor};

pub(crate) use core::editing::{command, history, intent, paths};
pub(crate) use core::host::module;
pub(crate) use ui::workbench::{autolayout, layout, project, snapshot, view};

pub use core::editing::intent::EditorIntent;
pub use core::editing::state::EditorState;
pub use core::editing::ui_asset::{
    apply_external_effects_to_asset_sources, UiAssetDragDropPolicy, UiAssetEditorCommand,
    UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorExternalEffect, UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
    UiAssetEditorPreviewCanvasSlotTarget, UiAssetEditorReplayResult, UiAssetEditorReplayWorkspace,
    UiAssetEditorReplayWorkspaceResult, UiAssetEditorSession, UiAssetEditorSessionError,
    UiAssetEditorSourceCursorSnapshot, UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind,
    UiAssetEditorUndoExternalEffects, UiAssetEditorUndoReplayRecord, UiAssetEditorUndoStack,
    UiAssetEditorUndoTransition, UiAssetPreviewHost, UiAssetSourceBuffer,
};
pub use core::editor_event::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode,
    EditorDraftEvent, EditorEvent, EditorEventDispatcher, EditorEventEffect, EditorEventEnvelope,
    EditorEventId, EditorEventJournal, EditorEventRecord, EditorEventReplay, EditorEventResult,
    EditorEventRuntime, EditorEventSequence, EditorEventSource, EditorEventTransient,
    EditorEventUndoPolicy, EditorInspectorEvent, EditorTransientUiState, EditorViewportEvent,
};
pub use core::host::asset_editor::{
    resolve_editor_asset_manager, AssetCatalogRecord, DefaultEditorAssetManager,
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChange,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetManager, EditorAssetManagerHandle,
    EditorAssetReferenceRecord, PreviewArtifactKey, PreviewCache, PreviewScheduler, ReferenceGraph,
};
pub use core::host::manager::{EditorError, EditorManager, NativeWindowHostState};
pub use core::host::module::{
    module_descriptor, EditorHostDriver, EDITOR_ASSET_MANAGER_NAME, EDITOR_HOST_DRIVER_NAME,
    EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME,
};
pub use scene::viewport::{
    GizmoAxis, GridMode, SceneViewportSettings, SceneViewportTool, TransformSpace,
    ViewOrientation, ViewportFeedback, ViewportInput, ViewportState,
};
pub use ui::binding_dispatch::{
    apply_draft_binding, apply_inspector_binding, apply_selection_binding, apply_viewport_binding,
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_draft_binding, dispatch_inspector_binding, dispatch_selection_binding,
    dispatch_viewport_binding, dispatch_welcome_binding, AnimationHostEvent, AssetHostEvent,
    DraftHostEvent, EditorBindingDispatchError, InspectorBindingBatch, SelectionHostEvent,
    WelcomeHostEvent,
};
pub use ui::slint_host::run_editor;
pub use ui::slint_host::tab_drag::{resolve_workbench_drag_target_group, WorkbenchDragTargetGroup};
pub use ui::template_runtime::{
    EditorUiCompatibilityHarness, EditorUiCompatibilitySnapshot, EditorUiHostRuntime,
    EditorUiHostRuntimeError, SlintUiBindingProjection, SlintUiHostAdapter,
    SlintUiHostBindingProjection, SlintUiHostComponentKind, SlintUiHostModel, SlintUiHostNodeModel,
    SlintUiHostNodeProjection, SlintUiHostProjection, SlintUiHostRouteProjection, SlintUiHostValue,
    SlintUiNodeProjection, SlintUiProjection,
};
pub use ui::workbench::autolayout::{
    compute_workbench_shell_geometry, default_constraints_for_content, default_region_constraints,
    solve_axis_constraints, AxisConstraint, AxisConstraintOverride, PaneConstraintOverride,
    PaneConstraints, ResolvedAxisConstraint, ShellFrame, ShellRegionId, ShellSizePx, StretchMode,
    WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
pub use ui::workbench::event::{
    dispatch_workbench_binding, menu_action_binding, WorkbenchHostEvent, WorkbenchHostEventError,
};
pub use ui::workbench::fixture::{
    default_preview_fixture, PreviewEditorData, PreviewFixture, PreviewGizmoAxis, PreviewInspector,
    PreviewSceneEntry,
};
pub use ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DockEdge, DocumentNode,
    DragPayload, DropTarget, FloatingWindowLayout, HitTarget, LayoutCommand, LayoutDiff,
    LayoutManager, LayoutNormalizationReport, MainHostPageLayout, MainPageId, RestorePolicy,
    SplitAxis, SplitPlacement, TabInsertionAnchor, TabInsertionSide, TabStackLayout,
    WorkbenchLayout, WorkspaceTarget,
};
pub use ui::workbench::model::{
    BreadcrumbModel, DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel,
    FloatingWindowModel, HostPageTabModel, MainHostStripModel, MainHostStripViewModel, MenuAction,
    MenuBarModel, MenuItemModel, MenuModel, PaneActionModel, PaneEmptyStateModel, PaneTabModel,
    StatusBarModel, ToolWindowStackModel, WorkbenchViewModel,
};
pub use ui::workbench::project::{EditorProjectDocument, ProjectEditorWorkspace};
pub use ui::workbench::reflection::{
    activity_descriptors_from_views, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
pub use ui::workbench::snapshot::{
    ActivityDrawerSnapshot, DocumentWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot,
    FloatingWindowSnapshot as WorkbenchFloatingWindowSnapshot, InspectorSnapshot, MainPageSnapshot,
    SceneEntry, ViewContentKind, ViewTabSnapshot, WorkbenchSnapshot,
};
pub use ui::workbench::startup::{
    EditorSessionMode, EditorStartupSessionDocument, NewProjectDraft, NewProjectFormSnapshot,
    NewProjectTemplate, RecentProjectEntry, RecentProjectItemSnapshot, RecentProjectValidation,
    WelcomePaneSnapshot,
};
pub use ui::workbench::view::{
    DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance,
    ViewInstanceId, ViewKind, ViewRegistry,
};
pub use ui::{
    inspector_field_control_id, ui_asset_editor_window_descriptor, ActivityDrawerSlotPreference,
    ActivityViewDescriptor, ActivityWindowDescriptor, AssetCommand, DockCommand, DraftCommand,
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorComponentCatalog,
    EditorComponentDescriptor, EditorDrawerReflectionModel, EditorFloatingWindowReflectionModel,
    EditorHostPageReflectionModel, EditorMenuItemReflectionModel, EditorTemplateAdapter,
    EditorTemplateError, EditorTemplateRegistry, EditorUiBinding, EditorUiBindingPayload,
    EditorUiControlService, EditorUiError, EditorUiEventKind, EditorUiReflectionAdapter,
    EditorUiRouter, EditorWorkbenchReflectionModel, InspectorFieldChange, SelectionCommand,
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetPreviewPreset,
    UiDesignerSelectionModel, UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    ViewportCommand, WelcomeCommand, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE, UI_ASSET_EDITOR_WINDOW_ID,
};
pub use zircon_runtime::ui::binding::UiBindingValue;
pub use zircon_runtime::ui::layout::UiSize;

#[derive(Clone, Copy, Debug, Default)]
pub struct EditorModule;

impl EngineModule for EditorModule {
    fn module_name(&self) -> &'static str {
        EDITOR_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Slint-based editor host and tooling shell"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
