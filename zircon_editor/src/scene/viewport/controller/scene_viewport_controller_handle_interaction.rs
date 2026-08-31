use crate::scene::viewport::{HandleOverlayExtract, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Transform, Vec2};

use crate::core::editing::interactive_transform::selection_pivot_transform;
use crate::scene::viewport::handles::HandleSelection;
use crate::scene::viewport::GizmoAxis;

use super::{viewport_drag_session::ViewportDragSession, SceneViewportController};

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn projected_selected_node(
        &self,
        scene: &Scene,
    ) -> Option<u64> {
        self.state
            .selection
            .active_primary()
            .filter(|entity| scene.contains_entity(*entity))
    }

    pub(in crate::scene::viewport::controller) fn handle_overlays(
        &self,
        scene: &Scene,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let selected = self.selected_handle_transform(scene);
        self.handle_overlays_for_transform(selected, camera)
    }

    pub(crate) fn handle_overlays_for_transform(
        &self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let handle_kind = self.base_transform_handle();
        self.handles.build_overlays(
            selected.map(|(entity, transform)| HandleSelection { entity, transform }),
            &self.state.settings,
            handle_kind,
            camera,
        )
    }

    pub(crate) fn handle_axis_at_cursor_for_transform(
        &self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
    ) -> Option<GizmoAxis> {
        if !self.state.settings.gizmos_enabled {
            return None;
        }
        let overlays = self.handle_overlays_for_transform(selected, camera);
        match crate::scene::viewport::pointer::local_handle_route(
            &overlays,
            camera,
            self.state.viewport.size,
            cursor,
        ) {
            Some(crate::scene::viewport::pointer::ViewportPointerRoute::HandleAxis {
                axis,
                ..
            }) => Some(axis),
            _ => None,
        }
    }

    pub(in crate::scene::viewport::controller) fn begin_handle_drag(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let camera = self.current_camera(scene);
        let selected = self.selected_handle_transform(scene);
        self.begin_handle_drag_for_transform(selected, &camera, cursor, axis)
    }

    pub(crate) fn begin_handle_drag_for_transform(
        &mut self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let snap_steps = self.snap_steps();
        let handle_kind = self.base_transform_handle();
        let Some(session) = self.handles.begin_drag(
            selected.map(|(entity, transform)| HandleSelection { entity, transform }),
            &self.state.settings,
            handle_kind,
            snap_steps,
            camera,
            cursor,
            axis,
        ) else {
            return false;
        };

        self.state.drag = Some(ViewportDragSession::Handle { session });
        self.state.hover.hovered_axis = Some(axis);
        true
    }

    pub(crate) fn update_handle_drag_for_transform(
        &mut self,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
    ) -> Option<crate::scene::viewport::ViewportTransformRequest> {
        let Some(ViewportDragSession::Handle { mut session }) = self.state.drag.take() else {
            return None;
        };
        let preview = self
            .handles
            .update_drag(&mut session, camera, self.state.viewport.size, cursor)
            .map(
                |target_pivot_world| crate::scene::viewport::ViewportTransformRequest {
                    primary: session.node_id(),
                    target_pivot_world,
                },
            );
        self.state.drag = Some(ViewportDragSession::Handle { session });
        preview
    }

    pub(crate) fn finish_handle_drag_for_transform(&mut self) -> bool {
        let Some(ViewportDragSession::Handle { session }) = self.state.drag.take() else {
            return false;
        };
        self.handles.end_drag(session);
        true
    }

    pub(crate) fn set_handle_hover_for_transform(&mut self, axis: Option<GizmoAxis>) -> bool {
        let changed = self.state.hover.hovered_axis != axis;
        self.state.hover.hovered_axis = axis;
        changed
    }

    pub(crate) fn active_interactive_transform_spec(
        &self,
    ) -> Option<crate::core::editing::interactive_transform::InteractiveTransformSpec> {
        match self.state.drag.as_ref()? {
            ViewportDragSession::Handle { session } => Some(session.interactive_transform_spec()),
            _ => None,
        }
    }

    fn selected_handle_transform(&self, scene: &Scene) -> Option<(u64, Transform)> {
        let primary = self.projected_selected_node(scene)?;
        selection_pivot_transform(
            scene,
            self.state.selection.active_items().iter().copied(),
            primary,
            self.interactive_transform_pivot_mode(),
        )
    }

    pub(in crate::scene::viewport::controller) fn end_handle_drag(&mut self) {
        let Some(ViewportDragSession::Handle { session }) = self.state.drag.take() else {
            return;
        };
        self.handles.end_drag(session);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editor_message::SceneModeId;
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeCtx, ViewportOverlayBuilder,
    };
    use crate::scene::viewport::{PivotMode, TransformHandleKind, ViewportInput};
    use crate::ui::binding::ViewportCommand;
    use zircon_runtime_interface::math::{Transform, UVec2, Vec2, Vec3};

    use super::{SceneViewportController, ViewportCameraSnapshot};

    struct PassThroughOverlayMode {
        id: SceneModeId,
    }

    impl EditorSceneMode for PassThroughOverlayMode {
        fn id(&self) -> &SceneModeId {
            &self.id
        }

        fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

        fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

        fn handle_input(
            &mut self,
            _input: &ViewportInput,
            _ctx: &mut SceneModeCtx<'_>,
        ) -> InputOutcome {
            InputOutcome::PassThrough
        }

        fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
    }

    #[test]
    fn pass_through_overlay_keeps_base_transform_handles_in_the_render_extract() {
        let scene = zircon_runtime::scene::Scene::new();
        let selected = scene.nodes()[0].id;
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        controller.selection_mut().select_only_active(selected);
        controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Rotate))
            .unwrap();
        {
            let state = &mut controller.state;
            let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state
                .scene_modes
                .push_overlay(
                    SceneModeActivation::Custom(SceneModeId::new("test.pass-through-overlay")),
                    Box::new(PassThroughOverlayMode {
                        id: SceneModeId::new("test.pass-through-overlay"),
                    }),
                    &mut mode_ctx,
                )
                .unwrap();
        }

        assert_eq!(
            controller.base_transform_handle(),
            Some(TransformHandleKind::Rotate)
        );
        assert!(!controller
            .handle_overlays(&scene, &ViewportCameraSnapshot::default())
            .is_empty());
    }

    #[test]
    fn world_neutral_handle_route_drives_one_local_drag_session() {
        let mut controller = SceneViewportController::new(UVec2::new(800, 600));
        controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
            .unwrap();
        let camera = ViewportCameraSnapshot::default();
        let selected = Some((
            41,
            Transform {
                translation: Vec3::new(0.0, 0.0, -5.0),
                ..Transform::identity()
            },
        ));
        let cursor = Vec2::new(400.0, 300.0);
        let axis = controller
            .handle_axis_at_cursor_for_transform(selected, &camera, cursor)
            .expect("the projected handle origin should route to one axis");

        assert!(controller.begin_handle_drag_for_transform(selected, &camera, cursor, axis));
        let preview = controller
            .update_handle_drag_for_transform(&camera, Vec2::new(430.0, 300.0))
            .expect("an active local handle drag should produce a transform preview");

        assert_eq!(preview.primary, 41);
        assert_ne!(preview.target_pivot_world, selected.unwrap().1);
        assert!(controller.finish_handle_drag_for_transform());
        assert!(!controller.is_handle_drag_active());
    }

    #[test]
    fn world_neutral_handle_overlay_projects_runtime_transform_and_hover_state() {
        let mut controller = SceneViewportController::new(UVec2::new(800, 600));
        controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
            .unwrap();
        let camera = ViewportCameraSnapshot::default();
        let selected = Some((
            41,
            Transform {
                translation: Vec3::new(0.0, 0.0, -5.0),
                ..Transform::identity()
            },
        ));

        let passive =
            controller.handle_screen_lines_for_transform(selected, &camera, UVec2::new(800, 600));
        assert!(!passive.is_empty());
        assert!(passive.iter().all(|line| line.is_finite()));

        controller.set_handle_hover_for_transform(Some(crate::scene::viewport::GizmoAxis::X));
        let hovered =
            controller.handle_screen_lines_for_transform(selected, &camera, UVec2::new(800, 600));
        assert_eq!(hovered.len(), passive.len());
        assert!(hovered.iter().any(|line| {
            line.axis() == Some(crate::scene::viewport::GizmoAxis::X)
                && line.width()
                    > passive
                        .iter()
                        .find(|passive| passive.axis() == line.axis())
                        .expect("passive X axis line")
                        .width()
        }));
    }

    #[test]
    fn multi_selection_handle_uses_the_same_centroid_as_the_transform_session() {
        let mut scene = zircon_runtime::scene::Scene::empty();
        let left = scene
            .spawn_node(zircon_runtime::scene::components::NodeKind::Cube)
            .unwrap();
        let right = scene
            .spawn_node(zircon_runtime::scene::components::NodeKind::Cube)
            .unwrap();
        scene
            .update_transform(left, Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)))
            .unwrap();
        scene
            .update_transform(right, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
            .unwrap();
        let mut controller = SceneViewportController::new(UVec2::new(800, 600));
        controller
            .selection_mut()
            .replace_active([left, right], Some(left));

        let (primary, pivot) = controller
            .selected_handle_transform(&scene)
            .expect("a valid multi-selection should expose one shared gizmo pivot");

        assert_eq!(primary, left);
        assert_eq!(pivot.translation, Vec3::ZERO);

        let feedback = controller
            .apply_command(None, &ViewportCommand::SetPivotMode(PivotMode::Primary))
            .expect("pivot mode command should update the viewport controller");
        let (_, primary_pivot) = controller
            .selected_handle_transform(&scene)
            .expect("the same selection should expose its primary pivot");

        assert!(feedback.settings_changed);
        assert!(feedback.interaction_extract_stale);
        assert_eq!(primary_pivot.translation, Vec3::new(-4.0, 0.0, 0.0));
    }
}
