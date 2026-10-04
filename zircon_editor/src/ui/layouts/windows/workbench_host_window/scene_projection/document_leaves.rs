use super::super::chrome_template_projection::dock_overflow_frame;
use super::*;

pub(super) fn build_document_leaves(
    surface: &HostWindowSurfaceData,
    shell: &HostWindowShellData,
    layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    mut project_pane: impl FnMut(
        &super::super::document_leaves::DocumentLeafSurfaceData,
        f32,
        f32,
    ) -> PaneData,
) -> Vec<HostDocumentDockSurfaceData> {
    surface
        .document_leaves
        .iter()
        .map(|leaf| {
            let bounds = &layout.document_region_frame;
            let region = FrameRect {
                x: bounds.x + leaf.relative_frame.x * bounds.width,
                y: bounds.y + leaf.relative_frame.y * bounds.height,
                width: leaf.relative_frame.width * bounds.width,
                height: leaf.relative_frame.height * bounds.height,
            };
            let header_height = metrics
                .document_header_height_px
                .min(region.height)
                .max(0.0);
            let separator = (region.height - header_height).clamp(0.0, 1.0);
            let content_height = (region.height - header_height - separator).max(0.0);
            let header_nodes = document_dock_header_nodes(
                &leaf.tabs,
                &leaf.pane.subtitle,
                &shell.panel_preset_id,
                region.width,
                header_height,
            );
            HostDocumentDockSurfaceData {
                surface_key: format!("document:{}", leaf.node_id).into(),
                region_frame: region.clone(),
                header_frame: dock_header_frame(&header_nodes),
                overflow_frame: dock_overflow_frame(&header_nodes),
                subtitle_frame: dock_subtitle_frame(&header_nodes),
                content_frame: FrameRect {
                    x: 0.0,
                    y: header_height + separator,
                    width: region.width,
                    height: content_height,
                },
                tab_frames: dock_tab_frames(&header_nodes, &leaf.tabs),
                tabs: leaf.tabs.clone(),
                header_nodes,
                pane: project_pane(leaf, region.width, content_height),
                header_height_px: header_height,
            }
        })
        .collect()
}
