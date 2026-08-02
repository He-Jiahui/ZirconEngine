use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::AssetManager;
use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::watch::AssetChange;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime::core::framework::channel::ChannelReceiver;
use zircon_runtime::core::manager::{ManagerResolver, ManagerServiceHandle};
use zircon_runtime::core::resource::ResourceEventReceiver;
use zircon_runtime::scene::{NodeId, Scene, WorldInspectionHierarchyRow};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceLocator,
};
use zircon_runtime_interface::ui::{
    binding::UiBindingValue,
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragPayload,
        UiDragPayloadKind, UiDragSourceMetadata, UiValue,
    },
    design_tokens::EditorDesignTokens,
    dispatch::UiPointerComponentEvent,
    layout::UiFrame,
    layout::UiPoint,
    layout::UiSize,
};

use crate::core::editing::paths::canonical_model_source_path;
use crate::core::editor_event::EditorViewportEvent;
use crate::core::gateway::SharedEditorRuntimeGateway;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::play::NativePluginBridgeActivation;
use crate::core::settings::editor_design_tokens_at_startup;
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::host::EditorHostEventController;
use crate::ui::host::EditorManager;
use crate::ui::host::editor_asset_manager::{
    EditorAssetChange, EditorAssetChangeSubscription,
    EditorAssetManager as EditorAssetManagerContract,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::resource_access::resolve_ready_handle;
use crate::ui::retained_host::ui_perf::UiPerfScenario;
use crate::ui::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
use crate::ui::v2_design_tokens::install_editor_v2_design_tokens;
use crate::ui::workbench::autolayout::{
    ShellRegionId, ShellSizePx, WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
use crate::ui::workbench::layout::{ActivityDrawerMode, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::snapshot::{SceneEntries, ViewContentKind};
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};
use crate::ui::workbench::state::EditorState;

use super::activity_rail_pointer::{
    HostActivityRailPointerBridge, HostActivityRailPointerSide,
    build_host_activity_rail_pointer_layout_with_workbench_layout_frames,
};
use super::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerLayout, AssetFolderTreePointerBridge,
    AssetFolderTreePointerLayout, AssetListPointerState, AssetPointerContentRoute,
    AssetPointerReferenceRoute, AssetReferenceListPointerBridge, AssetReferenceListPointerLayout,
};
use super::callback_dispatch;
use super::detail_pointer::{
    asset_details_scroll_layout, console_content_extent, console_scroll_layout,
    inspector_scroll_layout,
};
use super::document_tab_pointer::{
    HostDocumentTabPointerBridge,
    build_host_document_tab_pointer_layout_with_workbench_layout_frames,
};
use super::drawer_header_pointer::{
    HostDrawerHeaderPointerBridge,
    build_host_drawer_header_pointer_layout_with_workbench_layout_frames,
};
use super::drawer_resize::dispatch_resize_to_group;
use super::event_bridge::UiHostEventEffects;
use super::floating_window_projection::FloatingWindowProjectionBundle;
use super::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerState,
};
use super::host_page_pointer::{HostPagePointerBridge, build_host_page_pointer_layout};
use super::menu_pointer::{HostMenuPointerBridge, HostMenuPointerLayout, HostMenuPointerState};
use super::scroll_surface_host::ScrollSurfaceHostState;
use super::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use super::tab_drag::host_shell_pointer_route_group_key;
use super::ui::apply_presentation;
use super::viewport::RetainedViewportController;
use super::viewport_toolbar_pointer::ViewportToolbarPointerBridge;
use super::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
    WelcomeRecentPointerState,
};
use super::{
    FrameRect, UiHostWindow, WorkbenchContextMenuRequestData, apply_host_appearance_from_tokens,
};

