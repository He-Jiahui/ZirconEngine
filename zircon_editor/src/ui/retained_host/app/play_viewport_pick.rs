use std::time::{Duration, Instant};

use thiserror::Error;
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::{
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickTicket, ZrRuntimeViewportPixelV1,
    ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

use crate::core::gateway::{EditorRuntimeViewportPickRoute, GatewayError};
use crate::core::play::{
    PlayInstanceId, PlayKind, PlayMode, PlayPreviewFrameIdentity, WorldDomain,
};
use crate::scene::selection::SelectionMutation;
use crate::ui::host::EditorHostEventController;

use super::*;

const FIRST_PLAY_VIEWPORT_PICK_INPUT_SEQUENCE: u64 = 1;
const PLAY_VIEWPORT_PICK_POLL_INTERVAL: Duration = Duration::from_millis(8);
const PLAY_VIEWPORT_PICK_ERROR_RETRY_INTERVAL: Duration = Duration::from_millis(50);

pub(super) struct PlayViewportPickConsumer {
    next_input_sequence: Option<u64>,
    pending: Option<PendingPlayViewportPick>,
}

struct PendingPlayViewportPick {
    instance: PlayInstanceId,
    route: EditorRuntimeViewportPickRoute,
    request: ZrRuntimeViewportPickRequestV1,
    ticket: ZrRuntimeViewportPickTicket,
    mutation: SelectionMutation,
    apply_selection: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayViewportPickPoll {
    Idle,
    Pending,
    Selection { entity: Option<u64>, changed: bool },
    Discarded(ZrRuntimeViewportPickDispositionV1),
    Terminal(ZrRuntimeViewportPickDispositionV1),
    Retired,
}

#[derive(Debug, Error)]
pub(super) enum PlayViewportPickError {
    #[error("displayed SIE frame belongs to a different Play instance")]
    FrameInstanceMismatch,
    #[error("displayed SIE frame has no current runtime gateway")]
    GatewayUnavailable,
    #[error("SIE viewport pick input sequence is exhausted")]
    InputSequenceExhausted,
    #[error("SIE viewport point is outside the displayed frame")]
    PixelOutsideFrame,
    #[error("SIE viewport pick {phase} failed: {source}")]
    Gateway {
        phase: &'static str,
        #[source]
        source: GatewayError,
    },
}

impl Default for PlayViewportPickConsumer {
    fn default() -> Self {
        Self {
            next_input_sequence: Some(FIRST_PLAY_VIEWPORT_PICK_INPUT_SEQUENCE),
            pending: None,
        }
    }
}

impl PlayViewportPickConsumer {
    pub(super) fn request(
        &mut self,
        runtime: &EditorHostEventController,
        frame: &PlayPreviewFrameIdentity,
        point: UiPoint,
        mutation: SelectionMutation,
    ) -> Result<bool, PlayViewportPickError> {
        if !matches!(
            runtime.play_sessions().mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) {
            return Ok(false);
        }

        let instance = frame.instance();
        if runtime.play_sessions().attached_world_domain() != Some(WorldDomain::Play(instance))
            || frame.gateway().play_instance() != Some(instance.raw())
        {
            return Err(PlayViewportPickError::FrameInstanceMismatch);
        }
        let pixel =
            viewport_pixel(point, frame.size()).ok_or(PlayViewportPickError::PixelOutsideFrame)?;
        let gateway = runtime
            .gateway_for(WorldDomain::Play(instance))
            .ok_or(PlayViewportPickError::GatewayUnavailable)?;
        let route = EditorRuntimeViewportPickRoute::capture_at_identity(&gateway, frame.gateway())
            .map_err(|source| PlayViewportPickError::Gateway {
                phase: "route capture",
                source,
            })?;

        self.cancel_for_replacement()?;
        let input_sequence = self.take_input_sequence()?;
        let request = ZrRuntimeViewportPickRequestV1::new(
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ZrRuntimeViewportSizeV1::new(frame.size().0, frame.size().1),
            pixel,
            frame.generation(),
            input_sequence,
            ZrRuntimeViewportPickPurposeV1::Press,
            0,
        );
        let ticket = route.request_viewport_pick(request).map_err(|source| {
            PlayViewportPickError::Gateway {
                phase: "request",
                source,
            }
        })?;
        self.pending = Some(PendingPlayViewportPick {
            instance,
            route,
            request,
            ticket,
            mutation,
            apply_selection: true,
        });
        zircon_runtime::profile_counter!("editor", "play.viewport_pick.request_count", 1);
        Ok(true)
    }

    pub(super) fn poll(
        &mut self,
        runtime: &EditorHostEventController,
    ) -> Result<PlayViewportPickPoll, PlayViewportPickError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(PlayViewportPickPoll::Idle);
        };
        let domain = WorldDomain::Play(pending.instance);
        let still_current = matches!(
            runtime.play_sessions().mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) && runtime.play_sessions().attached_world_domain() == Some(domain)
            && runtime.world_gateway_identity(domain).as_ref() == Some(pending.route.identity());
        if !still_current {
            self.retire_pending()?;
            return Ok(PlayViewportPickPoll::Retired);
        }

        let result = pending
            .route
            .poll_viewport_pick(pending.ticket, pending.request)
            .map_err(|source| PlayViewportPickError::Gateway {
                phase: "poll",
                source,
            })?;
        let disposition = result
            .disposition()
            .expect("gateway route validates viewport-pick completions");
        if disposition == ZrRuntimeViewportPickDispositionV1::Pending {
            zircon_runtime::profile_counter!("editor", "play.viewport_pick.pending_poll_count", 1);
            return Ok(PlayViewportPickPoll::Pending);
        }

        let pending = self
            .pending
            .take()
            .expect("a terminal viewport pick retains its local owner");
        if !pending.apply_selection {
            zircon_runtime::profile_counter!(
                "editor",
                "play.viewport_pick.discarded_completion_count",
                1
            );
            return Ok(PlayViewportPickPoll::Discarded(disposition));
        }
        match disposition {
            ZrRuntimeViewportPickDispositionV1::Hit | ZrRuntimeViewportPickDispositionV1::NoHit => {
                let entity = (disposition == ZrRuntimeViewportPickDispositionV1::Hit)
                    .then_some(result.entity);
                let Some(changed) = runtime.apply_play_viewport_pick_selection(
                    pending.instance,
                    entity,
                    pending.mutation,
                ) else {
                    return Ok(PlayViewportPickPoll::Retired);
                };
                let counter = if entity.is_some() {
                    "play.viewport_pick.hit_count"
                } else {
                    "play.viewport_pick.no_hit_count"
                };
                zircon_runtime::profile_counter!("editor", counter, 1);
                Ok(PlayViewportPickPoll::Selection { entity, changed })
            }
            terminal => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "play.viewport_pick.non_selection_terminal_count",
                    1
                );
                Ok(PlayViewportPickPoll::Terminal(terminal))
            }
        }
    }

    pub(super) fn cancel(&mut self) -> Result<bool, PlayViewportPickError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(false);
        };
        let cancellation = pending.route.cancel_viewport_pick(pending.ticket);
        match cancellation {
            Ok(()) => {
                self.pending = None;
                zircon_runtime::profile_counter!("editor", "play.viewport_pick.cancel_count", 1);
                Ok(true)
            }
            Err(source) => {
                self.suppress_pending_selection();
                Err(PlayViewportPickError::Gateway {
                    phase: "cancel",
                    source,
                })
            }
        }
    }

    pub(super) fn next_poll_deadline() -> Instant {
        Instant::now()
            .checked_add(PLAY_VIEWPORT_PICK_POLL_INTERVAL)
            .unwrap_or_else(Instant::now)
    }

    pub(super) fn next_error_retry_deadline() -> Instant {
        Instant::now()
            .checked_add(PLAY_VIEWPORT_PICK_ERROR_RETRY_INTERVAL)
            .unwrap_or_else(Instant::now)
    }

    fn cancel_for_replacement(&mut self) -> Result<(), PlayViewportPickError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(());
        };
        let cancellation = pending.route.cancel_viewport_pick(pending.ticket);
        match cancellation {
            Ok(()) => {
                self.pending = None;
                zircon_runtime::profile_counter!("editor", "play.viewport_pick.replaced_count", 1);
                Ok(())
            }
            Err(source) => {
                self.suppress_pending_selection();
                Err(PlayViewportPickError::Gateway {
                    phase: "replacement cancellation",
                    source,
                })
            }
        }
    }

    fn retire_pending(&mut self) -> Result<(), PlayViewportPickError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(());
        };
        let cancellation = pending.route.cancel_viewport_pick(pending.ticket);
        match cancellation {
            Ok(()) => {
                self.pending = None;
                zircon_runtime::profile_counter!("editor", "play.viewport_pick.retired_count", 1);
                Ok(())
            }
            Err(source) => {
                self.suppress_pending_selection();
                Err(PlayViewportPickError::Gateway {
                    phase: "retirement cancellation",
                    source,
                })
            }
        }
    }

    fn suppress_pending_selection(&mut self) {
        if let Some(pending) = self.pending.as_mut() {
            pending.apply_selection = false;
        }
    }

    fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    fn take_input_sequence(&mut self) -> Result<u64, PlayViewportPickError> {
        let sequence = self
            .next_input_sequence
            .ok_or(PlayViewportPickError::InputSequenceExhausted)?;
        self.next_input_sequence = sequence.checked_add(1);
        Ok(sequence)
    }
}

