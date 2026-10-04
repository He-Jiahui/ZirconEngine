use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::{UiDirtyFlags, UiTreeError},
};

use super::UiSurface;

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_pointer_component_state(
        &mut self,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        let mut changed_nodes = ChangedNodeAccumulator::default();
        for node_id in &route.entered {
            if self.component_states.set_hovered(*node_id, true) {
                changed_nodes.insert(*node_id);
            }
        }
        for node_id in &route.left {
            if self.component_states.set_hovered(*node_id, false) {
                changed_nodes.insert(*node_id);
            }
        }
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)?
                        && self.component_states.set_pressed(target, true)
                    {
                        changed_nodes.insert(target);
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    if self
                        .component_states
                        .set_pressed(pressed, self.input.has_primary_pointer_press(pressed))
                    {
                        changed_nodes.insert(pressed);
                    }
                }
            }
            _ => {}
        }
        if matches!(route.kind, UiPointerEventKind::Cancel) {
            if let Some(pressed) = route.pressed {
                if self
                    .component_states
                    .set_pressed(pressed, self.input.has_primary_pointer_press(pressed))
                {
                    changed_nodes.insert(pressed);
                }
            }
        }
        if focus_before_dispatch != self.focus.focused {
            if let Some(previous) = focus_before_dispatch {
                if self.component_states.set_focused(previous, false) {
                    changed_nodes.insert(previous);
                }
            }
            if let Some(current) = self.focus.focused {
                if self.component_states.set_focused(current, true) {
                    changed_nodes.insert(current);
                }
            }
        }
        changed_nodes.finish(self)
    }

    pub(in crate::ui::surface::surface) fn apply_pointer_transient_state_dirty(
        &mut self,
        route: &UiPointerRoute,
        pressed_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(previous) =
                    pressed_before_dispatch.filter(|previous| Some(*previous) != route.target)
                {
                    self.set_node_pressed_dirty(
                        previous,
                        self.input.has_primary_pointer_press(previous),
                    )?;
                }
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)? {
                        self.set_node_pressed_dirty(target, true)?;
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    self.set_node_pressed_dirty(
                        pressed,
                        self.input.has_primary_pointer_press(pressed),
                    )?;
                }
            }
            _ => {}
        }
        if matches!(route.kind, UiPointerEventKind::Cancel) {
            if let Some(pressed) = route.pressed {
                self.set_node_pressed_dirty(
                    pressed,
                    self.input.has_primary_pointer_press(pressed),
                )?;
            }
        }
        Ok(())
    }

    pub(in crate::ui::surface::surface) fn set_node_pressed_dirty(
        &mut self,
        node_id: UiNodeId,
        pressed: bool,
    ) -> Result<(), UiTreeError> {
        let component_state_changed = self.component_states.set_pressed(node_id, pressed);
        let state_flags_changed = {
            let node = self
                .tree
                .nodes
                .get_mut(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node.state_flags.pressed == pressed {
                false
            } else {
                node.state_flags.pressed = pressed;
                true
            }
        };
        if component_state_changed {
            self.mark_component_state_render_dirty(node_id)?;
        } else if state_flags_changed {
            self.mark_node_dirty(
                node_id,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )?;
        }
        Ok(())
    }

    pub(crate) fn mark_component_state_render_dirty(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<(), UiTreeError> {
        if self
            .runtime_style
            .node_state_can_affect_descendants(&self.tree, node_id)?
        {
            let _ = self.apply_runtime_state_style_subtree(node_id, true)?;
        } else {
            let _ = self.apply_runtime_state_style_node(node_id, true)?;
        }
        self.mark_node_dirty(
            node_id,
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )?;
        Ok(())
    }

    pub(crate) fn mark_component_states_render_dirty(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        if changed_node_ids.len() == 1 {
            return self.mark_component_state_render_dirty(
                *changed_node_ids
                    .first()
                    .expect("a non-empty single-node set has one first element"),
            );
        }
        let mut descendant_affecting_node_ids = BTreeSet::new();
        for node_id in changed_node_ids {
            if self
                .runtime_style
                .node_state_can_affect_descendants(&self.tree, *node_id)?
            {
                descendant_affecting_node_ids.insert(*node_id);
            }
        }
        let style_roots =
            minimal_changed_subtree_roots(&self.tree, &descendant_affecting_node_ids)?;
        let style_root_ids = style_roots.iter().copied().collect::<BTreeSet<_>>();
        for root_id in &style_roots {
            let _ = self.apply_runtime_state_style_subtree(*root_id, true)?;
        }
        for node_id in changed_node_ids {
            if !node_is_covered_by_roots(&self.tree, *node_id, &style_root_ids)? {
                let _ = self.apply_runtime_state_style_node(*node_id, true)?;
            }
        }
        for node_id in changed_node_ids {
            self.mark_node_dirty(
                *node_id,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )?;
        }
        Ok(())
    }
}

// Pointer transitions usually touch one node; only a genuinely multi-node transition needs
// the deterministic ordered set used by the subtree batching path.
#[derive(Default)]
struct ChangedNodeAccumulator {
    single: Option<UiNodeId>,
    many: Option<BTreeSet<UiNodeId>>,
}

impl ChangedNodeAccumulator {
    fn insert(&mut self, node_id: UiNodeId) {
        if let Some(many) = self.many.as_mut() {
            many.insert(node_id);
            return;
        }

        match self.single {
            None => self.single = Some(node_id),
            Some(existing) if existing == node_id => {}
            Some(existing) => {
                let mut many = BTreeSet::new();
                many.insert(existing);
                many.insert(node_id);
                self.many = Some(many);
                self.single = None;
            }
        }
    }

    fn finish(self, surface: &mut UiSurface) -> Result<(), UiTreeError> {
        match self.many {
            Some(many) => surface.mark_component_states_render_dirty(&many),
            None => match self.single {
                Some(node_id) => surface.mark_component_state_render_dirty(node_id),
                None => Ok(()),
            },
        }
    }
}

fn minimal_changed_subtree_roots(
    tree: &zircon_runtime_interface::ui::tree::UiTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut roots = Vec::with_capacity(changed_node_ids.len());
    for node_id in changed_node_ids {
        let node = tree
            .nodes
            .get(node_id)
            .ok_or(UiTreeError::MissingNode(*node_id))?;
        let mut ancestor = node.parent;
        let mut covered = false;
        while let Some(ancestor_id) = ancestor {
            if changed_node_ids.contains(&ancestor_id) {
                covered = true;
                break;
            }
            ancestor = tree
                .nodes
                .get(&ancestor_id)
                .ok_or(UiTreeError::MissingNode(ancestor_id))?
                .parent;
        }
        if !covered {
            roots.push(*node_id);
        }
    }
    Ok(roots)
}

fn node_is_covered_by_roots(
    tree: &zircon_runtime_interface::ui::tree::UiTree,
    node_id: UiNodeId,
    root_ids: &BTreeSet<UiNodeId>,
) -> Result<bool, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        if root_ids.contains(&current_id) {
            return Ok(true);
        }
        current = tree
            .nodes
            .get(&current_id)
            .ok_or(UiTreeError::MissingNode(current_id))?
            .parent;
    }
    Ok(false)
}

#[cfg(test)]
#[path = "tests/state_invalidation_optimization_tests.rs"]
mod optimization_tests;