mod asset_content_pointer;
mod asset_drag_payload;
mod asset_reference_pointer;
mod asset_surface_pointer_state;
mod asset_tree_pointer;
mod assets;
pub(crate) mod backend_refresh;
mod build_export_actions;
mod build_export_projection;
mod build_export_wizard_session;
mod callback_wiring;
mod close_prompt;
mod command_palette_actions;
mod component_showcase_runtime;
mod detail_scroll_pointer;
mod helpers;
mod hierarchy_filter;
mod hierarchy_pointer;
pub(in crate::ui::retained_host) mod hierarchy_rename;
mod host_lifecycle;
mod inspector;
mod invalidation;
mod job_progress;
mod menu_pointer;
mod module_plugin_actions;
mod module_plugin_projection;
mod native_keyboard_actions;
mod native_window_close;
mod native_windows;
mod pane_payload_visibility;
mod pane_surface_actions;
mod pointer_layout;
mod product_frame_diagnostics;
mod profiling;
mod reference_drop_payload;
mod runtime_diagnostics_visibility;
mod scene_picker_actions;
mod scene_picker_session;
#[cfg(test)]
mod scene_picker_session_tests;
mod showcase_event_inputs;
mod startup;
#[cfg(test)]
mod tests;
mod ui_asset_editor;
mod ui_asset_editor_detail_events;
mod ui_asset_editor_detail_routes;
mod viewport;
mod viewport_image_redraw;
mod viewport_toolbar_projection;
mod welcome_recent_pointer;
mod welcome_session;
mod workbench_context_menu;
mod workbench_notifications;
mod workbench_pointer;
mod workbench_snapshot_access;
mod workspace_docking;
use super::run_config::EditorHostRunConfig;
use callback_wiring::wire_callbacks;
pub(super) use helpers::{
    asset_surface_visible, compute_window_menu_popup_height,
    derive_animation_assets_from_model_source, resolve_callback_source_window_id,
    shell_region_group_key, stage_model_source, viewport_size_from_frame,
};
pub(crate) use invalidation::HostInvalidationMask;
use invalidation::HostInvalidationRoot;
pub(crate) use native_windows::NativeWindowPresenterStore;
#[cfg(test)]
pub(crate) use native_windows::{
    NativeFloatingWindowTarget, collect_native_floating_window_targets,
    configure_native_floating_window_presentation,
};
use product_frame_diagnostics::editor_product_frame_diagnostics;
pub(crate) use startup::build_startup_state;

pub fn run_editor(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
) -> Result<(), Box<dyn Error>> {
    run_editor_with_config(core, runtime_gateway, EditorHostRunConfig::new())
}

pub fn run_editor_with_startup_request(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<(), Box<dyn Error>> {
    run_editor_with_config(
        core,
        runtime_gateway,
        EditorHostRunConfig::new().with_startup_request(startup_request),
    )
}

pub fn run_editor_with_config(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
    config: EditorHostRunConfig,
) -> Result<(), Box<dyn Error>> {
    let design_tokens = editor_design_tokens_at_startup();
    apply_host_appearance_from_tokens(&design_tokens);
    install_editor_v2_design_tokens(&design_tokens);
    let exit_after_first_presented_frame = config.exit_after_first_presented_frame();
    let (
        startup_request,
        prepared_project,
        first_presented_frame_capture_path,
        editor_plugin_registrations,
    ) = config.into_parts();
    let product_frame_evidence_requested = first_presented_frame_capture_path.is_some();
    let ui = UiHostWindow::new()?;
    ui.set_exit_after_first_presented_frame(exit_after_first_presented_frame);
    ui.set_first_presented_frame_capture_path(first_presented_frame_capture_path);
    let mut retained_host = RetainedEditorHost::new(
        core,
        runtime_gateway,
        ui.clone_strong(),
        startup_request,
        prepared_project,
    )?;
    for registration in editor_plugin_registrations {
        retained_host
            .runtime
            .register_editor_plugin_registration(registration)?;
    }
    retained_host.sync_plugin_template_documents_if_changed()?;
    let host = Rc::new(RefCell::new(retained_host));
    wire_callbacks(&ui, &host);
    let host_weak = Rc::downgrade(&host);
    ui.window().on_close_requested(move || {
        if let Some(host) = host_weak.upgrade() {
            host.borrow_mut().native_main_window_close_requested()
        } else {
            crate::ui::retained_host::primitives::CloseRequestResponse::KeepWindowShown
        }
    });
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));

    host.borrow_mut().refresh_ui();
    if product_frame_evidence_requested {
        let diagnostic =
            editor_product_frame_diagnostics(&host.borrow().runtime.editor_snapshot())?;
        zircon_runtime::diagnostic_log::write_log("editor_host_window", &diagnostic);
    }
    let run_result = ui.run();
    if let Some(error) = ui.take_fatal_failure() {
        return Err(error.into());
    }
    run_result?;
    if let Some(error) = ui.take_first_presented_frame_capture_error() {
        return Err(std::io::Error::other(error).into());
    }
    Ok(())
}

