use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::shell_pointer::WorkbenchShellPointerRoute;
use crate::{
    DockEdge, MainPageId, SplitAxis, SplitPlacement, ViewHost, WorkbenchChromeMetrics,
    WorkbenchLayout, WorkbenchShellGeometry, WorkbenchViewModel, WorkspaceTarget,
};

use super::drop_resolution::resolve_tab_drop_with_root_frames;
use super::group::{
    document_edge_from_group_key, floating_window_edge_from_group_key,
    floating_window_from_group_key, WorkbenchDragTargetGroup,
};
use super::host_resolution::{active_floating_window_path, preferred_document_page};
use super::resolved_drop::{
    ResolvedTabDrop, ResolvedWorkbenchTabDropRoute, ResolvedWorkbenchTabDropTarget,
};

#[cfg(test)]
pub(crate) fn resolve_workbench_tab_drop_route(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<WorkbenchShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<ResolvedWorkbenchTabDropRoute> {
    resolve_workbench_tab_drop_route_with_root_frames(
        layout,
        model,
        geometry,
        metrics,
        instance_id,
        pointer_route,
        fallback_target_group,
        pointer_x,
        pointer_y,
        None,
    )
}

pub(crate) fn resolve_workbench_tab_drop_route_with_root_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    pointer_route: Option<WorkbenchShellPointerRoute>,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ResolvedWorkbenchTabDropRoute> {
    match pointer_route {
        Some(WorkbenchShellPointerRoute::DocumentEdge(edge)) => {
            document_edge_drop_route(layout, model, edge)
        }
        Some(WorkbenchShellPointerRoute::FloatingWindow(window_id)) => {
            floating_window_attach_route(layout, &window_id)
        }
        Some(WorkbenchShellPointerRoute::FloatingWindowEdge { window_id, edge }) => {
            floating_window_edge_drop_route(layout, &window_id, edge)
        }
        Some(WorkbenchShellPointerRoute::DragTarget(target_group)) => {
            if target_group == WorkbenchDragTargetGroup::Document {
                if let Some(edge) = document_edge_from_group_key(fallback_target_group) {
                    return document_edge_drop_route(layout, model, edge);
                }
            }
            let drop = resolve_tab_drop_with_root_frames(
                layout,
                model,
                geometry,
                metrics,
                instance_id,
                target_group.as_str(),
                pointer_x,
                pointer_y,
                shared_root_frames,
            )?;
            Some(ResolvedWorkbenchTabDropRoute {
                target_group,
                target_label: target_group.label(),
                target: ResolvedWorkbenchTabDropTarget::Attach(drop),
            })
        }
        Some(WorkbenchShellPointerRoute::Resize(_)) | None => resolve_fallback_drop_route(
            layout,
            model,
            geometry,
            metrics,
            instance_id,
            fallback_target_group,
            pointer_x,
            pointer_y,
            shared_root_frames,
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
) -> Option<ResolvedWorkbenchTabDropRoute> {
    let (page_id, path) = active_document_workspace_target(layout, model)?;
    Some(ResolvedWorkbenchTabDropRoute {
        target_group: WorkbenchDragTargetGroup::Document,
        target_label: document_edge_label(edge),
        target: ResolvedWorkbenchTabDropTarget::Split {
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
) -> Option<ResolvedWorkbenchTabDropRoute> {
    let path = active_floating_window_path(layout, window_id)?;
    Some(ResolvedWorkbenchTabDropRoute {
        target_group: WorkbenchDragTargetGroup::Document,
        target_label: "floating window",
        target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
            host: ViewHost::FloatingWindow(window_id.clone(), path),
            anchor: None,
        }),
    })
}

fn floating_window_edge_drop_route(
    layout: &WorkbenchLayout,
    window_id: &MainPageId,
    edge: DockEdge,
) -> Option<ResolvedWorkbenchTabDropRoute> {
    let path = active_floating_window_path(layout, window_id)?;
    Some(ResolvedWorkbenchTabDropRoute {
        target_group: WorkbenchDragTargetGroup::Document,
        target_label: floating_window_edge_label(edge),
        target: ResolvedWorkbenchTabDropTarget::Split {
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
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    fallback_target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ResolvedWorkbenchTabDropRoute> {
    if let Some(edge) = document_edge_from_group_key(fallback_target_group) {
        return document_edge_drop_route(layout, model, edge);
    }
    if let Some((window_id, edge)) = floating_window_edge_from_group_key(fallback_target_group) {
        return floating_window_edge_drop_route(layout, &window_id, edge);
    }
    if let Some(window_id) = floating_window_from_group_key(fallback_target_group) {
        return floating_window_attach_route(layout, &window_id);
    }

    let target_group = WorkbenchDragTargetGroup::from_str(fallback_target_group)?;
    let drop = resolve_tab_drop_with_root_frames(
        layout,
        model,
        geometry,
        metrics,
        instance_id,
        target_group.as_str(),
        pointer_x,
        pointer_y,
        shared_root_frames,
    )?;
    Some(ResolvedWorkbenchTabDropRoute {
        target_group,
        target_label: target_group.label(),
        target: ResolvedWorkbenchTabDropTarget::Attach(drop),
    })
}
