use super::super::super::super::*;

pub(super) struct StartupInteractionState {
    pub(super) viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge,
    pub(super) viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge,
    pub(super) shell_pointer_bridge: HostShellPointerBridge,
    pub(super) activity_rail_pointer_bridge: HostActivityRailPointerBridge,
    pub(super) host_page_pointer_bridge: HostPagePointerBridge,
    pub(super) document_tab_pointer_bridge: HostDocumentTabPointerBridge,
    pub(super) drawer_header_pointer_bridge: HostDrawerHeaderPointerBridge,
    pub(super) menu_pointer_bridge: HostMenuPointerBridge,
    pub(super) menu_pointer_state: HostMenuPointerState,
    pub(super) menu_pointer_layout: HostMenuPointerLayout,
    pub(super) welcome_recent_pointer_bridge: WelcomeRecentPointerBridge,
    pub(super) welcome_recent_pointer_state: WelcomeRecentPointerState,
    pub(super) welcome_recent_pointer_size: UiSize,
    pub(super) hierarchy_pointer_bridge: HierarchyPointerBridge,
    pub(super) hierarchy_pointer_state: HierarchyPointerState,
    pub(super) hierarchy_pointer_size: UiSize,
    pub(super) hierarchy_scene_entries: Arc<[WorldInspectionHierarchyRow]>,
    pub(super) console_scroll_surface: ScrollSurfaceHostState,
    pub(super) inspector_scroll_surface: ScrollSurfaceHostState,
    pub(super) browser_asset_details_scroll_surface: ScrollSurfaceHostState,
    pub(super) activity_asset_pointer: AssetSurfacePointerState,
    pub(super) browser_asset_pointer: AssetSurfacePointerState,
}

impl StartupInteractionState {
    pub(super) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge::new(
                UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32),
            ),
            viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge::new(),
            shell_pointer_bridge: HostShellPointerBridge::new(),
            activity_rail_pointer_bridge: HostActivityRailPointerBridge::new(),
            host_page_pointer_bridge: HostPagePointerBridge::new(),
            document_tab_pointer_bridge: HostDocumentTabPointerBridge::new(),
            drawer_header_pointer_bridge: HostDrawerHeaderPointerBridge::new(),
            menu_pointer_bridge: HostMenuPointerBridge::new(),
            menu_pointer_state: HostMenuPointerState::default(),
            menu_pointer_layout: HostMenuPointerLayout::default(),
            welcome_recent_pointer_bridge: WelcomeRecentPointerBridge::new(),
            welcome_recent_pointer_state: WelcomeRecentPointerState::default(),
            welcome_recent_pointer_size: UiSize::new(0.0, 0.0),
            hierarchy_pointer_bridge: HierarchyPointerBridge::new(),
            hierarchy_pointer_state: HierarchyPointerState::default(),
            hierarchy_pointer_size: UiSize::new(0.0, 0.0),
            hierarchy_scene_entries: Arc::from(Vec::<WorldInspectionHierarchyRow>::new()),
            console_scroll_surface: ScrollSurfaceHostState::new(
                "zircon.editor.console.pointer",
                "editor.console",
            ),
            inspector_scroll_surface: ScrollSurfaceHostState::new(
                "zircon.editor.inspector.pointer",
                "editor.inspector",
            ),
            browser_asset_details_scroll_surface: ScrollSurfaceHostState::new(
                "zircon.editor.asset_details.pointer",
                "editor.asset_details",
            ),
            activity_asset_pointer: AssetSurfacePointerState::new(),
            browser_asset_pointer: AssetSurfacePointerState::new(),
        }
    }
}
