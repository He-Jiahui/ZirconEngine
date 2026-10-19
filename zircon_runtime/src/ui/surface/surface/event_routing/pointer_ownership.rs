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

        let target_is_pass = target.is_some_and(|id| {
            self.tree
                .nodes
                .get(&id)
                .is_some_and(|node| node.is_pointer_passthrough())
        });
        let action_path = if captured.is_none() && target_is_pass {
            Some(self.pointer_action_ancestors(target)?)
        } else {
            None
        };
        let action_target = match &action_path {
            Some(path) => path.first().copied(),
            None => target,
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
                self.focus.pressed = action_target;
                self.input.record_pointer_press(action_target, button);
            }
            let focus_target = if let Some(path) = &action_path {
                path.first().copied().filter(|id| {
                    self.tree
                        .nodes
                        .get(id)
                        .is_some_and(|node| node.is_focus_candidate())
                })
            } else {
                self.tree.first_focusable_in_route_iter(
                    routing_path.root_to_leaf(&hit.path).iter().rev().copied(),
                )?
            };
            if let Some(focus_target) = focus_target.filter(|id| is_valid_input_owner(self, *id)) {
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
            && previous_pressed.is_some_and(|node_id| {
                self.tree
                    .nodes
                    .get(&node_id)
                    .is_some_and(|node| node.state_flags.clickable)
            })
            && previous_pressed.is_some_and(|node_id| {
                hit.stacked.contains(&node_id)
                    || (captured.is_none()
                        && hit.top_hit.is_some_and(|hit| {
                            self.tree
                                .nodes
                                .get(&hit)
                                .is_some_and(|node| node.is_pointer_passthrough())
                        })
                        && action_path
                            .as_ref()
                            .is_some_and(|owners| owners.contains(&node_id)))
            }) {
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

impl UiSurface {
    // Real ancestors only; unrelated stacked hits are never default-action substitutes.
    fn pointer_action_ancestors(
        &self,
        target: Option<UiNodeId>,
    ) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut owners = Vec::new();
        if let Some(target) = target {
            for owner in self.tree.bubble_route(target)? {
                let node = self
                    .tree
                    .nodes
                    .get(&owner)
                    .ok_or(UiTreeError::MissingNode(owner))?;
                if !node.is_pointer_passthrough() {
                    if is_valid_input_owner(self, owner) {
                        owners.push(owner);
                    }
                    break;
                }
            }
        }
        Ok(owners)
    }

    pub(crate) fn pointer_default_action_route<'a>(
        &self,
        route: &'a UiPointerRoute,
        explicit_stop: bool,
    ) -> Result<std::borrow::Cow<'a, UiPointerRoute>, UiTreeError> {
        if route.captured.is_some() {
            return Ok(std::borrow::Cow::Borrowed(route));
        }
        let target_is_pass = route.target.is_some_and(|id| {
            self.tree
                .nodes
                .get(&id)
                .is_some_and(|node| node.is_pointer_passthrough())
        });
        if !target_is_pass {
            return Ok(std::borrow::Cow::Borrowed(route));
        }
        let mut action = route.clone();
        let owners = if explicit_stop {
            Vec::new()
        } else {
            self.pointer_action_ancestors(route.target)?
        };
        action.target = owners.first().copied();
        action.routing_path = UiPointerRoutingPath::from_bubble_route(owners.clone());
        action.stacked = owners;
        action.root_targets.clear();
        action.fallback_to_root = false;
        if explicit_stop {
            action.activation_phase = UiPointerActivationPhase::Hover;
            action.pressed = None;
            action.click_target = None;
            action.release_inside_pressed = false;
        }
        Ok(std::borrow::Cow::Owned(action))
    }

    pub(crate) fn pointer_explicitly_stopped(result: &UiPointerDispatchResult) -> bool {
        use zircon_runtime_interface::ui::dispatch::{UiDispatchPhase, UiPointerDispatchEffect};
        result.invocations.iter().any(|invocation| {
            matches!(
                invocation.effect,
                UiPointerDispatchEffect::CapturePointer
                    | UiPointerDispatchEffect::ReleasePointerCapture
                    | UiPointerDispatchEffect::SetFocus
                    | UiPointerDispatchEffect::ClearFocus
            ) || (invocation.phase == UiDispatchPhase::PreviewTunnel
                && matches!(
                    invocation.effect,
                    UiPointerDispatchEffect::Handled | UiPointerDispatchEffect::Blocked
                ))
        })
    }
}
