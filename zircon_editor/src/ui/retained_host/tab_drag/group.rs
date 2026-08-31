use crate::ui::retained_host::shell_pointer::HostShellPointerRoute;
use crate::ui::workbench::layout::DockEdge;
use crate::ui::workbench::layout::MainPageId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostDragTargetGroup {
    Left,
    Right,
    Bottom,
    Document,
}

impl HostDragTargetGroup {
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

pub(crate) fn host_shell_pointer_route_group_key(route: &HostShellPointerRoute) -> Option<String> {
    match route {
        HostShellPointerRoute::DragTarget(group) => Some(group.as_str().to_string()),
        HostShellPointerRoute::DocumentEdge(edge) => {
            Some(document_edge_group_key(*edge).to_string())
        }
        HostShellPointerRoute::FloatingWindow(window_id) => {
            Some(floating_window_group_key(window_id))
        }
        HostShellPointerRoute::FloatingWindowEdge { window_id, edge } => {
            Some(floating_window_edge_group_key(window_id, *edge))
        }
        HostShellPointerRoute::Resize(_) => None,
    }
}

pub(crate) fn host_shell_pointer_route_matches_group_key(
    route: &HostShellPointerRoute,
    group_key: &str,
) -> bool {
    match route {
        HostShellPointerRoute::DragTarget(group) => group_key == group.as_str(),
        HostShellPointerRoute::DocumentEdge(edge) => group_key == document_edge_group_key(*edge),
        HostShellPointerRoute::FloatingWindow(window_id) => group_key
            .strip_prefix(FLOATING_WINDOW_GROUP_PREFIX)
            .is_some_and(|value| value == window_id.0.as_str()),
        HostShellPointerRoute::FloatingWindowEdge { window_id, edge } => group_key
            .strip_prefix(FLOATING_WINDOW_EDGE_GROUP_PREFIX)
            .and_then(|value| value.rsplit_once('/'))
            .is_some_and(|(value, segment)| {
                value == window_id.0.as_str() && segment == floating_edge_segment(*edge)
            }),
        HostShellPointerRoute::Resize(_) => false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_routes_match_their_materialized_group_keys() {
        let window_id = MainPageId::new("floating/alpha");
        let routes = [
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Left),
            HostShellPointerRoute::DocumentEdge(DockEdge::Bottom),
            HostShellPointerRoute::FloatingWindow(window_id.clone()),
            HostShellPointerRoute::FloatingWindowEdge {
                window_id,
                edge: DockEdge::Right,
            },
        ];

        for route in routes {
            let key = host_shell_pointer_route_group_key(&route)
                .expect("drag route should have a group key");
            assert!(host_shell_pointer_route_matches_group_key(&route, &key));
        }
    }

    #[test]
    fn typed_route_match_rejects_other_groups_and_partial_floating_ids() {
        let route = HostShellPointerRoute::FloatingWindowEdge {
            window_id: MainPageId::new("alpha/beta"),
            edge: DockEdge::Top,
        };

        assert!(!host_shell_pointer_route_matches_group_key(
            &route,
            "floating-window-edge/alpha/top"
        ));
        assert!(!host_shell_pointer_route_matches_group_key(
            &route,
            "floating-window-edge/alpha/beta/bottom"
        ));
    }
}
