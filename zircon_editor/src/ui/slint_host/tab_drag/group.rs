use crate::ui::slint_host::shell_pointer::WorkbenchShellPointerRoute;
use crate::{DockEdge, MainPageId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkbenchDragTargetGroup {
    Left,
    Right,
    Bottom,
    Document,
}

impl WorkbenchDragTargetGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Document => "document",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "left tool stack",
            Self::Right => "right tool stack",
            Self::Bottom => "bottom tool stack",
            Self::Document => "document workspace",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "bottom" => Some(Self::Bottom),
            "document" => Some(Self::Document),
            _ => None,
        }
    }
}

pub(crate) const fn document_edge_group_key(edge: DockEdge) -> &'static str {
    match edge {
        DockEdge::Left => "document-left",
        DockEdge::Right => "document-right",
        DockEdge::Top => "document-top",
        DockEdge::Bottom => "document-bottom",
    }
}

const FLOATING_WINDOW_GROUP_PREFIX: &str = "floating-window/";
const FLOATING_WINDOW_EDGE_GROUP_PREFIX: &str = "floating-window-edge/";

pub(crate) fn floating_window_group_key(window_id: &MainPageId) -> String {
    format!("{FLOATING_WINDOW_GROUP_PREFIX}{}", window_id.0)
}

pub(crate) fn floating_window_edge_group_key(window_id: &MainPageId, edge: DockEdge) -> String {
    format!(
        "{FLOATING_WINDOW_EDGE_GROUP_PREFIX}{}/{}",
        window_id.0,
        floating_edge_segment(edge)
    )
}

pub(crate) fn workbench_shell_pointer_route_group_key(
    route: &WorkbenchShellPointerRoute,
) -> Option<String> {
    match route {
        WorkbenchShellPointerRoute::DragTarget(group) => Some(group.as_str().to_string()),
        WorkbenchShellPointerRoute::DocumentEdge(edge) => {
            Some(document_edge_group_key(*edge).to_string())
        }
        WorkbenchShellPointerRoute::FloatingWindow(window_id) => {
            Some(floating_window_group_key(window_id))
        }
        WorkbenchShellPointerRoute::FloatingWindowEdge { window_id, edge } => {
            Some(floating_window_edge_group_key(window_id, *edge))
        }
        WorkbenchShellPointerRoute::Resize(_) => None,
    }
}

pub(super) fn document_edge_from_group_key(value: &str) -> Option<DockEdge> {
    match value {
        "document-left" => Some(DockEdge::Left),
        "document-right" => Some(DockEdge::Right),
        "document-top" => Some(DockEdge::Top),
        "document-bottom" => Some(DockEdge::Bottom),
        _ => None,
    }
}

pub(super) fn floating_window_from_group_key(value: &str) -> Option<MainPageId> {
    value
        .strip_prefix(FLOATING_WINDOW_GROUP_PREFIX)
        .filter(|window_id| !window_id.is_empty())
        .map(MainPageId::new)
}

pub(super) fn floating_window_edge_from_group_key(value: &str) -> Option<(MainPageId, DockEdge)> {
    let remainder = value.strip_prefix(FLOATING_WINDOW_EDGE_GROUP_PREFIX)?;
    let (window_id, edge) = remainder.rsplit_once('/')?;
    Some((
        MainPageId::new(window_id),
        floating_edge_from_segment(edge)?,
    ))
}

pub(super) const fn floating_edge_segment(edge: DockEdge) -> &'static str {
    match edge {
        DockEdge::Left => "left",
        DockEdge::Right => "right",
        DockEdge::Top => "top",
        DockEdge::Bottom => "bottom",
    }
}

fn floating_edge_from_segment(value: &str) -> Option<DockEdge> {
    match value {
        "left" => Some(DockEdge::Left),
        "right" => Some(DockEdge::Right),
        "top" => Some(DockEdge::Top),
        "bottom" => Some(DockEdge::Bottom),
        _ => None,
    }
}
