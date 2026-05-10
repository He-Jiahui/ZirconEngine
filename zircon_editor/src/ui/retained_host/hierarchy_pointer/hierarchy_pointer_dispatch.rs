use super::hierarchy_pointer_route::HierarchyPointerRoute;
use super::hierarchy_pointer_state::HierarchyPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HierarchyPointerDispatch {
    pub route: Option<HierarchyPointerRoute>,
    pub state: HierarchyPointerState,
}
