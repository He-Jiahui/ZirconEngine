use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    component::UiComponentState,
    ecs::{
        UiEcsDirtyDomainImpact, UiEcsDirtyDomains, UiEcsInteractionState, UiEcsNodeProjection,
        UiEcsProjectionDelta, UiEcsProjectionScheduleImpact, UiEcsProjectionScheduleMask,
        UiEcsProjectionSnapshot,
    },
    event_ui::UiNodeId,
};

use super::{ui_surface_effective_disabled, UiSurface};

impl UiSurface {
    pub fn ui_ecs_projection(&self) -> UiEcsProjectionSnapshot {
        let render_counts = render_command_counts(self);
        let hit_counts = hit_entry_counts(self);
        let nodes = self
            .tree
            .nodes
            .iter()
            .map(|(node_id, node)| {
                let metadata = node.template_metadata.as_ref();
                let component_state = self.component_states.get(*node_id);
                let disabled = ui_surface_effective_disabled(self, *node_id, node, metadata);
                UiEcsNodeProjection {
                    node_id: *node_id,
                    node_path: node.node_path.clone(),
                    parent: node.parent,
                    children: node.children.clone(),
                    component: metadata
                        .map(|metadata| metadata.component.clone())
                        .unwrap_or_default(),
                    control_id: metadata.and_then(|metadata| metadata.control_id.clone()),
                    frame: node.layout_cache.frame,
                    dirty: UiEcsDirtyDomains::from_dirty_flags(projection_dirty_flags(node)),
                    interaction: ecs_interaction_state(self, *node_id, component_state, disabled),
                    render_command_count: render_counts.get(node_id).copied().unwrap_or(0),
                    hit_entry_count: hit_counts.get(node_id).copied().unwrap_or(0),
                }
            })
            .collect();

        UiEcsProjectionSnapshot::from_nodes(
            self.tree.tree_id.clone(),
            self.tree.roots.clone(),
            nodes,
        )
    }

    pub fn ui_ecs_projection_delta_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> UiEcsProjectionDelta {
        self.ui_ecs_projection().diff_from(previous)
    }

    pub fn ui_ecs_schedule_mask_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> UiEcsProjectionScheduleMask {
        self.ui_ecs_projection_delta_from(previous).schedule_mask()
    }

    pub fn ui_ecs_schedule_impacts_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiEcsProjectionScheduleImpact> {
        self.ui_ecs_projection_delta_from(previous)
            .schedule_impacts()
    }

    pub fn ui_ecs_dirty_domain_impacts_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiEcsDirtyDomainImpact> {
        self.ui_ecs_projection_delta_from(previous)
            .dirty_domain_impacts()
    }

    pub fn ui_ecs_component_structure_change_node_ids_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiNodeId> {
        self.ui_ecs_projection_delta_from(previous)
            .component_structure_change_node_ids()
    }

    pub fn ui_ecs_interaction_change_node_ids_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiNodeId> {
        self.ui_ecs_projection_delta_from(previous)
            .interaction_change_node_ids()
    }

    pub fn ui_ecs_interaction_only_change_node_ids_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiNodeId> {
        self.ui_ecs_projection_delta_from(previous)
            .interaction_only_change_node_ids()
    }

    pub fn ui_ecs_render_only_change_node_ids_from(
        &self,
        previous: &UiEcsProjectionSnapshot,
    ) -> Vec<UiNodeId> {
        self.ui_ecs_projection_delta_from(previous)
            .render_only_change_node_ids()
    }
}

fn ecs_interaction_state(
    surface: &UiSurface,
    node_id: UiNodeId,
    component_state: Option<&UiComponentState>,
    disabled: bool,
) -> UiEcsInteractionState {
    let Some(node) = surface.tree.nodes.get(&node_id) else {
        return UiEcsInteractionState::default();
    };
    let flags = component_state
        .map(|state| state.flags.clone())
        .unwrap_or_default();
    UiEcsInteractionState {
        visible: node.is_render_visible(),
        enabled: !disabled,
        disabled,
        focused: surface.focus.focused == Some(node_id) || flags.focused,
        hovered: surface.focus.hovered.contains(&node_id) || flags.hovered,
        pressed: surface.focus.pressed == Some(node_id) || flags.pressed,
        captured: surface.focus.captured == Some(node_id),
        focusable: node.state_flags.focusable || node.focus.focusable,
        clickable: node.state_flags.clickable,
        hoverable: node.state_flags.hoverable,
        checked: node.state_flags.checked || flags.checked,
        selected: flags.selected,
        expanded: flags.expanded,
        popup_open: flags.popup_open,
        dragging: flags.dragging,
    }
}

fn render_command_counts(surface: &UiSurface) -> BTreeMap<UiNodeId, u64> {
    let mut counts = BTreeMap::new();
    for command in &surface.render_extract.list.commands {
        *counts.entry(command.node_id).or_insert(0) += 1;
    }
    counts
}

fn hit_entry_counts(surface: &UiSurface) -> BTreeMap<UiNodeId, u64> {
    let mut counts = BTreeMap::new();
    for entry in &surface.hit_test.grid.entries {
        *counts.entry(entry.node_id).or_insert(0) += 1;
    }
    counts
}

fn projection_dirty_flags(
    node: &zircon_runtime_interface::ui::tree::UiTreeNode,
) -> zircon_runtime_interface::ui::tree::UiDirtyFlags {
    let mut dirty = node.dirty;
    if node.state_flags.dirty {
        dirty.input = true;
        dirty.hit_test = true;
        dirty.render = true;
    }
    dirty
}