struct RetainedEditorHost {
    ui: UiHostWindow,
    self_handle: Option<Weak<RefCell<RetainedEditorHost>>>,
    runtime: EditorHostEventController,
    editor_manager: Arc<EditorManager>,
    module_plugin_projection_cache:
        RefCell<module_plugin_projection::ModulePluginPaneProjectionCache>,
    #[cfg(feature = "profiling")]
    runtime_gateway: SharedEditorRuntimeGateway,
    module_plugin_live_host_backend: Box<dyn module_plugin_actions::ModulePluginLiveHostBackend>,
    desktop_export_reports: BTreeMap<String, build_export_actions::DesktopExportExecutionSummary>,
    desktop_export_jobs: build_export_actions::DesktopExportJobQueue,
    desktop_export_output_overrides: BTreeMap<String, std::path::PathBuf>,
    desktop_export_wizard_sessions: build_export_wizard_session::DesktopExportWizardSessions,
    viewport: RetainedViewportController,
    asset_manager: ManagerServiceHandle<dyn AssetManager>,
    editor_asset_manager: ManagerServiceHandle<dyn EditorAssetManagerContract>,
    resource_manager_resolver: ManagerResolver,
    resource_manager: ManagerServiceHandle<dyn ResourceManager>,
    asset_change_events: ChannelReceiver<AssetChange>,
    editor_asset_change_events: EditorAssetChangeSubscription,
    resource_change_events: ResourceEventReceiver,
    asset_refresh_queue_age: assets::AssetRefreshQueueAgeState,
    startup_session: EditorStartupSessionDocument,
    welcome_project_probe: welcome_session::WelcomeProjectProbeState,
    viewport_size: UVec2,
    viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge,
    builtin_template_runtime: Arc<EditorUiHostRuntime>,
    plugin_template_generation: u64,
    plugin_template_capabilities: Vec<String>,
    template_bridge: callback_dispatch::BuiltinHostWindowTemplateBridge,
    workbench_window_bridge: callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    floating_window_source_bridge: callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge,
    viewport_toolbar_bridge: callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge,
    asset_surface_bridge: Option<callback_dispatch::BuiltinAssetSurfaceTemplateBridge>,
    welcome_surface_bridge: Option<callback_dispatch::BuiltinWelcomeSurfaceTemplateBridge>,
    inspector_surface_bridge: callback_dispatch::BuiltinInspectorSurfaceTemplateBridge,
    pane_surface_bridge: callback_dispatch::BuiltinPaneSurfaceTemplateBridge,
    component_showcase_runtime: EditorUiHostRuntime,
    component_showcase_runtime_loaded: bool,
    shell_pointer_bridge: HostShellPointerBridge,
    activity_rail_pointer_bridge: HostActivityRailPointerBridge,
    host_page_pointer_bridge: HostPagePointerBridge,
    document_tab_pointer_bridge: HostDocumentTabPointerBridge,
    drawer_header_pointer_bridge: HostDrawerHeaderPointerBridge,
    menu_pointer_bridge: HostMenuPointerBridge,
    menu_pointer_state: HostMenuPointerState,
    menu_pointer_layout: HostMenuPointerLayout,
    welcome_recent_pointer_bridge: WelcomeRecentPointerBridge,
    welcome_recent_pointer_state: WelcomeRecentPointerState,
    welcome_recent_pointer_size: UiSize,
    hierarchy_pointer_bridge: HierarchyPointerBridge,
    hierarchy_pointer_state: HierarchyPointerState,
    hierarchy_pointer_size: UiSize,
    hierarchy_scene_entries: Arc<[WorldInspectionHierarchyRow]>,
    hierarchy_filter_query: String,
    console_scroll_surface: ScrollSurfaceHostState,
    inspector_scroll_surface: ScrollSurfaceHostState,
    browser_asset_details_scroll_surface: ScrollSurfaceHostState,
    activity_asset_pointer: AssetSurfacePointerState,
    browser_asset_pointer: AssetSurfacePointerState,
    active_asset_drag_payload: Option<UiDragPayload>,
    active_scene_drag_payload: Option<UiDragPayload>,
    active_hierarchy_drag_node_ids: Vec<NodeId>,
    last_hierarchy_rename_click: Option<hierarchy_rename::HierarchyRenameClick>,
    active_object_drag_payload: Option<UiDragPayload>,
    native_window_presenters: NativeWindowPresenterStore,
    floating_window_projection_bundle: FloatingWindowProjectionBundle,
    callback_source_window: Option<MainPageId>,
    last_focused_callback_window: Option<MainPageId>,
    active_layout_preset: Option<String>,
    shell_size: ShellSizePx,
    shell_scale_factor: f32,
    chrome_metrics: WorkbenchChromeMetrics,
    shell_geometry: Option<WorkbenchShellGeometry>,
    shell_token_region_defaults: Option<(EditorDesignTokens, BTreeMap<ShellRegionId, f32>)>,
    transient_region_preferred: BTreeMap<ShellRegionId, f32>,
    active_drawer_resize: Option<ActiveDrawerResize>,
    pending_close_prompt: Option<close_prompt::PendingClosePrompt>,
    scene_picker_session: Option<scene_picker_session::ScenePickerSession>,
    invalidation: HostInvalidationRoot,
    pending_ui_perf_scenario: Option<UiPerfScenario>,
    presentation_dirty: bool,
    layout_dirty: bool,
    window_metrics_dirty: bool,
    render_dirty: bool,
}

