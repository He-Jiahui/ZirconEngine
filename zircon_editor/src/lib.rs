//! Editor host UI built on Slint, with viewport frames coming from core graphics.

pub mod editor_event;

mod editing;
mod host;
mod workbench;

pub(crate) use editing::{command, history, intent, paths};
pub(crate) use host::{manager, module};
pub(crate) use workbench::{autolayout, layout, project, snapshot, view};

pub use editing::intent::EditorIntent;
pub use editing::state::EditorState;
pub use editor_event::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode, EditorEvent,
    EditorEventDispatcher, EditorEventEffect, EditorEventEnvelope, EditorEventId,
    EditorEventJournal, EditorEventRecord, EditorEventReplay, EditorEventResult,
    EditorEventRuntime, EditorEventSequence, EditorEventSource, EditorEventTransient,
    EditorEventUndoPolicy, EditorInspectorEvent, EditorTransientUiState, EditorViewportEvent,
};
pub use host::binding_dispatch::{
    apply_inspector_binding, apply_selection_binding, apply_viewport_binding,
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_inspector_binding, dispatch_selection_binding, dispatch_viewport_binding,
    AnimationHostEvent, AssetHostEvent, EditorBindingDispatchError, InspectorBindingBatch,
    SelectionHostEvent,
};
pub use host::manager::{EditorError, EditorManager, EditorSessionState, WindowHostManager};
pub use host::module::{
    module_descriptor, EditorHostDriver, EDITOR_HOST_DRIVER_NAME, EDITOR_MANAGER_NAME,
    EDITOR_MODULE_NAME,
};
pub use host::slint_host::run_editor;
pub use host::viewport_texture::{ViewportTextureBridge, ViewportTextureBridgeError};
pub use workbench::event::{
    dispatch_workbench_binding, menu_action_binding, WorkbenchHostEvent, WorkbenchHostEventError,
};
pub use workbench::fixture::{
    default_preview_fixture, PreviewEditorData, PreviewFixture, PreviewGizmoAxis, PreviewInspector,
    PreviewSceneEntry,
};
pub use workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DockEdge, DocumentNode,
    DragPayload, DropTarget, FloatingWindowLayout, HitTarget, LayoutCommand, LayoutDiff,
    LayoutManager, LayoutNormalizationReport, MainHostPageLayout, MainPageId, RestorePolicy,
    SplitAxis, SplitPlacement, TabInsertionAnchor, TabInsertionSide, TabStackLayout,
    WorkbenchLayout, WorkspaceTarget,
};
pub use workbench::autolayout::{
    compute_workbench_shell_geometry, default_constraints_for_content,
    default_region_constraints, solve_axis_constraints, AxisConstraint,
    AxisConstraintOverride, PaneConstraintOverride, PaneConstraints,
    ResolvedAxisConstraint, ShellFrame, ShellRegionId, ShellSizePx, StretchMode,
    WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
pub use workbench::model::{
    BreadcrumbModel, DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel, HostPageTabModel,
    MainHostStripModel, MainHostStripViewModel, MenuAction, MenuBarModel, MenuItemModel, MenuModel,
    PaneActionModel, PaneEmptyStateModel, PaneTabModel, StatusBarModel, ToolWindowStackModel,
    WorkbenchViewModel,
};
pub use workbench::project::{EditorProjectDocument, ProjectEditorWorkspace};
pub use workbench::reflection::{
    activity_descriptors_from_views, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
pub use workbench::snapshot::{
    ActivityDrawerSnapshot, DocumentWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot,
    FloatingWindowSnapshot as WorkbenchFloatingWindowSnapshot, InspectorSnapshot, MainPageSnapshot,
    SceneEntry, ViewContentKind, ViewTabSnapshot, WorkbenchSnapshot,
};
pub use workbench::startup::{
    EditorSessionMode, EditorStartupSessionDocument, NewProjectDraft, NewProjectFormSnapshot,
    NewProjectTemplate, RecentProjectEntry, RecentProjectItemSnapshot, RecentProjectValidation,
    WelcomePaneSnapshot,
};
pub use workbench::view::{
    DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance,
    ViewInstanceId, ViewKind, ViewRegistry,
};
pub use zircon_editor_ui::InspectorFieldChange;
pub use zircon_ui::UiBindingValue;

#[cfg(test)]
mod tests;
