#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::shell_pointer::HostShellPointerRoute;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::layout::{
    DockEdge, MainPageId, SplitAxis, SplitPlacement, WorkbenchLayout, WorkspaceTarget,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::ViewHost;
#[cfg(test)]
use zircon_runtime_interface::ui::layout::UiFrame;

use super::drop_resolution::resolve_tab_drop_with_workbench_layout_frames;
use super::group::{
    document_edge_from_group_key, floating_window_edge_from_group_key,
    floating_window_from_group_key, HostDragTargetGroup,
};
use super::host_resolution::{active_floating_window_path, preferred_document_page};
use super::resolved_drop::{ResolvedHostTabDropRoute, ResolvedHostTabDropTarget, ResolvedTabDrop};

#[cfg(test)]
pub(crate) fn resolve_host_tab_drop_route(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<HostShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<ResolvedHostTabDropRoute> {
    resolve_host_tab_drop_route_with_root_frames(
        layout,
        model,
        metrics,
        instance_id,
        pointer_route,
        fallback_target_group,
        pointer_x,
        pointer_y,
        None,
    )
}

#[cfg(test)]
pub(crate) fn resolve_host_tab_drop_route_with_root_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<HostShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ResolvedHostTabDropRoute> {
    resolve_host_tab_drop_route_with_workbench_layout_frames(
        layout,
        model,
        metrics,
        instance_id,
        pointer_route,
        fallback_target_group,
        pointer_x,
        pointer_y,
        test_workbench_layout_frames_from_root_frames(shared_root_frames, metrics),
    )
}

pub(crate) fn resolve_host_tab_drop_route_with_workbench_layout_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<HostShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<ResolvedHostTabDropRoute> {
    resolve_host_tab_drop_route_with_frames(
        layout,
        model,
        metrics,
        instance_id,
        pointer_route,
        fallback_target_group,
        pointer_x,
        pointer_y,
        componentized_workbench_layout_frames,
    )
}

fn resolve_host_tab_drop_route_with_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<HostShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<ResolvedHostTabDropRoute> {
    match pointer_route {
        Some(HostShellPointerRoute::DocumentEdge(edge)) => {
            document_edge_drop_route(layout, model, edge)
        }
        Some(HostShellPointerRoute::FloatingWindow(window_id)) => {
            floating_window_attach_route(layout, &window_id)
        }
        Some(HostShellPointerRoute::FloatingWindowEdge { window_id, edge }) => {
            floating_window_edge_drop_route(layout, &window_id, edge)
        }
        Some(HostShellPointerRoute::DragTarget(target_group)) => {
            if target_group == HostDragTargetGroup::Document {
                if let Some(edge) = document_edge_from_group_key(fallback_target_group) {
                    return document_edge_drop_route(layout, model, edge);
                }
            }
            let drop = resolve_tab_drop_with_workbench_layout_frames(
                layout,
                model,
                metrics,
                instance_id,
                target_group.as_str(),
                pointer_x,
                pointer_y,
                componentized_workbench_layout_frames,
            )?;
            Some(ResolvedHostTabDropRoute {
                target_group,
                target_label: target_group.label(),
                target: ResolvedHostTabDropTarget::Attach(drop),
            })
        }
        Some(HostShellPointerRoute::Resize(_)) | None => resolve_fallback_drop_route(
            layout,
            model,
            metrics,
            instance_id,
            fallback_target_group,
            pointer_x,
            pointer_y,
            componentized_workbench_layout_frames,
        ),
    }
}

fn active_document_workspace_target(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
) -> Option<(MainPageId, Vec<usize>)> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
        .and_then(|tab| match &tab.workspace {
            WorkspaceTarget::MainPage(page_id) => {
                Some((page_id.clone(), tab.workspace_path.clone()))
            }
            WorkspaceTarget::FloatingWindow(_) => None,
        })
        .or_else(|| preferred_document_page(layout).map(|page_id| (page_id, Vec::new())))
}

fn document_edge_drop_route(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    edge: DockEdge,
) -> Option<ResolvedHostTabDropRoute> {
    let (page_id, path) = active_document_workspace_target(layout, model)?;
    Some(ResolvedHostTabDropRoute {
        target_group: HostDragTargetGroup::Document,
        target_label: document_edge_label(edge),
        target: ResolvedHostTabDropTarget::Split {
            workspace: WorkspaceTarget::MainPage(page_id),
            path,
            axis: edge_axis(edge),
            placement: edge_placement(edge),
        },
    })
}

