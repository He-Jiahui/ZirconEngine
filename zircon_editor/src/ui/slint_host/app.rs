use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Duration;

use slint::{Timer, TimerMode};
use zircon_runtime::asset::pipeline::manager::AssetManager;
use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::watch::AssetChange;
use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::{ChannelReceiver, CoreHandle};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceEvent, ResourceHandle, ResourceLocator,
};
use zircon_runtime_interface::ui::{
    binding::UiBindingValue,
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragPayload,
        UiDragPayloadKind, UiDragSourceMetadata, UiValue,
    },
    layout::UiFrame,
    layout::UiPoint,
    layout::UiSize,
};

use crate::core::editing::paths::canonical_model_source_path;
use crate::core::editor_event::{EditorEventRuntime, EditorViewportEvent};
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::host::editor_asset_manager::{
    EditorAssetChange, EditorAssetManager as EditorAssetManagerContract,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::resource_access::resolve_ready_handle;
use crate::ui::host::EditorManager;
use crate::ui::host::SharedEditorRuntimeClient;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellRegionId, ShellSizePx, WorkbenchChromeMetrics,
    WorkbenchShellGeometry,
};
use crate::ui::workbench::layout::{ActivityDrawerMode, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::snapshot::{SceneEntry, ViewContentKind};
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};
use crate::ui::workbench::state::EditorState;

use super::activity_rail_pointer::{
    build_host_activity_rail_pointer_layout, HostActivityRailPointerBridge,
    HostActivityRailPointerSide,
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
    build_host_document_tab_pointer_layout, HostDocumentTabPointerBridge,
};
use super::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout, HostDrawerHeaderPointerBridge,
};
use super::drawer_resize::dispatch_resize_to_group;
use super::event_bridge::SlintDispatchEffects;
use super::floating_window_projection::FloatingWindowProjectionBundle;
use super::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerState,
};
use super::host_page_pointer::{build_host_page_pointer_layout, HostPagePointerBridge};
use super::menu_pointer::{HostMenuPointerBridge, HostMenuPointerLayout, HostMenuPointerState};
use super::scroll_surface_host::ScrollSurfaceHostState;
use super::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use super::tab_drag::host_shell_pointer_route_group_key;
use super::ui::apply_presentation;
use super::viewport::SlintViewportController;
use super::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout_with_size, ViewportToolbarPointerBridge,
};
use super::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
    WelcomeRecentPointerState,
};
use super::{FrameRect, UiHostWindow};

mod asset_content_pointer;
mod asset_drag_payload;
mod asset_reference_pointer;
mod asset_surface_pointer_state;
mod asset_tree_pointer;
mod assets;
pub(crate) mod backend_refresh;
mod callback_wiring;
mod detail_scroll_pointer;
mod helpers;
mod hierarchy_pointer;
mod host_lifecycle;
mod inspector;
mod menu_pointer;
mod module_plugin_actions;
mod native_window_close;
mod native_windows;
mod pane_surface_actions;
mod pointer_layout;
mod reference_drop_payload;
mod showcase_event_inputs;
mod startup;
#[cfg(test)]
mod tests;
mod ui_asset_editor;
mod viewport;
mod welcome_recent_pointer;
mod welcome_session;
mod workbench_pointer;
mod workspace_docking;
use callback_wiring::wire_callbacks;
pub(super) use helpers::{
    asset_surface_visible, compute_window_menu_popup_height,
    derive_animation_assets_from_model_source, resolve_callback_source_window_id,
    shell_region_group_key, stage_model_source, viewport_size_from_frame,
};
#[cfg(test)]
pub(crate) use native_windows::NativeFloatingWindowTarget;
pub(crate) use native_windows::{
    collect_native_floating_window_targets, configure_native_floating_window_presentation,
    NativeWindowPresenterStore,
};
pub(super) use startup::build_startup_state;

pub fn run_editor(
    core: CoreHandle,
    runtime_client: SharedEditorRuntimeClient,
) -> Result<(), Box<dyn Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;

    let ui = UiHostWindow::new()?;
    let host = Rc::new(RefCell::new(SlintEditorHost::new(
        core,
        runtime_client,
        ui.clone_strong(),
    )?));
    wire_callbacks(&ui, &host);
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));

    host.borrow_mut().refresh_ui();

    let timer = Timer::default();
    let host_weak = Rc::downgrade(&host);
    timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        if let Some(host) = host_weak.upgrade() {
            host.borrow_mut().tick();
        }
    });

    ui.run()?;
    timer.stop();
    Ok(())
}

struct SlintEditorHost {
    ui: UiHostWindow,
    self_handle: Option<Weak<RefCell<SlintEditorHost>>>,
    runtime: EditorEventRuntime,
    runtime_client: SharedEditorRuntimeClient,
    editor_manager: Arc<EditorManager>,
    viewport: SlintViewportController,
    asset_server: Arc<dyn AssetManager>,
    editor_asset_server: Arc<dyn EditorAssetManagerContract>,
    resource_server: Arc<dyn ResourceManager>,
    asset_change_events: ChannelReceiver<AssetChange>,
    editor_asset_change_events: ChannelReceiver<EditorAssetChange>,
    resource_change_events: ChannelReceiver<ResourceEvent>,
    startup_session: EditorStartupSessionDocument,
    viewport_size: UVec2,
    viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge,
    template_bridge: callback_dispatch::BuiltinHostWindowTemplateBridge,
    floating_window_source_bridge: callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge,
    viewport_toolbar_bridge: callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge,
    asset_surface_bridge: callback_dispatch::BuiltinAssetSurfaceTemplateBridge,
    welcome_surface_bridge: callback_dispatch::BuiltinWelcomeSurfaceTemplateBridge,
    inspector_surface_bridge: callback_dispatch::BuiltinInspectorSurfaceTemplateBridge,
    pane_surface_bridge: callback_dispatch::BuiltinPaneSurfaceTemplateBridge,
    component_showcase_runtime: EditorUiHostRuntime,
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
    console_scroll_surface: ScrollSurfaceHostState,
    inspector_scroll_surface: ScrollSurfaceHostState,
    browser_asset_details_scroll_surface: ScrollSurfaceHostState,
    activity_asset_pointer: AssetSurfacePointerState,
    browser_asset_pointer: AssetSurfacePointerState,
    active_asset_drag_payload: Option<UiDragPayload>,
    active_scene_drag_payload: Option<UiDragPayload>,
    active_object_drag_payload: Option<UiDragPayload>,
    native_window_presenters: NativeWindowPresenterStore,
    floating_window_projection_bundle: FloatingWindowProjectionBundle,
    callback_source_window: Option<MainPageId>,
    last_focused_callback_window: Option<MainPageId>,
    active_layout_preset: Option<String>,
    shell_size: ShellSizePx,
    chrome_metrics: WorkbenchChromeMetrics,
    shell_geometry: Option<WorkbenchShellGeometry>,
    transient_region_preferred: BTreeMap<ShellRegionId, f32>,
    active_drawer_resize: Option<ActiveDrawerResize>,
    presentation_dirty: bool,
    layout_dirty: bool,
    window_metrics_dirty: bool,
    render_dirty: bool,
}

#[derive(Clone, Copy, Debug)]
struct ActiveDrawerResize {
    region: ShellRegionId,
    start_x: f32,
    start_y: f32,
    base_preferred: f32,
}

struct AssetSurfacePointerState {
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
