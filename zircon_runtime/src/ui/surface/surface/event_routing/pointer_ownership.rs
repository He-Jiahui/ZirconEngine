use super::*;

impl UiSurface {
    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        if !is_valid_input_owner(self, node_id) {
            return Err(UiTreeError::MissingNode(node_id));
        }
        if let Some(previous) = self.focus.captured.filter(|owner| owner != &node_id) {
            self.input.clear_high_precision_for(previous);
        }
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        let released = self.focus.captured.take();
        if let Some(owner) = released {
            self.input.clear_pointer_capture_for(owner);
            self.input.clear_pointer_drag_for(owner);
        } else {
            self.input.clear_pointer_capture();
        }
        released
    }

    pub(super) fn clear_routed_pointer_capture(&mut self, owner: UiNodeId) {
        self.input
            .clear_pointer_capture_id_for_owner(self.input.routed_pointer_id(), owner);
        if !self.input.has_pointer_capture_for_owner(owner) {
            self.input.clear_high_precision_for(owner);
        }
    }

    pub(super) fn route_pointer_event_with_details(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
        button: Option<UiPointerButton>,
        modifiers: UiInputModifiers,
        scroll_delta: f32,
    ) -> Result<UiPointerRoute, UiTreeError> {
        let point = query.hit_point();
        let hit = self.hit_test_with_query(query);
        let captured = self.focus.captured;
        let previous_pressed = self.focus.pressed;
        let pointer_id = self.input.routed_pointer_id();
        let press_matches = previous_pressed
            .is_none_or(|owner| self.input.pointer_press_matches(pointer_id, owner, button));
        let capture_matches = captured.is_none_or(|owner| {
            self.input
                .pointer_capture_matches(pointer_id, owner, button)
        });
        let activation_matches = press_matches && capture_matches;
        let target = captured.or(hit.top_hit);
        let routing_path = match (captured, target) {
            (None, Some(node_id)) if hit.path.target == Some(node_id) => {
                UiPointerRoutingPath::HitPath
            }
            (_, Some(node_id)) => {
                UiPointerRoutingPath::from_bubble_route(self.tree.bubble_route(node_id)?)
            }
            (_, None) => UiPointerRoutingPath::ExplicitRootToLeaf(Vec::new()),
        };

        let (entered, left) = if hit.stacked == self.focus.hovered {
            (Vec::new(), Vec::new())
        } else {
            let previous_hovered = std::mem::replace(&mut self.focus.hovered, hit.stacked.clone());
            let mut hover_diff_scratch = std::mem::take(&mut self.hover_diff_scratch);
            let result = hover_diff_with_scratch(
                &hit.stacked,
                &previous_hovered,
                &mut hover_diff_scratch.membership,
            );
            if hover_diff_scratch.membership.capacity() > MAX_UI_LAYOUT_DISCRETE_VALUE {
                hover_diff_scratch.membership = HashSet::new();
            }
            self.hover_diff_scratch = hover_diff_scratch;
            result
        };
        if matches!(kind, UiPointerEventKind::Down) {
            if activation_matches {
                self.focus.pressed = target;
                self.input.record_pointer_press(target, button);
            }
            if let Some(focus_target) = self
                .tree
                .first_focusable_in_route_iter(
                    routing_path.root_to_leaf(&hit.path).iter().rev().copied(),
                )?
                .filter(|focus_target| is_valid_input_owner(self, *focus_target))
            {
                self.focus_node_with_reason(
                    focus_target,
                    UiFocusChangeReason::Input,
                    UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
                )?;
            }
        }
        let click_target = if matches!(kind, UiPointerEventKind::Up)
            && button == Some(UiPointerButton::Primary)
            && activation_matches
            && previous_pressed.is_some_and(|node_id| hit.stacked.contains(&node_id))
        {
            previous_pressed
        } else {
            None
        };
        if matches!(kind, UiPointerEventKind::Up) {
            if press_matches {
                self.focus.pressed = None;
                self.input.clear_pointer_press(pointer_id);
            }
            if capture_matches {
                self.focus.captured = None;
                if let Some(owner) = captured {
                    self.clear_routed_pointer_capture(owner);
                }
            }
        } else if matches!(kind, UiPointerEventKind::Cancel) {
            self.focus.pressed = None;
            self.input.clear_pointer_press(pointer_id);
            self.focus.captured = None;
            if let Some(owner) = captured {
                self.clear_routed_pointer_capture(owner);
                if !self.input.has_pointer_capture_for_owner(owner) {
                    self.input.clear_pointer_drag_for(owner);
                }
            }
        }
        let pressed = if matches!(kind, UiPointerEventKind::Down) {
            self.focus.pressed
        } else {
            previous_pressed
        };

        Ok(UiPointerRoute {
            kind,
            button,
            modifiers,
            activation_phase: if !activation_matches
                && matches!(kind, UiPointerEventKind::Down | UiPointerEventKind::Up)
            {
                UiPointerActivationPhase::Hover
            } else {
                activation_phase(kind, button)
            },
            point,
            scroll_delta,
            target,
            hit_path: hit.path,
            routing_path,
            stacked: hit.stacked,
            entered,
            left,
            captured,
            pressed,
            click_target,
            release_inside_pressed: click_target.is_some(),
            focused: self.focus.focused,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }
}
