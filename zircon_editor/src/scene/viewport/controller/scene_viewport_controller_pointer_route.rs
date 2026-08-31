use crate::scene::viewport::{
    GizmoAxis, ViewportCameraSnapshot, ViewportInteractionExtractPointerResolution,
};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::{
    math::Vec2,
    ui::{layout::UiPoint, tree::UiTreeError},
};

use crate::scene::viewport::pointer::{ViewportPointerDispatch, ViewportPointerRoute};

use super::SceneViewportController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::scene::viewport::controller) enum PointerBridgeProductState {
    Current,
    Stale,
    Preparing,
}

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn sync_pointer_bridge(
        &mut self,
        scene: &Scene,
    ) -> (ViewportCameraSnapshot, PointerBridgeProductState) {
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let settings = &self.state.settings;
        let interaction_extract = self.interaction_extract.resolve_for_pointer(
            scene,
            selected,
            settings,
            &camera,
            self.state.viewport.size,
        );
        match interaction_extract {
            ViewportInteractionExtractPointerResolution::Ready(interaction_extract) => {
                self.pointer_bridge.sync_scene(
                    &camera,
                    self.state.viewport.size,
                    scene.world_generation(),
                    interaction_extract,
                );
                (camera, PointerBridgeProductState::Current)
            }
            ViewportInteractionExtractPointerResolution::Stale => {
                self.pointer_bridge.clear_scene();
                (camera, PointerBridgeProductState::Stale)
            }
            ViewportInteractionExtractPointerResolution::Preparing => {
                self.pointer_bridge.clear_scene();
                (camera, PointerBridgeProductState::Preparing)
            }
        }
    }

    pub(crate) fn sync_renderer_visible_spatial_snapshot(
        &mut self,
        scene: &Scene,
        snapshot: Option<
            zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot,
        >,
    ) -> bool {
        let (_, product_state) = self.sync_pointer_bridge(scene);
        if product_state != PointerBridgeProductState::Current {
            return false;
        }
        self.pointer_bridge
            .sync_renderer_visible_spatial_snapshot(scene.world_generation(), snapshot)
    }

    pub(crate) fn clear_renderer_visible_spatial_snapshot(&mut self) {
        self.pointer_bridge.clear_scene();
    }

    pub(in crate::scene::viewport::controller) fn route_at_cursor(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        press: bool,
    ) -> Result<(Option<ViewportPointerRoute>, PointerBridgeProductState), UiTreeError> {
        let (_camera, product_state) = self.sync_pointer_bridge(scene);
        match product_state {
            PointerBridgeProductState::Current => {}
            PointerBridgeProductState::Stale | PointerBridgeProductState::Preparing => {
                return Ok((None, product_state));
            }
        }
        let point = UiPoint::new(cursor.x, cursor.y);
        let dispatch = if press {
            self.pointer_bridge.handle_down(point)
        } else {
            self.pointer_bridge.handle_move(point)
        };
        current_pointer_route(dispatch)
    }

    pub(in crate::scene::viewport::controller) fn set_hover_route(
        &mut self,
        route: Option<&ViewportPointerRoute>,
    ) -> Option<GizmoAxis> {
        match route {
            Some(ViewportPointerRoute::HandleAxis { owner, axis }) => {
                self.state.hover.hovered_axis = Some(*axis);
                self.state.hover.hovered_entity = Some(*owner);
            }
            Some(ViewportPointerRoute::SceneGizmo { owner })
            | Some(ViewportPointerRoute::Renderable { owner }) => {
                self.state.hover.hovered_axis = None;
                self.state.hover.hovered_entity = Some(*owner);
            }
            None => {
                self.state.hover.hovered_axis = None;
                self.state.hover.hovered_entity = None;
            }
        }
        self.state.hover.hovered_axis
    }
}

fn current_pointer_route(
    dispatch: Result<ViewportPointerDispatch, UiTreeError>,
) -> Result<(Option<ViewportPointerRoute>, PointerBridgeProductState), UiTreeError> {
    let dispatch = dispatch?;
    Ok((dispatch.route, PointerBridgeProductState::Current))
}

pub(in crate::scene::viewport::controller) fn route_owner(route: &ViewportPointerRoute) -> u64 {
    route.target().owner()
}

#[cfg(test)]
mod tests {
    use super::current_pointer_route;
    use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTreeError};

    #[test]
    fn current_pointer_route_preserves_the_ui_tree_error() {
        let expected = UiTreeError::MissingNode(UiNodeId::new(901));

        let result = current_pointer_route(Err(expected.clone()));

        assert_eq!(result.unwrap_err(), expected);
    }
}
