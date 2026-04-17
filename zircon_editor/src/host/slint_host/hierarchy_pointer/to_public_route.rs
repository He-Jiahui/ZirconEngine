use super::hierarchy_pointer_route::HierarchyPointerRoute;
use super::hierarchy_pointer_target::HierarchyPointerTarget;

pub(super) fn to_public_route(target: HierarchyPointerTarget) -> HierarchyPointerRoute {
    match target {
        HierarchyPointerTarget::Node {
            item_index,
            node_id,
        } => HierarchyPointerRoute::Node {
            item_index,
            node_id,
        },
        HierarchyPointerTarget::ListSurface => HierarchyPointerRoute::ListSurface,
    }
}
