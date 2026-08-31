use crate::ui::dispatch::UiDispatchPhase;
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

/// Ephemeral handler view over the route that becomes the dispatch result authority.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPointerDispatchContext<'route> {
    pub node_id: UiNodeId,
    pub phase: UiDispatchPhase,
    pub route: &'route UiPointerRoute,
}
