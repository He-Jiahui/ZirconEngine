use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

/// Ephemeral handler view over the route that becomes the dispatch result authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiNavigationDispatchContext<'route> {
    pub node_id: UiNodeId,
    pub route: &'route UiNavigationRoute,
}