impl RetainedEditorHost {
    fn sync_plugin_template_documents_if_changed(
        &mut self,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let (generation, enabled_capabilities) = self.runtime.plugin_template_revision();
        if self.plugin_template_generation == generation
            && self.plugin_template_capabilities == enabled_capabilities
        {
            return Ok(());
        }

        let (generation, enabled_capabilities, templates_by_owner) =
            self.runtime.enabled_plugin_template_descriptors();
        self.builtin_template_runtime
            .sync_plugin_v2_template_descriptor_sets(&templates_by_owner)?;

        // The runtime publishes the complete owner set atomically before this revision advances.
        self.plugin_template_generation = generation;
        self.plugin_template_capabilities = enabled_capabilities;
        self.mark_presentation_dirty();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ActiveDrawerResize {
    region: ShellRegionId,
    start_x: f32,
    start_y: f32,
    base_preferred: f32,
}

struct AssetSurfacePointerState {
    snapshot: Option<Arc<crate::ui::workbench::snapshot::AssetWorkspaceSnapshot>>,
    tree_bridge: AssetFolderTreePointerBridge,
    tree_state: AssetListPointerState,
    tree_size: UiSize,
    content_bridge: AssetContentListPointerBridge,
    content_state: AssetListPointerState,
    content_size: UiSize,
    references: AssetReferenceListSurfacePointerState,
    used_by: AssetReferenceListSurfacePointerState,
}

struct AssetReferenceListSurfacePointerState {
    bridge: AssetReferenceListPointerBridge,
    state: AssetListPointerState,
    size: UiSize,
}
