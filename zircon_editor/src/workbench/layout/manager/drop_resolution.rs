use crate::ViewHost;

use super::super::{
    DockEdge, DragPayload, DropTarget, HitTarget, LayoutManager, SplitAxis, SplitPlacement,
    WorkspaceTarget,
};

impl LayoutManager {
    pub fn resolve_drop(&self, payload: DragPayload, target: HitTarget) -> DropTarget {
        match target {
            HitTarget::Drawer(slot) => DropTarget::Host(ViewHost::Drawer(slot)),
            HitTarget::Document(page_id, path) => {
                DropTarget::Host(ViewHost::Document(page_id, path))
            }
            HitTarget::DocumentEdge {
                page_id,
                path,
                edge,
            } => DropTarget::Split {
                workspace: WorkspaceTarget::MainPage(page_id),
                path,
                axis: edge_axis(edge),
                placement: edge_placement(edge),
            },
            HitTarget::FloatingWindow(window_id, path) => {
                DropTarget::Host(ViewHost::FloatingWindow(window_id, path))
            }
            HitTarget::FloatingWindowEdge {
                window_id,
                path,
                edge,
            } => DropTarget::Split {
                workspace: WorkspaceTarget::FloatingWindow(window_id),
                path,
                axis: edge_axis(edge),
                placement: edge_placement(edge),
            },
            HitTarget::ExclusivePage(page_id) => {
                let _ = payload;
                DropTarget::Host(ViewHost::ExclusivePage(page_id))
            }
            HitTarget::NewFloatingWindow => DropTarget::NewFloatingWindow,
        }
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