fn document_edge_label(edge: DockEdge) -> &'static str {
    match edge {
        DockEdge::Left => "Split Document Left",
        DockEdge::Right => "Split Document Right",
        DockEdge::Top => "Split Document Top",
        DockEdge::Bottom => "Split Document Bottom",
    }
}

fn floating_window_attach_route(
    layout: &WorkbenchLayout,
    window_id: &MainPageId,
) -> Option<ResolvedHostTabDropRoute> {
    let path = active_floating_window_path(layout, window_id)?;
    Some(ResolvedHostTabDropRoute {
        target_group: HostDragTargetGroup::Document,
        target_label: "floating window",
        target: ResolvedHostTabDropTarget::Attach(ResolvedTabDrop {
            host: ViewHost::FloatingWindow(window_id.clone(), path),
            anchor: None,
        }),
    })
}

fn floating_window_edge_drop_route(
    layout: &WorkbenchLayout,
    window_id: &MainPageId,
    edge: DockEdge,
) -> Option<ResolvedHostTabDropRoute> {
    let path = active_floating_window_path(layout, window_id)?;
    Some(ResolvedHostTabDropRoute {
        target_group: HostDragTargetGroup::Document,
        target_label: floating_window_edge_label(edge),
        target: ResolvedHostTabDropTarget::Split {
            workspace: WorkspaceTarget::FloatingWindow(window_id.clone()),
            path,
            axis: edge_axis(edge),
            placement: edge_placement(edge),
        },
    })
}

fn floating_window_edge_label(edge: DockEdge) -> &'static str {
    match edge {
        DockEdge::Left => "Split Floating Window Left",
        DockEdge::Right => "Split Floating Window Right",
        DockEdge::Top => "Split Floating Window Top",
        DockEdge::Bottom => "Split Floating Window Bottom",
    }
}

fn edge_axis(edge: DockEdge) -> SplitAxis {
    match edge {
        DockEdge::Left | DockEdge::Right => SplitAxis::Horizontal,
        DockEdge::Top | DockEdge::Bottom => SplitAxis::Vertical,
    }
}

fn edge_placement(edge: DockEdge) -> SplitPlacement {
    match edge {
        DockEdge::Left | DockEdge::Top => SplitPlacement::Before,
        DockEdge::Right | DockEdge::Bottom => SplitPlacement::After,
    }
}

fn resolve_fallback_drop_route(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<ResolvedHostTabDropRoute> {
    if let Some(edge) = document_edge_from_group_key(fallback_target_group) {
        return document_edge_drop_route(layout, model, edge);
    }
    if let Some((window_id, edge)) = floating_window_edge_from_group_key(fallback_target_group) {
        return floating_window_edge_drop_route(layout, &window_id, edge);
    }
    if let Some(window_id) = floating_window_from_group_key(fallback_target_group) {
        return floating_window_attach_route(layout, &window_id);
    }

    let target_group = HostDragTargetGroup::from_str(fallback_target_group)?;
    let drop = resolve_tab_drop_with_workbench_layout_frames(
        layout,
        model,
        metrics,
        instance_id,
        target_group.as_str(),
        pointer_x,
        pointer_y,
        componentized_workbench_layout_frames,
    )?;
    Some(ResolvedHostTabDropRoute {
        target_group,
        target_label: target_group.label(),
        target: ResolvedHostTabDropTarget::Attach(drop),
    })
}

#[cfg(test)]
fn test_workbench_layout_frames_from_root_frames(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    metrics: &WorkbenchChromeMetrics,
) -> BuiltinWorkbenchWindowLayoutFrames {
    let document_region_frame = shared_root_frames.and_then(|frames| frames.document_host_frame);
    let document_tabs_frame = shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .or_else(|| {
            document_region_frame
                .filter(ui_frame_is_visible)
                .map(|frame| {
                    UiFrame::new(
                        frame.x,
                        frame.y,
                        frame.width,
                        metrics.document_header_height.max(0.0),
                    )
                })
        });

    BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: shared_root_frames.and_then(|frames| frames.host_body_frame),
        document_tabs_frame,
        document_region_frame,
        ..BuiltinWorkbenchWindowLayoutFrames::default()
    }
}

#[cfg(test)]
fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
