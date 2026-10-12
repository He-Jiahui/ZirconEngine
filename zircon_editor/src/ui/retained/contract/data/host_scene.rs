use super::{
    HostBottomDockSurfaceData, HostDocumentDockSurfaceData, HostFloatingWindowLayerData,
    HostMenuChromeData, HostPageChromeData, HostResizeLayerData, HostSideDockSurfaceData,
    HostStatusBarData, HostTabDragOverlayData, HostWindowLayoutData, HostWindowSurfaceMetricsData,
    HostWindowSurfaceOrchestrationData,
};

#[derive(Clone, Default)]
pub(crate) struct HostWindowSceneData {
    pub document_leaves: Vec<HostDocumentDockSurfaceData>,
    pub layout: HostWindowLayoutData,
    pub metrics: HostWindowSurfaceMetricsData,
    pub orchestration: HostWindowSurfaceOrchestrationData,
    pub menu_chrome: HostMenuChromeData,
    pub page_chrome: HostPageChromeData,
    pub status_bar: HostStatusBarData,
    pub resize_layer: HostResizeLayerData,
    pub drag_overlay: HostTabDragOverlayData,
    pub left_dock: HostSideDockSurfaceData,
    pub document_dock: HostDocumentDockSurfaceData,
    pub right_dock: HostSideDockSurfaceData,
    pub bottom_dock: HostBottomDockSurfaceData,
    pub floating_layer: HostFloatingWindowLayerData,
}

impl HostWindowSceneData {
    pub(crate) fn document_surfaces(&self) -> &[HostDocumentDockSurfaceData] {
        if self.document_leaves.is_empty() {
            std::slice::from_ref(&self.document_dock)
        } else {
            &self.document_leaves
        }
    }
}