fn viewport_pixel(point: UiPoint, size: (u32, u32)) -> Option<ZrRuntimeViewportPixelV1> {
    let (width, height) = size;
    if width == 0
        || height == 0
        || !point.x.is_finite()
        || !point.y.is_finite()
        || point.x < 0.0
        || point.y < 0.0
        || point.x >= width as f32
        || point.y >= height as f32
    {
        return None;
    }
    Some(ZrRuntimeViewportPixelV1::new(
        point.x.floor() as u32,
        point.y.floor() as u32,
    ))
}

impl RetainedEditorHost {
    pub(super) fn poll_play_viewport_pick_for_native_host(&mut self) {
        match self.play_viewport_pick.poll(&self.runtime) {
            Ok(PlayViewportPickPoll::Idle | PlayViewportPickPoll::Retired) => {}
            Ok(PlayViewportPickPoll::Pending) => self
                .ui
                .set_lifecycle_frame_update(Some(PlayViewportPickConsumer::next_poll_deadline())),
            Ok(PlayViewportPickPoll::Selection { entity, changed }) => {
                if changed {
                    match entity {
                        Some(entity) => {
                            self.set_status_line(format!("Selected runtime entity {entity}"))
                        }
                        None => self.set_status_line("Cleared runtime selection"),
                    }
                }
            }
            Ok(PlayViewportPickPoll::Discarded(disposition)) => self.set_status_line(format!(
                "Discarded a retired runtime viewport pick completion: {disposition:?}"
            )),
            Ok(PlayViewportPickPoll::Terminal(disposition)) => self.set_status_line(format!(
                "Runtime viewport pick finished without a selection: {disposition:?}"
            )),
            Err(error) => {
                self.set_status_line(error.to_string());
                if self.play_viewport_pick.has_pending() {
                    self.ui.set_lifecycle_frame_update(Some(
                        PlayViewportPickConsumer::next_error_retry_deadline(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_pixel_mapping_floors_inside_points_and_rejects_edges() {
        assert_eq!(
            viewport_pixel(UiPoint::new(12.75, 24.25), (100, 50)),
            Some(ZrRuntimeViewportPixelV1::new(12, 24))
        );
        assert!(viewport_pixel(UiPoint::new(-0.01, 0.0), (100, 50)).is_none());
        assert!(viewport_pixel(UiPoint::new(100.0, 0.0), (100, 50)).is_none());
        assert!(viewport_pixel(UiPoint::new(0.0, 50.0), (100, 50)).is_none());
        assert!(viewport_pixel(UiPoint::new(f32::NAN, 0.0), (100, 50)).is_none());
    }

    #[test]
    fn input_sequence_never_wraps_or_reuses_an_identity() {
        let mut consumer = PlayViewportPickConsumer {
            next_input_sequence: Some(u64::MAX),
            pending: None,
        };

        assert_eq!(consumer.take_input_sequence().unwrap(), u64::MAX);
        assert!(matches!(
            consumer.take_input_sequence(),
            Err(PlayViewportPickError::InputSequenceExhausted)
        ));
    }

    #[test]
    fn failed_cancellation_paths_keep_the_pending_owner_until_runtime_acknowledges_it() {
        let source = include_str!("play_viewport_pick.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(!production.contains("let Some(pending) = self.pending.take() else"));
        assert!(production.contains("self.suppress_pending_selection();"));
        assert!(production.contains("discarded_completion_count"));
        assert!(production.contains("next_error_retry_deadline"));
    }
}
