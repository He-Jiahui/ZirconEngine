use std::{
    collections::{BTreeSet, HashMap},
    time::{Duration, Instant},
};

use zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest;
use zircon_runtime_interface::ui::dispatch::{
    UiAccessibilityInputEvent, UiClipboardInputEvent, UiInputEvent, UiInputEventMetadata,
    UiInputSequence, UiInputTimestamp, UiPointerEvent, UiPointerInputEvent, UiPointerSource,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::input_publication::{RuntimeUiInputQueryAdmission, RuntimeUiInputQueryRejectReason};
use super::{
    ui_size, RuntimeUiSurface, RuntimeUiSurfaceSet, NODE_ID_LOCAL_MASK, NODE_ID_SURFACE_SHIFT,
};

impl RuntimeUiSurfaceSet {
    pub(in crate::dynamic_api::session) fn next_input_timer_delay(&self) -> Option<Duration> {
        self.next_input_timer_delay_at(self.input_clock.now())
    }

    pub(super) fn next_input_timer_delay_at(&self, now: UiInputTimestamp) -> Option<Duration> {
        self.surfaces
            .iter()
            .filter_map(|surface| surface.input.next_frame_visible_delay(now))
            .min()
    }

    pub(in crate::dynamic_api::session) fn tick_input_timers(&mut self) -> Result<(), UiTreeError> {
        self.tick_input_timers_at(self.input_clock.now())
    }

    pub(super) fn tick_input_timers_at(
        &mut self,
        now: UiInputTimestamp,
    ) -> Result<(), UiTreeError> {
        for surface_index in 0..self.surfaces.len() {
            let results = {
                let runtime_surface = &mut self.surfaces[surface_index];
                runtime_surface
                    .input
                    .tick(&mut runtime_surface.surface, now)?
            };
            for result in results {
                self.record_dispatch_outputs(surface_index, &result);
            }
        }
        Ok(())
    }

    pub(in crate::dynamic_api::session) fn dispatch_clipboard_result(
        &mut self,
        target_surface: u32,
        transfer_id: zircon_runtime_interface::ui::dispatch::UiClipboardTransferId,
        owner: UiNodeId,
        outcome: zircon_runtime_interface::ui::dispatch::UiClipboardTransferOutcome,
    ) -> Result<bool, UiTreeError> {
        let metadata = self.next_input_metadata();
        let Ok(target_surface) = usize::try_from(target_surface) else {
            return Ok(false);
        };
        let Some(runtime_surface) = self.surfaces.get_mut(target_surface) else {
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        let result = runtime_surface.surface.dispatch_input_event_with_manager(
            &mut runtime_surface.input,
            UiInputEvent::Clipboard(UiClipboardInputEvent {
                metadata,
                transfer_id,
                owner,
                outcome,
            }),
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        self.update_focused_surface_after_dispatch(target_surface, focus_before, focus_after);
        self.record_dispatch_outputs(target_surface, &result);
        Ok(true)
    }

    pub(in crate::dynamic_api::session) fn dispatch_accessibility_action(
        &mut self,
        mut request: UiAccessibilityActionRequest,
    ) -> Result<bool, UiTreeError> {
        let Some((surface_index, local_node_id)) = split_global_node_id(request.target) else {
            return Ok(false);
        };
        let metadata = self.next_input_metadata();
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            return Ok(false);
        };
        request.target = local_node_id;
        let focus_before = runtime_surface.surface.focus.focused;
        let result = runtime_surface.surface.dispatch_input_event_with_manager(
            &mut runtime_surface.input,
            UiInputEvent::Accessibility(UiAccessibilityInputEvent { metadata, request }),
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        let handled = result.reply.stops_propagation();
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
    }

    pub(in crate::dynamic_api::session) fn dispatch_input(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        mut event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        if matches!(&event, UiInputEvent::MouseMotion(_)) {
            crate::profile_counter!("runtime", "ui.surface_set.input.unrouted_reject_count", 1);
            return Ok(false);
        }
        stamp_input_event(&mut event, self.input_clock.now());
        let root_size = ui_size(viewport_size);
        if input_requires_focus_owner(&event) {
            let Some(surface_index) = self.focused_surface.filter(|surface_index| {
                self.surfaces
                    .get(*surface_index)
                    .is_some_and(|surface| surface.surface.focus.focused.is_some())
            }) else {
                if let Some(stale_surface) = self.focused_surface.take() {
                    self.focused_surfaces.remove(&stale_surface);
                }
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.focus_owner_miss_count",
                    1
                );
                return Ok(false);
            };
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.focus_direct_route_count",
                1
            );
            return self.dispatch_input_to_surface(surface_index, root_size, event, false);
        }
        if input_requires_navigation_owner(&event) {
            let Some(surface_index) = self.navigation_surface.filter(|surface_index| {
                self.surfaces.get(*surface_index).is_some_and(|surface| {
                    surface.surface.focus.focused.is_some()
                        || surface.surface.has_navigation_candidate()
                })
            }) else {
                self.navigation_surface = None;
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.navigation_owner_miss_count",
                    1
                );
                return Ok(false);
            };
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.navigation_direct_route_count",
                1
            );
            return self.dispatch_input_to_surface(surface_index, root_size, event, false);
        }
        let mut event = Some(event);
        for surface_index in (0..self.surfaces.len()).rev() {
            let Some(event) = input_event_for_surface(&mut event, surface_index == 0) else {
                return Ok(false);
            };
            if self.dispatch_input_to_surface(surface_index, root_size, event, true)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn dispatch_input_to_surface(
        &mut self,
        surface_index: usize,
        root_size: UiSize,
        event: UiInputEvent,
        rebuild_before_dispatch: bool,
    ) -> Result<bool, UiTreeError> {
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            self.focused_surfaces.remove(&surface_index);
            if self.focused_surface == Some(surface_index) {
                self.focused_surface = self.focused_surfaces.last().copied();
            }
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        if rebuild_before_dispatch {
            runtime_surface.rebuild_dirty(root_size)?;
        }
        let result = runtime_surface
            .surface
            .dispatch_input_event_with_manager(&mut runtime_surface.input, event)?;
        let focus_after = runtime_surface.surface.focus.focused;
        let handled = result.reply.stops_propagation();
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
    }

    pub(in crate::dynamic_api::session) fn dispatch_pointer(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: Option<UiPointerButton>,
        pointer_id: Option<u64>,
        pointer_source: UiPointerSource,
        scroll_delta: f32,
    ) -> Result<bool, UiTreeError> {
        let mut event = UiPointerEvent::new(kind, point).with_scroll_delta(scroll_delta);
        if let Some(button) = button {
            event = event.with_button(button);
        }
        let mut metadata = self.next_input_metadata();
        metadata.pointer_id =
            pointer_id.map(zircon_runtime_interface::ui::dispatch::UiPointerId::new);
        metadata.pointer_source = pointer_source;
        self.dispatch_pointer_input(
            viewport_size,
            pointer_id,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata,
                event,
                precise_scroll: None,
            }),
        )
    }

    fn dispatch_pointer_input(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        pointer_id: Option<u64>,
        event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        let UiInputEvent::Pointer(pointer) = &event else {
            return Ok(false);
        };
        let kind = pointer.event.kind;
        let point = pointer.event.point;
        let previous_point = if kind == UiPointerEventKind::Down {
            point
        } else {
            self.pointer_positions
                .get(&pointer_id)
                .copied()
                .unwrap_or(point)
        };
        let root_size = ui_size(viewport_size);
        let query_admission = self
            .input_publication
            .query(viewport_size, point, previous_point);
        if kind == UiPointerEventKind::Down {
            self.pointer_capture_surfaces.remove(&pointer_id);
        }
        let published_query = match query_admission {
            RuntimeUiInputQueryAdmission::Published(query) => Some(query),
            RuntimeUiInputQueryAdmission::Unpublished => None,
            RuntimeUiInputQueryAdmission::Rejected(reason) => {
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.invalid_pointer_reject_count",
                    1
                );
                match reason {
                    RuntimeUiInputQueryRejectReason::NonFinitePointer => {
                        crate::profile_counter!(
                            "runtime",
                            "ui.surface_set.input.non_finite_pointer_reject_count",
                            1
                        );
                    }
                    RuntimeUiInputQueryRejectReason::DegenerateViewport => {
                        crate::profile_counter!(
                            "runtime",
                            "ui.surface_set.input.degenerate_viewport_reject_count",
                            1
                        );
                    }
                    RuntimeUiInputQueryRejectReason::AffineProjectionOverflow => {
                        crate::profile_counter!(
                            "runtime",
                            "ui.surface_set.input.affine_projection_reject_count",
                            1
                        )
                    }
                }
                if matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel) {
                    self.pointer_positions.remove(&pointer_id);
                    self.pointer_capture_surfaces.remove(&pointer_id);
                }
                return Ok(false);
            }
        };
        if pointer_id.is_some()
            && matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel)
        {
            self.pointer_positions.remove(&pointer_id);
        } else {
            self.pointer_positions.insert(pointer_id, point);
        }
        if let Some(surface_index) =
            capture_surface_for_event(&self.pointer_capture_surfaces, pointer_id, kind)
        {
            return self.dispatch_pointer_to_surface(
                surface_index,
                root_size,
                pointer_id,
                kind,
                event,
                published_query.map(|query| query.hit_test_query()),
                false,
            );
        }
        if let Some(query) = published_query {
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.candidate_surface_count",
                query.candidate_count()
            );
            let mut event = Some(event);
            for candidate_offset in 0..query.candidate_count() {
                let Some(surface_index) = self
                    .input_publication
                    .candidate_surface(query, candidate_offset)
                else {
                    break;
                };
                let Some(event) = input_event_for_surface(
                    &mut event,
                    candidate_offset.saturating_add(1) == query.candidate_count(),
                ) else {
                    return Ok(false);
                };
                if self.dispatch_pointer_to_surface(
                    surface_index,
                    root_size,
                    pointer_id,
                    kind,
                    event,
                    Some(query.hit_test_query()),
                    false,
                )? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_unavailable_fallback_count",
            1
        );
        let mut event = Some(event);
        for surface_index in (0..self.surfaces.len()).rev() {
            let Some(event) = input_event_for_surface(&mut event, surface_index == 0) else {
                return Ok(false);
            };
            let result = self.dispatch_pointer_to_surface(
                surface_index,
                root_size,
                pointer_id,
                kind,
                event,
                None,
                true,
            )?;
            if result {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn dispatch_pointer_to_surface(
        &mut self,
        surface_index: usize,
        root_size: UiSize,
        pointer_id: Option<u64>,
        kind: UiPointerEventKind,
        event: UiInputEvent,
        pointer_query: Option<UiHitTestQuery>,
        rebuild_before_dispatch: bool,
    ) -> Result<bool, UiTreeError> {
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            self.pointer_capture_surfaces.remove(&pointer_id);
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        if rebuild_before_dispatch {
            runtime_surface.rebuild_dirty(root_size)?;
        }
        let result = runtime_surface.input.dispatch_input_event_with_query(
            &mut runtime_surface.surface,
            event,
            pointer_query,
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        update_capture_surface(
            &mut self.pointer_capture_surfaces,
            pointer_id,
            surface_index,
            kind,
            result
                .pointer_routing
                .as_ref()
                .and_then(|routing| routing.capture_target)
                .is_some(),
        );
        let handled = result.reply.stops_propagation();
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
    }

    pub(super) fn refresh_input_owners_from_publication(&mut self) {
        self.focused_surfaces = published_focused_surfaces(&self.surfaces);
        if !self
            .focused_surface
            .is_some_and(|surface_index| self.focused_surfaces.contains(&surface_index))
        {
            self.focused_surface = self.focused_surfaces.last().copied();
        }
        self.navigation_surface =
            published_navigation_surface(&self.surfaces, self.focused_surface);
    }

    pub(super) fn publish_input_authority(&mut self, viewport_size: crate::core::math::UVec2) {
        let report = self.input_publication.publish(
            viewport_size,
            self.surfaces.len(),
            self.surfaces
                .iter()
                .map(|surface| surface.surface.surface_frame()),
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_full_rebuild_count",
            report.full_rebuild as usize
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_patch_surface_count",
            report.patched_surface_count
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_visited_entry_count",
            report.visited_entry_count
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_cell_membership_count",
            report.cell_membership_count
        );
    }

    pub(in crate::dynamic_api::session::runtime_ui) fn update_focused_surface_after_dispatch(
        &mut self,
        surface_index: usize,
        focus_before: Option<UiNodeId>,
        focus_after: Option<UiNodeId>,
    ) {
        if focus_before != focus_after {
            if focus_after.is_some() {
                self.focused_surfaces.insert(surface_index);
                self.focused_surface = Some(surface_index);
                self.navigation_surface = Some(surface_index);
            } else {
                self.focused_surfaces.remove(&surface_index);
                if self.focused_surface == Some(surface_index) {
                    self.focused_surface = self.focused_surfaces.last().copied();
                }
                if self.navigation_surface == Some(surface_index)
                    && !self
                        .surfaces
                        .get(surface_index)
                        .is_some_and(|surface| surface.surface.has_navigation_candidate())
                {
                    self.navigation_surface = None;
                }
            }
        }
    }

    pub(in crate::dynamic_api::session) fn next_input_metadata(&mut self) -> UiInputEventMetadata {
        self.input_sequence = self.input_sequence.saturating_add(1);
        UiInputEventMetadata::new(
            self.input_clock.now(),
            UiInputSequence::new(self.input_sequence),
        )
    }

    fn record_dispatch_outputs(
        &mut self,
        surface_index: usize,
        result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    ) {
        if result.host_requests.is_empty() && result.component_events.is_empty() {
            return;
        }
        let Some(runtime_surface) = self.surfaces.get(surface_index) else {
            return;
        };
        let Ok(target_surface) = u32::try_from(surface_index) else {
            return;
        };
        let tree_id = runtime_surface.surface.tree.tree_id.clone();
        self.host_requests
            .record_result(target_surface, &tree_id, result);
        let revoked = self
            .action_requests
            .record_result(target_surface, &tree_id, result);
        if revoked.is_empty() {
            return;
        }
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            return;
        };
        for reference in revoked {
            runtime_surface.surface.revoke_secure_text_value(&reference);
        }
    }
}

pub(super) fn ui_input_timestamp_at(origin: Instant, now: Instant) -> UiInputTimestamp {
    let elapsed_micros = now
        .saturating_duration_since(origin)
        .as_micros()
        .min(u64::MAX as u128) as u64;
    UiInputTimestamp::from_micros(elapsed_micros.saturating_add(1))
}

fn stamp_input_event(event: &mut UiInputEvent, timestamp: UiInputTimestamp) {
    let metadata = match event {
        UiInputEvent::Pointer(input) => &mut input.metadata,
        UiInputEvent::Keyboard(input) => &mut input.metadata,
        UiInputEvent::Text(input) => &mut input.metadata,
        UiInputEvent::Ime(input) => &mut input.metadata,
        UiInputEvent::Clipboard(input) => &mut input.metadata,
        UiInputEvent::Navigation(input) => &mut input.metadata,
        UiInputEvent::Analog(input) => &mut input.metadata,
        UiInputEvent::MouseMotion(input) => &mut input.metadata,
        UiInputEvent::DragDrop(input) => &mut input.metadata,
        UiInputEvent::Popup(input) => &mut input.metadata,
        UiInputEvent::TooltipTimer(input) => &mut input.metadata,
        UiInputEvent::TypeaheadTimer(input) => &mut input.metadata,
        UiInputEvent::SubmenuHoverTimer(input) => &mut input.metadata,
        UiInputEvent::ToastTimer(input) => &mut input.metadata,
        UiInputEvent::Accessibility(input) => &mut input.metadata,
    };
    metadata.timestamp = timestamp;
}

fn input_event_for_surface(
    event: &mut Option<UiInputEvent>,
    last_surface: bool,
) -> Option<UiInputEvent> {
    if last_surface {
        event.take()
    } else {
        event.as_ref().cloned()
    }
}

fn input_requires_focus_owner(event: &UiInputEvent) -> bool {
    matches!(
        event,
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_)
    )
}

