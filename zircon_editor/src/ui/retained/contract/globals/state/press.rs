use super::HostContractState;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::native_pointer::routing::{
    route_pointer_to_pane, route_pointer_to_workbench_generation, PanePointerTarget,
};
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;

impl HostContractState {
    pub(crate) fn begin_template_button_press(
        &mut self,
        hit: &TemplateNodePointerHit,
        x: f32,
        y: f32,
    ) {
        if self.exit_requested
            || hit.disabled
            || !matches!(
                hit.component_family,
                Some(TemplateComponentFamily::Button | TemplateComponentFamily::IconButton)
            )
        {
            return;
        }
        let previous_frame = (!self
            .pane_interaction_state
            .pressed_template_control_id
            .is_empty())
        .then(|| self.pane_interaction_state.pressed_template_frame.clone());
        let scale_factor = self.window_scale_factor;
        let project = self.host_presentation.host_shell.project_path.clone();
        let changed = self.update_pane_interaction(|interaction| {
            interaction.pressed_template_control_id = hit.control_id.clone();
            interaction.pressed_template_pane_id = hit.pane_id.clone();
            interaction.pressed_template_action_id = hit.action_id.clone();
            interaction.pressed_template_dispatch_kind = hit.dispatch_kind.clone();
            interaction.pressed_template_project_path = project;
            interaction.pressed_template_frame = hit.frame.clone();
            interaction.pressed_template_down_x = x;
            interaction.pressed_template_down_y = y;
            interaction.pressed_template_scale_factor = scale_factor;
        });
        if changed {
            let damage = previous_frame.map_or_else(
                || hit.frame.clone(),
                |previous| union_frame(&previous, &hit.frame),
            );
            self.queue_external_redraw(
                HostRedrawRequest::region_with_frame_update(damage).into_interactive_frame_update(),
            );
        }
    }

    pub(crate) fn clear_template_button_press(&mut self) -> Option<FrameRect> {
        if self
            .pane_interaction_state
            .pressed_template_control_id
            .is_empty()
        {
            return None;
        }
        let frame = self.pane_interaction_state.pressed_template_frame.clone();
        self.update_pane_interaction(|interaction| {
            interaction.pressed_template_control_id.clear();
            interaction.pressed_template_pane_id.clear();
            interaction.pressed_template_action_id.clear();
            interaction.pressed_template_dispatch_kind.clear();
            interaction.pressed_template_project_path.clear();
            interaction.pressed_template_frame = FrameRect::default();
            interaction.pressed_template_down_x = 0.0;
            interaction.pressed_template_down_y = 0.0;
            interaction.pressed_template_scale_factor = 0.0;
        });
        Some(frame)
    }

    // Reuse the published hit routes only when an active press survives a publish.
    // A replaced, moved, disabled, or differently bound button cannot inherit it.
    pub(super) fn reconcile_template_button_press(&mut self) {
        let interaction = self.pane_interaction_state.as_ref();
        if interaction.pressed_template_control_id.is_empty() {
            return;
        }
        let frame = &interaction.pressed_template_frame;
        // The original validated native point remains valid for a partially clipped
        // button; its full-frame center need not lie inside the visible clip.
        let x = interaction.pressed_template_down_x;
        let y = interaction.pressed_template_down_y;
        let generation = self.presentation_generation();
        let hit = route_pointer_to_workbench_generation(&generation, x, y).or_else(|| {
            let route = route_pointer_to_pane(self.host_presentation.as_ref(), interaction, x, y)?;
            match route.target {
                PanePointerTarget::TemplateNode(hit) => Some(hit.to_owned_hit()),
                _ => None,
            }
        });
        let retained = interaction.pressed_template_scale_factor == self.window_scale_factor
            && interaction.pressed_template_project_path
                == self.host_presentation.host_shell.project_path
            && hit.is_some_and(|hit| {
                !hit.disabled
                    && hit.control_id == interaction.pressed_template_control_id
                    && hit.pane_id == interaction.pressed_template_pane_id
                    && hit.action_id == interaction.pressed_template_action_id
                    && hit.dispatch_kind == interaction.pressed_template_dispatch_kind
                    && hit.frame == *frame
                    && matches!(
                        hit.component_family,
                        Some(TemplateComponentFamily::Button | TemplateComponentFamily::IconButton)
                    )
            });
        if !retained {
            if let Some(frame) = self.clear_template_button_press() {
                if !self.exit_requested {
                    self.queue_external_redraw(
                        HostRedrawRequest::region_with_frame_update(frame)
                            .into_interactive_frame_update(),
                    );
                }
            }
        }
    }

    pub(crate) fn queue_external_redraw(&mut self, redraw: HostRedrawRequest) {
        if !redraw.request_redraw() {
            return;
        }
        let existing =
            std::mem::replace(&mut self.external_redraw_request, HostRedrawRequest::none());
        if existing.request_redraw() {
            self.external_redraw_coalesced_count =
                self.external_redraw_coalesced_count.saturating_add(1);
        }
        self.external_redraw_request = existing.merge(redraw);
        self.external_redraw_queued_count = self.external_redraw_queued_count.saturating_add(1);
    }
}
