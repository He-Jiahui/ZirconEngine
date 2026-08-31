use thiserror::Error;
use zircon_runtime_interface::math::{Transform, Vec2};
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::world_sync::{WorldQuery, WorldQueryResult};
use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1,
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

use crate::core::editing::command::EditorCommand;
use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayError};
use crate::core::play::{
    PlayInstanceId, PlayKind, PlayMode, PlayPreviewFrameIdentity, WorldDomain,
};
use crate::scene::viewport::GizmoAxis;

use super::EditorHostEventController;

const FIRST_PLAY_GIZMO_INTERACTION_ID: u64 = 1;
const FIRST_PLAY_GIZMO_SEQUENCE: u64 = 1;

mod overlay;

pub(crate) use overlay::PlayGizmoOverlaySnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayGizmoPointerOutcome {
    Unhandled,
    Hover {
        axis: Option<GizmoAxis>,
        changed: bool,
    },
    Began {
        axis: GizmoAxis,
    },
    Previewed {
        changed: bool,
    },
    Committed {
        changed: bool,
    },
    Cancelled,
}

impl PlayGizmoPointerOutcome {
    pub(crate) const fn consumed(self) -> bool {
        matches!(
            self,
            Self::Began { .. } | Self::Previewed { .. } | Self::Committed { .. } | Self::Cancelled
        )
    }

    pub(crate) const fn supersedes_scene_pick(self) -> bool {
        matches!(self, Self::Began { .. })
    }

    pub(crate) const fn presentation_changed(self) -> bool {
        match self {
            Self::Hover { changed, .. }
            | Self::Previewed { changed }
            | Self::Committed { changed } => changed,
            Self::Began { .. } | Self::Cancelled => true,
            Self::Unhandled => false,
        }
    }