fn input_requires_navigation_owner(event: &UiInputEvent) -> bool {
    matches!(event, UiInputEvent::Navigation(_) | UiInputEvent::Analog(_))
}

pub(super) fn published_focused_surfaces(surfaces: &[RuntimeUiSurface]) -> BTreeSet<usize> {
    surfaces
        .iter()
        .enumerate()
        .filter_map(|(surface_index, surface)| {
            surface
                .surface
                .focus
                .focused
                .is_some()
                .then_some(surface_index)
        })
        .collect()
}

pub(super) fn published_navigation_surface(
    surfaces: &[RuntimeUiSurface],
    focused_surface: Option<usize>,
) -> Option<usize> {
    focused_surface.or_else(|| {
        surfaces
            .iter()
            .enumerate()
            .rev()
            .find_map(|(surface_index, surface)| {
                surface
                    .surface
                    .has_navigation_candidate()
                    .then_some(surface_index)
            })
    })
}

pub(super) fn split_global_node_id(node_id: UiNodeId) -> Option<(usize, UiNodeId)> {
    let surface = node_id.0 >> NODE_ID_SURFACE_SHIFT;
    let surface_index = usize::try_from(surface.checked_sub(1)?).ok()?;
    Some((surface_index, UiNodeId::new(node_id.0 & NODE_ID_LOCAL_MASK)))
}

pub(super) fn capture_surface_for_event(
    capture_surfaces: &HashMap<Option<u64>, usize>,
    pointer_id: Option<u64>,
    kind: UiPointerEventKind,
) -> Option<usize> {
    matches!(
        kind,
        UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
    )
    .then(|| capture_surfaces.get(&pointer_id).copied())
    .flatten()
}

pub(super) fn update_capture_surface(
    capture_surfaces: &mut HashMap<Option<u64>, usize>,
    pointer_id: Option<u64>,
    surface_index: usize,
    kind: UiPointerEventKind,
    captures_pointer: bool,
) {
    if matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel) {
        capture_surfaces.remove(&pointer_id);
    } else if captures_pointer {
        capture_surfaces.insert(pointer_id, surface_index);
    } else if capture_surfaces.get(&pointer_id) == Some(&surface_index) {
        capture_surfaces.remove(&pointer_id);
    }
}
