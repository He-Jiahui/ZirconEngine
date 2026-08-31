use zircon_runtime_interface::math::UVec2;

use crate::core::play::PlayPreviewFrameIdentity;
use crate::core::play::WorldDomain;
use crate::scene::viewport::HandleScreenLine;

use super::{capture_projection, PlayGizmoError, PlayGizmoProjection};
use crate::ui::host::EditorHostEventController;

pub(crate) struct PlayGizmoOverlaySnapshot {
    resource_scope: String,
    viewport: UVec2,
    lines: Vec<HandleScreenLine>,
}

impl PlayGizmoOverlaySnapshot {
    pub(crate) fn into_raster_parts(self) -> (String, UVec2, Vec<HandleScreenLine>) {
        (self.resource_scope, self.viewport, self.lines)
    }
}

impl EditorHostEventController {
    pub(crate) fn play_gizmo_overlay_snapshot(
        &self,
        frame: &PlayPreviewFrameIdentity,
    ) -> Result<Option<PlayGizmoOverlaySnapshot>, PlayGizmoError> {
        let Some(gateway) = self.validate_play_gizmo_frame(frame)? else {
            return Ok(None);
        };
        let Some(entity) = self.active_play_gizmo_entity(frame.instance()) else {
            self.play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .invalidate_projection();
            return Ok(None);
        };

        let (active, cached) = {
            let owner = self
                .play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let active = owner
                .active
                .as_ref()
                .filter(|active| {
                    active.projection.instance == frame.instance()
                        && active.projection.gateway == *frame.gateway()
                        && active.projection.entity == entity
                })
                .map(|active| PlayGizmoProjection {
                    transform: active.current,
                    ..active.projection.clone()
                });
            let cached = owner
                .projection
                .as_ref()
                .filter(|projection| projection.matches(frame, entity))
                .cloned();
            (active, cached)
        };
        let projection = match active.or(cached) {
            Some(projection) => projection,
            None => {
                let projection = capture_projection(&gateway, frame, entity)?;
                self.play_gizmo
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .projection = Some(projection.clone());
                zircon_runtime::profile_counter!(
                    "editor",
                    "play.gizmo.overlay_projection_query_count",
                    1
                );
                projection
            }
        };
        let camera = self.play_gizmo_camera()?;
        let viewport = UVec2::new(frame.size().0, frame.size().1);
        let lines = {
            let shell = self.shell().lock();
            let selection = shell.state.viewport_controller.selection();
            if !shell.state.is_playing()
                || selection.active_domain() != WorldDomain::Play(frame.instance())
                || selection.active_primary() != Some(entity)
            {
                return Ok(None);
            }
            shell
                .state
                .viewport_controller
                .handle_screen_lines_for_transform(
                    Some((entity, projection.transform)),
                    &camera,
                    viewport,
                )
        };
        if lines.is_empty() {
            return Ok(None);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "play.gizmo.overlay_projected_line_count",
            lines.len()
        );
        let resource_prefix = format!("play-gizmo:{entity}:{}", projection.world_replacement_epoch);
        Ok(Some(PlayGizmoOverlaySnapshot {
            resource_scope: frame.resource_scope(&resource_prefix),
            viewport,
            lines,
        }))
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

    use crate::core::play::{PlayInstanceId, PlayPreviewFrame};

    #[test]
    fn overlay_resource_scope_pins_complete_frame_world_and_entity_identity() {
        let frame = PlayPreviewFrame::for_test(
            PlayInstanceId::for_test(7),
            GatewaySessionIdentity::new(11, ZrRuntimeSessionHandle::new(13), 17, None)
                .with_gateway_generation(19)
                .with_play_instance(Some(7)),
            640,
            360,
            23,
            vec![0; 640 * 360 * 4],
        );

        assert_eq!(
            frame.identity().resource_scope("play-gizmo:29:31"),
            "play-gizmo:29:31:7:11:13:17:19:some:7:640:360:23:none"
        );
    }
}