    pub(crate) fn status_line(self) -> Option<String> {
        match self {
            Self::Hover {
                axis: Some(axis),
                changed: true,
            } => Some(format!("Hover Play gizmo axis {axis:?}")),
            Self::Began { axis } => Some(format!("Begin Play gizmo axis {axis:?}")),
            Self::Committed { changed: true } => Some("Committed Play transform".to_string()),
            Self::Cancelled => Some("Cancelled Play transform".to_string()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
struct PlayGizmoProjection {
    instance: PlayInstanceId,
    gateway: GatewaySessionIdentity,
    entity: u64,
    world_replacement_epoch: u64,
    frame_generation: u64,
    frame_size: (u32, u32),
    transform: Transform,
}

impl PlayGizmoProjection {
    fn matches(&self, frame: &PlayPreviewFrameIdentity, entity: u64) -> bool {
        self.instance == frame.instance()
            && self.gateway == *frame.gateway()
            && self.entity == entity
            && self.frame_generation == frame.generation()
            && self.frame_size == frame.size()
    }
}

#[derive(Clone, Debug)]
struct ActivePlayGizmoInteraction {
    projection: PlayGizmoProjection,
    interaction_id: u64,
    sequence: u64,
    initial: Transform,
    current: Transform,
}

pub(super) struct PlayGizmoInteractionController {
    next_interaction_id: Option<u64>,
    projection: Option<PlayGizmoProjection>,
    active: Option<ActivePlayGizmoInteraction>,
}

impl Default for PlayGizmoInteractionController {
    fn default() -> Self {
        Self {
            next_interaction_id: Some(FIRST_PLAY_GIZMO_INTERACTION_ID),
            projection: None,
            active: None,
        }
    }
}

impl PlayGizmoInteractionController {
    pub(super) fn invalidate_projection(&mut self) {
        if self.active.is_none() {
            self.projection = None;
        }
    }

    fn take_interaction_id(&mut self) -> Result<u64, PlayGizmoError> {
        let interaction_id = self
            .next_interaction_id
            .ok_or(PlayGizmoError::InteractionIdExhausted)?;
        self.next_interaction_id = interaction_id.checked_add(1);
        Ok(interaction_id)
    }

    fn next_sequence(active: &ActivePlayGizmoInteraction) -> Result<u64, PlayGizmoError> {
        active
            .sequence
            .checked_add(1)
            .ok_or(PlayGizmoError::SequenceExhausted)
    }

    fn retire_local(&mut self) {
        self.active = None;
        self.projection = None;
    }
}

impl EditorHostEventController {
    pub(crate) fn route_play_gizmo_pointer(
        &self,
        frame: Option<&PlayPreviewFrameIdentity>,
        kind: UiPointerEventKind,
        button: Option<UiPointerButton>,
        point: UiPoint,
    ) -> Result<PlayGizmoPointerOutcome, PlayGizmoError> {
        let Some(frame) = frame else {
            return Ok(PlayGizmoPointerOutcome::Unhandled);
        };
        let Some(gateway) = self.validate_play_gizmo_frame(frame)? else {
            return Ok(PlayGizmoPointerOutcome::Unhandled);
        };

        match (kind, button) {
            (UiPointerEventKind::Down, Some(UiPointerButton::Primary)) => {
                self.begin_play_gizmo(frame, &gateway, Vec2::new(point.x, point.y))
            }
            (UiPointerEventKind::Move, _) => {
                self.move_play_gizmo(frame, &gateway, Vec2::new(point.x, point.y))
            }
            (UiPointerEventKind::Up, Some(UiPointerButton::Primary)) => {
                self.commit_play_gizmo(frame, &gateway)
            }
            (UiPointerEventKind::Cancel, _) => self.cancel_play_gizmo(frame, &gateway),
            _ => Ok(PlayGizmoPointerOutcome::Unhandled),
        }
    }

    pub(super) fn retire_play_gizmo_local_state(&self) {
        self.play_gizmo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retire_local();
        let mut shell = self.shell().lock();
        shell.state.viewport_controller.cancel_interaction();
        shell
            .state
            .viewport_controller
            .set_handle_hover_for_transform(None);
    }

    fn begin_play_gizmo(
        &self,
        frame: &PlayPreviewFrameIdentity,
        gateway: &EditorRuntimeGatewayHandle,
        cursor: Vec2,
    ) -> Result<PlayGizmoPointerOutcome, PlayGizmoError> {
        let mut owner = self
            .play_gizmo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if owner.active.is_some() {
            return Err(PlayGizmoError::InteractionBusy);
        }
        let Some(entity) = self.active_play_gizmo_entity(frame.instance()) else {
            owner.projection = None;
            return Ok(PlayGizmoPointerOutcome::Unhandled);
        };
        let projection = capture_projection(gateway, frame, entity)?;
        let camera = self.play_gizmo_camera()?;
        let axis = {
            let shell = self.shell().lock();
            shell
                .state
                .viewport_controller
                .handle_axis_at_cursor_for_transform(
                    Some((entity, projection.transform)),
                    &camera,
                    cursor,
                )
        };
        let Some(axis) = axis else {
            owner.projection = Some(projection);
            return Ok(PlayGizmoPointerOutcome::Unhandled);
        };

        let interaction_id = owner.take_interaction_id()?;
        dispatch_transform(
            gateway,
            frame.gateway(),
            ZrRuntimeEditorTransformWriteV1::new(
                entity,
                interaction_id,
                FIRST_PLAY_GIZMO_SEQUENCE,
                projection.world_replacement_epoch,
                ZrRuntimeEditorTransformPhaseV1::Begin,
                projection.transform,
                projection.transform,
            ),
            "begin",
        )?;

        let began = self
            .shell()
            .lock()
            .state
            .viewport_controller
            .begin_handle_drag_for_transform(
                Some((entity, projection.transform)),
                &camera,
                cursor,
                axis,
            );
        if !began {
            let _ = dispatch_transform(
                gateway,
                frame.gateway(),
                ZrRuntimeEditorTransformWriteV1::new(
                    entity,
                    interaction_id,
                    FIRST_PLAY_GIZMO_SEQUENCE + 1,
                    projection.world_replacement_epoch,
                    ZrRuntimeEditorTransformPhaseV1::Cancel,
                    projection.transform,
                    projection.transform,
                ),
                "compensate failed local begin",
            );
            return Err(PlayGizmoError::LocalHandleRejected);
        }

        owner.projection = Some(projection.clone());
        owner.active = Some(ActivePlayGizmoInteraction {
            initial: projection.transform,
            current: projection.transform,
            projection,
            interaction_id,
            sequence: FIRST_PLAY_GIZMO_SEQUENCE,
        });
        zircon_runtime::profile_counter!("editor", "play.gizmo.begin_count", 1);
        Ok(PlayGizmoPointerOutcome::Began { axis })
    }

    fn move_play_gizmo(
        &self,
        frame: &PlayPreviewFrameIdentity,
        gateway: &EditorRuntimeGatewayHandle,
        cursor: Vec2,
    ) -> Result<PlayGizmoPointerOutcome, PlayGizmoError> {
        let mut owner = self
            .play_gizmo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(active) = owner.active.clone() {
            if let Err(error) = ensure_active_frame(&active, frame) {
                owner.retire_local();
                drop(owner);
                self.shell()
                    .lock()
                    .state
                    .viewport_controller
                    .cancel_interaction();
                return Err(error);
            }
            let camera = self.play_gizmo_camera()?;
            let preview = match self
                .shell()
                .lock()
                .state
                .viewport_controller
                .update_handle_drag_for_transform(&camera, cursor)
            {
                Some(preview) => preview,
                None => {
                    owner.retire_local();
                    return Err(PlayGizmoError::LocalHandleMissing);
                }
            };
            if preview.node_id != active.projection.entity {
                owner.retire_local();
                drop(owner);
                self.shell()
                    .lock()
                    .state
                    .viewport_controller
                    .cancel_interaction();
                return Err(PlayGizmoError::LocalHandleEntityMismatch);
            }
            if preview.transform == active.current {
                return Ok(PlayGizmoPointerOutcome::Previewed { changed: false });
            }
            let sequence = PlayGizmoInteractionController::next_sequence(&active)?;
            if let Err(error) = dispatch_transform(
                gateway,
                &active.projection.gateway,
                ZrRuntimeEditorTransformWriteV1::new(
                    active.projection.entity,
                    active.interaction_id,
                    sequence,
                    active.projection.world_replacement_epoch,
                    ZrRuntimeEditorTransformPhaseV1::Preview,
                    active.current,
                    preview.transform,
                ),
                "preview",
            ) {
                owner.retire_local();
                drop(owner);
                self.shell()
                    .lock()
                    .state
                    .viewport_controller
                    .cancel_interaction();
                return Err(error);
            }
            if let Some(current) = owner.active.as_mut() {
                current.current = preview.transform;
                current.sequence = sequence;
                current.projection.transform = preview.transform;
            }
            if let Some(projection) = owner.projection.as_mut() {
                projection.transform = preview.transform;
            }
            zircon_runtime::profile_counter!("editor", "play.gizmo.preview_write_count", 1);
            return Ok(PlayGizmoPointerOutcome::Previewed { changed: true });
        }

        let Some(entity) = self.active_play_gizmo_entity(frame.instance()) else {
            owner.projection = None;
            let changed = self
                .shell()
                .lock()
                .state
                .viewport_controller
                .set_handle_hover_for_transform(None);
            return Ok(PlayGizmoPointerOutcome::Hover {
                axis: None,
                changed,
            });
        };
        let projection = match owner
            .projection
            .as_ref()
            .filter(|projection| projection.matches(frame, entity))
        {
            Some(projection) => projection.clone(),
            None => capture_projection(gateway, frame, entity)?,
        };
        let camera = self.play_gizmo_camera()?;
        let mut shell = self.shell().lock();
        let axis = shell
            .state
            .viewport_controller
            .handle_axis_at_cursor_for_transform(
                Some((entity, projection.transform)),
                &camera,
                cursor,
            );
        let changed = shell
            .state
            .viewport_controller
            .set_handle_hover_for_transform(axis);
        owner.projection = Some(projection);
        Ok(PlayGizmoPointerOutcome::Hover { axis, changed })
    }

    fn commit_play_gizmo(
        &self,
        frame: &PlayPreviewFrameIdentity,
        gateway: &EditorRuntimeGatewayHandle,
    ) -> Result<PlayGizmoPointerOutcome, PlayGizmoError> {
        let active = {
            let owner = self
                .play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(active) = owner.active.clone() else {
                return Ok(PlayGizmoPointerOutcome::Unhandled);
            };
            if let Err(error) = ensure_active_frame(&active, frame) {
                drop(owner);
                self.retire_play_gizmo_local_state();
                return Err(error);
            }
            active
        };
        let sequence = PlayGizmoInteractionController::next_sequence(&active)?;
        if let Err(error) = dispatch_transform(
            gateway,
            &active.projection.gateway,
            ZrRuntimeEditorTransformWriteV1::new(
                active.projection.entity,
                active.interaction_id,
                sequence,
                active.projection.world_replacement_epoch,
                ZrRuntimeEditorTransformPhaseV1::Commit,
                active.current,
                active.current,
            ),
            "commit",
        ) {
            self.retire_play_gizmo_local_state();
            return Err(error);
        }

        self.shell()
            .lock()
            .state
            .viewport_controller
            .finish_handle_drag_for_transform();
        {
            let mut owner = self
                .play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            owner.active = None;
            owner.projection = Some(PlayGizmoProjection {
                transform: active.current,
                ..active.projection.clone()
            });
        }

        let changed = active.initial != active.current;
        if changed {
            let command = EditorCommand::applied_play_transform(
                active.projection.entity,
                active.interaction_id,
                active.projection.world_replacement_epoch,
                active.initial,
                active.current,
            )
            .expect("a changed Play transform produces a command");
            if let Err(error) = self
                .shell()
                .lock()
                .state
                .execute_gizmo_scene_command("Transform Play scene node", command)
            {
                let compensation = compensate_history_failure(gateway, &active);
                self.play_gizmo
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .invalidate_projection();
                return match compensation {
                    Ok(()) => Err(PlayGizmoError::History(error.to_string())),
                    Err(rollback) => Err(PlayGizmoError::RollbackFailed {
                        cause: error.to_string(),
                        rollback: rollback.to_string(),
                    }),
                };
            }
        }
        zircon_runtime::profile_counter!("editor", "play.gizmo.commit_count", 1);
        Ok(PlayGizmoPointerOutcome::Committed { changed })
    }

    fn cancel_play_gizmo(
        &self,
        frame: &PlayPreviewFrameIdentity,
        gateway: &EditorRuntimeGatewayHandle,
    ) -> Result<PlayGizmoPointerOutcome, PlayGizmoError> {
        let active = {
            let owner = self
                .play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(active) = owner.active.clone() else {
                return Ok(PlayGizmoPointerOutcome::Unhandled);
            };
            if let Err(error) = ensure_active_frame(&active, frame) {
                drop(owner);
                self.retire_play_gizmo_local_state();
                return Err(error);
            }
            active
        };
        let sequence = PlayGizmoInteractionController::next_sequence(&active)?;
        let dispatch = dispatch_transform(
            gateway,
            &active.projection.gateway,
            ZrRuntimeEditorTransformWriteV1::new(
                active.projection.entity,
                active.interaction_id,
                sequence,
                active.projection.world_replacement_epoch,
                ZrRuntimeEditorTransformPhaseV1::Cancel,
                active.current,
                active.initial,
            ),
            "cancel",
        );
        self.retire_play_gizmo_local_state();
        dispatch?;
        zircon_runtime::profile_counter!("editor", "play.gizmo.cancel_count", 1);
        Ok(PlayGizmoPointerOutcome::Cancelled)
    }

    fn validate_play_gizmo_frame(
        &self,
        frame: &PlayPreviewFrameIdentity,
    ) -> Result<Option<EditorRuntimeGatewayHandle>, PlayGizmoError> {
        if !matches!(
            self.play_sessions().mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) {
            return Ok(None);
        }
        let domain = WorldDomain::Play(frame.instance());
        if self.play_sessions().attached_world_domain() != Some(domain)
            || frame.gateway().play_instance() != Some(frame.instance().raw())
        {
            return Err(PlayGizmoError::FrameInstanceMismatch);
        }
        let gateway = self
            .gateway_for(domain)
            .ok_or(PlayGizmoError::GatewayUnavailable)?;
        if gateway.identity() != *frame.gateway() {
            return Err(PlayGizmoError::FrameGatewayStale);
        }
        Ok(Some(gateway))
    }

    fn active_play_gizmo_entity(&self, instance: PlayInstanceId) -> Option<u64> {
        let shell = self.shell().lock();
        (shell.state.is_playing()
            && shell.state.viewport_controller.selection().active_domain()
                == WorldDomain::Play(instance))
        .then(|| shell.state.viewport_controller.selection().active_primary())
        .flatten()
    }

    fn play_gizmo_camera(
        &self,
    ) -> Result<crate::scene::viewport::ViewportCameraSnapshot, PlayGizmoError> {
        self.shell()
            .lock()
            .state
            .viewport_camera_snapshot()
            .map_err(|error| PlayGizmoError::Camera(error.to_string()))?
            .ok_or(PlayGizmoError::CameraUnavailable)
    }
}

fn capture_projection(
    gateway: &EditorRuntimeGatewayHandle,
    frame: &PlayPreviewFrameIdentity,
    entity: u64,
) -> Result<PlayGizmoProjection, PlayGizmoError> {
    match gateway
        .query_world_at_identity(frame.gateway(), WorldQuery::transform_snapshot(entity))
        .map_err(|source| PlayGizmoError::Gateway {
            phase: "capture transform",
            source,
        })? {
        WorldQueryResult::TransformSnapshot {
            entity: observed,
            world_replacement_epoch,
            transform,
            ..
        } if observed == entity && world_replacement_epoch != 0 => Ok(PlayGizmoProjection {
            instance: frame.instance(),
            gateway: frame.gateway().clone(),
            entity,
            world_replacement_epoch,
            frame_generation: frame.generation(),
            frame_size: frame.size(),
            transform,
        }),
        WorldQueryResult::EntityMissing { .. } => Err(PlayGizmoError::EntityMissing { entity }),
        WorldQueryResult::TransformSnapshot { .. } => Err(PlayGizmoError::ProjectionMismatch),
        _ => Err(PlayGizmoError::ProjectionKindMismatch),
    }
}

fn compensate_history_failure(
    gateway: &EditorRuntimeGatewayHandle,
    active: &ActivePlayGizmoInteraction,
) -> Result<(), PlayGizmoError> {
    let observed = gateway
        .query_world_at_identity(
            &active.projection.gateway,
            WorldQuery::transform_snapshot(active.projection.entity),
        )
        .map_err(|source| PlayGizmoError::Gateway {
            phase: "capture history rollback state",
            source,
        })?;
    let WorldQueryResult::TransformSnapshot {
        entity,
        world_replacement_epoch,
        transform,
        ..
    } = observed
    else {
        return Err(PlayGizmoError::ProjectionKindMismatch);
    };
    if entity != active.projection.entity
        || world_replacement_epoch != active.projection.world_replacement_epoch
    {
        return Err(PlayGizmoError::ProjectionMismatch);
    }
    if transform == active.initial {
        return Ok(());
    }
    if transform != active.current {
        return Err(PlayGizmoError::HistoryRollbackDiverged);
    }
    dispatch_transform(
        gateway,
        &active.projection.gateway,
        ZrRuntimeEditorTransformWriteV1::new(
            active.projection.entity,
            active.interaction_id,
            FIRST_PLAY_GIZMO_SEQUENCE,
            active.projection.world_replacement_epoch,
            ZrRuntimeEditorTransformPhaseV1::Apply,
            active.current,
            active.initial,
        ),
        "compensate history failure",
    )
}

fn dispatch_transform(
    gateway: &EditorRuntimeGatewayHandle,
    identity: &GatewaySessionIdentity,
    request: ZrRuntimeEditorTransformWriteV1,
    phase: &'static str,
) -> Result<(), PlayGizmoError> {
    gateway
        .handle_event_at_identity(
            identity,
            ZrRuntimeEventV1::editor_transform_write(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                &request,
            ),
        )
        .map_err(|source| PlayGizmoError::Gateway { phase, source })
}

fn ensure_active_frame(
    active: &ActivePlayGizmoInteraction,
    frame: &PlayPreviewFrameIdentity,
) -> Result<(), PlayGizmoError> {
    if active.projection.instance != frame.instance()
        || active.projection.gateway != *frame.gateway()
    {
        return Err(PlayGizmoError::InteractionFrameReplaced);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum PlayGizmoError {
    #[error("displayed SIE frame belongs to another Play instance")]
    FrameInstanceMismatch,
    #[error("displayed SIE frame belongs to a retired runtime gateway")]
    FrameGatewayStale,
    #[error("SIE Play gateway is unavailable")]
    GatewayUnavailable,
    #[error("Play Gizmo interaction id space is exhausted")]
    InteractionIdExhausted,
    #[error("Play Gizmo sequence space is exhausted")]
    SequenceExhausted,
    #[error("another Play Gizmo interaction is active")]
    InteractionBusy,
    #[error("the local HandleTool rejected a runtime-owned transform")]
    LocalHandleRejected,
    #[error("the local Play HandleTool session is missing")]
    LocalHandleMissing,
    #[error("the local Play HandleTool changed entity ownership")]
    LocalHandleEntityMismatch,
    #[error("the active Play Gizmo frame was replaced")]
    InteractionFrameReplaced,
    #[error("runtime transform projection for entity {entity} is missing")]
    EntityMissing { entity: u64 },
    #[error("runtime transform projection returned invalid identity")]
    ProjectionMismatch,
    #[error("runtime transform query returned another projection kind")]
    ProjectionKindMismatch,
    #[error("Play Gizmo history rollback left a divergent runtime transform")]
    HistoryRollbackDiverged,
    #[error("editor camera is unavailable for Play Gizmo projection")]
    CameraUnavailable,
    #[error("capture editor camera for Play Gizmo: {0}")]
    Camera(String),
    #[error("Play Gizmo runtime {phase} failed: {source}")]
    Gateway {
        phase: &'static str,
        #[source]
        source: GatewayError,
    },
    #[error("record committed Play Gizmo history: {0}")]
    History(String),
    #[error(
        "record Play Gizmo history failed ({cause}); runtime rollback also failed ({rollback})"
    )]
    RollbackFailed { cause: String, rollback: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_and_sequence_ids_do_not_wrap() {
        let mut owner = PlayGizmoInteractionController {
            next_interaction_id: Some(u64::MAX),
            projection: None,
            active: None,
        };

        assert_eq!(owner.take_interaction_id().unwrap(), u64::MAX);
        assert!(matches!(
            owner.take_interaction_id(),
            Err(PlayGizmoError::InteractionIdExhausted)
        ));
    }

    #[test]
    fn hover_never_consumes_the_scene_pick_path() {
        assert!(!PlayGizmoPointerOutcome::Hover {
            axis: Some(GizmoAxis::X),
            changed: true,
        }
        .consumed());
        assert!(PlayGizmoPointerOutcome::Began { axis: GizmoAxis::X }.consumed());
    }
}
