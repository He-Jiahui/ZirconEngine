use crate::scene::viewport::GizmoAxis;
use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::scene::viewport::pointer::ViewportPointerRoute;

use super::SceneViewportController;

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn sync_pointer_bridge(
        &mut self,
        scene: &Scene,
    ) -> ViewportCameraSnapshot {
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let handles = &self.handles;
        let settings = &self.state.settings;
        let handle_kind = self.active_transform_handle();
        let interaction_extract = self.interaction_extract.resolve_for_pointer(
            scene,
            selected,
            settings,
            &camera,
            self.state.viewport.size,
            || handles.build_overlays(scene, selected, settings, handle_kind, &camera),
            || self.viewport_overlay_gizmos(scene, selected),
        );
        self.pointer_bridge
            .sync_scene(&camera, self.state.viewport.size, interaction_extract);
        camera
    }

    pub(in crate::scene::viewport::controller) fn route_at_cursor(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        press: bool,
    ) -> Option<ViewportPointerRoute> {
        let _camera = self.sync_pointer_bridge(scene);
        let point = UiPoint::new(cursor.x, cursor.y);
        let dispatch = if press {
            self.pointer_bridge.handle_down(point)
        } else {
            self.pointer_bridge.handle_move(point)
        }
        .ok()?;
        dispatch.route
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

pub(in crate::scene::viewport::controller) fn route_owner(route: &ViewportPointerRoute) -> u64 {
    route.target().owner()
}
