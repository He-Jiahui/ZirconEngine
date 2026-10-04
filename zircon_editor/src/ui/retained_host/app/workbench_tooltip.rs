use std::time::{Instant, SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::dispatch::{UiInputTimestamp, UiPointerInputEvent};
use zircon_runtime_interface::ui::surface::UiPointerEventKind;

use crate::ui::retained_host::host_contract::HierarchyPointerSource;
use crate::ui::retained_host::host_contract::WorkbenchTooltipPointerTarget;
use crate::ui::retained_host::UiHostContext;

use super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn observe_workbench_pointer_input(
        &mut self,
        source: &HierarchyPointerSource,
        pointer: UiPointerInputEvent,
        tooltip_target: Option<WorkbenchTooltipPointerTarget>,
    ) {
        if pointer.event.kind == UiPointerEventKind::Scroll {
            let generation = self.ui.get_host_presentation_generation();
            let point = pointer.event.point;
            let structure = generation.structure();
            let in_pane = crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_panes(structure).iter().any(|pane| crate::ui::retained_host::host_contract::componentized_workbench_regions::point_in_frame(&pane.frame, point.x, point.y));
            let in_tree = crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(structure).is_some_and(|hierarchy| crate::ui::retained_host::host_contract::componentized_workbench_regions::point_in_frame(&hierarchy.viewport, point.x, point.y));
            drop(generation);
            if in_pane && !in_tree {
                match callback_dispatch::dispatch_componentized_workbench_pointer_event(
                    &self.runtime,
                    &mut self.workbench_window_bridge,
                    pointer.event.clone(),
                ) {
                    Some(Ok(effects)) => self.apply_dispatch_effects(effects),
                    Some(Err(error)) => self.set_status_line(error),
                    None => {}
                }
            }
        }
        if pointer.event.kind != UiPointerEventKind::Move {
            self.observe_hierarchy_native_input(source, &pointer);
        }
        if self
            .ui
            .global::<UiHostContext>()
            .host_popup_occludes_workbench_tooltip()
        {
            self.dismiss_workbench_tooltip();
            return;
        }
        let timestamp = pointer.metadata.timestamp;
        let observed_at = Instant::now();
        let changed = match self
            .workbench_window_bridge
            .update_workbench_icon_tooltip_candidate(pointer, tooltip_target)
        {
            Ok(changed) => changed,
            Err(error) => {
                self.set_status_line(error.to_string());
                self.ui.set_input_timer_frame_update(None);
                return;
            }
        };
        self.apply_workbench_tooltip_projection(changed);
        self.schedule_workbench_tooltip_tick(timestamp, observed_at);
    }

    pub(in crate::ui::retained_host::app) fn dismiss_workbench_tooltip(&mut self) {
        let changed = match self
            .workbench_window_bridge
            .dismiss_workbench_icon_tooltip()
        {
            Ok(changed) => changed,
            Err(error) => {
                self.set_status_line(error.to_string());
                false
            }
        };
        self.ui.set_input_timer_frame_update(None);
        self.apply_workbench_tooltip_projection(changed);
    }

    pub(super) fn tick_workbench_tooltip(&mut self) {
        if self
            .ui
            .global::<UiHostContext>()
            .host_popup_occludes_workbench_tooltip()
        {
            self.dismiss_workbench_tooltip();
            return;
        }
        let timestamp = current_input_timestamp();
        let now = Instant::now();
        let changed = match self
            .workbench_window_bridge
            .tick_workbench_icon_tooltip(timestamp)
        {
            Ok(changed) => changed,
            Err(error) => {
                self.set_status_line(error.to_string());
                self.ui.set_input_timer_frame_update(None);
                return;
            }
        };
        self.apply_workbench_tooltip_projection(changed);
        self.schedule_workbench_tooltip_tick(timestamp, now);
    }

    fn schedule_workbench_tooltip_tick(&self, timestamp: UiInputTimestamp, now: Instant) {
        let deadline = self
            .workbench_window_bridge
            .next_workbench_icon_tooltip_delay(timestamp)
            .map(|delay| now.checked_add(delay).unwrap_or(now));
        self.ui.set_input_timer_frame_update(deadline);
    }

    fn apply_workbench_tooltip_projection(&mut self, changed: bool) {
        if !changed {
            return;
        }
        let mut effects = UiHostEventEffects::default();
        effects.request_paint_only();
        effects.request_workbench_projection();
        self.apply_dispatch_effects(effects);
    }
}

fn current_input_timestamp() -> UiInputTimestamp {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default();
    UiInputTimestamp::from_micros(micros)
}
