use serde::{Deserialize, Serialize};

use crate::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    pipeline::{UiPipelineDirtyReason, UiPipelineStage},
    tree::UiDirtyFlags,
};

mod compute;

use compute::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: Vec<UiEcsNodeProjection>,
    pub totals: UiEcsProjectionTotals,
    pub schedule_mask: UiEcsProjectionScheduleMask,
    pub schedule_impacts: Vec<UiEcsProjectionScheduleImpact>,
    pub dirty_domain_impacts: Vec<UiEcsDirtyDomainImpact>,
}

impl UiEcsProjectionSnapshot {
    pub fn from_nodes(
        tree_id: UiTreeId,
        roots: Vec<UiNodeId>,
        nodes: Vec<UiEcsNodeProjection>,
    ) -> Self {
        let totals = UiEcsProjectionTotals::from_nodes(&nodes);
        let schedule_mask = projection_schedule_mask_from_nodes(&nodes);
        let schedule_impacts = projection_schedule_impacts_from_nodes(&nodes);
        let dirty_domain_impacts = projection_dirty_domain_impacts_from_nodes(&nodes);
        Self {
            tree_id,
            roots,
            nodes,
            totals,
            schedule_mask,
            schedule_impacts,
            dirty_domain_impacts,
        }
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiEcsNodeProjection> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn diff_from(&self, previous: &Self) -> UiEcsProjectionDelta {
        let previous_nodes = projection_node_map(&previous.nodes);
        let current_nodes = projection_node_map(&self.nodes);
        let mut changes = Vec::new();

        for (node_id, previous_node) in &previous_nodes {
            if let Some(current_node) = current_nodes.get(node_id) {
                let reasons = projection_node_change_reasons(previous_node, current_node);
                if !reasons.is_empty() {
                    changes.push(UiEcsProjectionNodeChange {
                        node_id: *node_id,
                        node_path: current_node.node_path.clone(),
                        kind: UiEcsProjectionChangeKind::Updated,
                        domains: projection_update_domains(previous_node, current_node, &reasons),
                        reasons,
                    });
                }
            } else {
                changes.push(UiEcsProjectionNodeChange {
                    node_id: *node_id,
                    node_path: previous_node.node_path.clone(),
                    kind: UiEcsProjectionChangeKind::Removed,
                    domains: UiEcsDirtyDomains::structural_change().union(previous_node.dirty),
                    reasons: vec![UiEcsProjectionChangeReason::Removed],
                });
            }
        }

        for (node_id, current_node) in &current_nodes {
            if !previous_nodes.contains_key(node_id) {
                changes.push(UiEcsProjectionNodeChange {
                    node_id: *node_id,
                    node_path: current_node.node_path.clone(),
                    kind: UiEcsProjectionChangeKind::Added,
                    domains: UiEcsDirtyDomains::structural_change().union(current_node.dirty),
                    reasons: vec![UiEcsProjectionChangeReason::Added],
                });
            }
        }

        let totals = UiEcsProjectionDeltaTotals::from_changes(&changes);
        let schedule_mask = projection_schedule_mask_from_changes(&changes);
        let schedule_impacts = projection_schedule_impacts_from_changes(&changes);
        let dirty_domain_impacts = projection_dirty_domain_impacts_from_changes(&changes);
        UiEcsProjectionDelta {
            previous_tree_id: previous.tree_id.clone(),
            current_tree_id: self.tree_id.clone(),
            changes,
            totals,
            schedule_mask,
            schedule_impacts,
            dirty_domain_impacts,
        }
    }

    pub fn schedule_mask(&self) -> UiEcsProjectionScheduleMask {
        projection_schedule_mask_from_nodes(&self.nodes)
    }

    pub fn schedule_impacts(&self) -> Vec<UiEcsProjectionScheduleImpact> {
        projection_schedule_impacts_from_nodes(&self.nodes)
    }

    pub fn schedule_impact(&self, stage: UiPipelineStage) -> Option<UiEcsProjectionScheduleImpact> {
        self.schedule_impacts()
            .into_iter()
            .find(|impact| impact.stage == stage)
    }

    pub fn node_ids_requiring_stage(&self, stage: UiPipelineStage) -> Vec<UiNodeId> {
        self.schedule_impact(stage)
            .map(|impact| impact.node_ids)
            .unwrap_or_default()
    }

    pub fn dirty_domain_impacts(&self) -> Vec<UiEcsDirtyDomainImpact> {
        projection_dirty_domain_impacts_from_nodes(&self.nodes)
    }

    pub fn dirty_domain_impact(
        &self,
        domain: UiEcsDirtyDomainKind,
    ) -> Option<UiEcsDirtyDomainImpact> {
        self.dirty_domain_impacts()
            .into_iter()
            .find(|impact| impact.domain == domain)
    }

    pub fn node_ids_in_dirty_domain(&self, domain: UiEcsDirtyDomainKind) -> Vec<UiNodeId> {
        self.dirty_domain_impact(domain)
            .map(|impact| impact.node_ids)
            .unwrap_or_default()
    }

    pub fn derived_fields_are_fresh(&self) -> bool {
        self.totals == UiEcsProjectionTotals::from_nodes(&self.nodes)
            && self.schedule_mask == self.schedule_mask()
            && self.schedule_impacts == self.schedule_impacts()
            && self.dirty_domain_impacts == self.dirty_domain_impacts()
    }

    pub fn recompute_totals(&mut self) {
        self.totals = UiEcsProjectionTotals::from_nodes(&self.nodes);
    }

    pub fn recompute_schedule_mask(&mut self) {
        self.schedule_mask = self.schedule_mask();
    }

    pub fn recompute_schedule_impacts(&mut self) {
        self.schedule_impacts = self.schedule_impacts();
    }

    pub fn recompute_dirty_domain_impacts(&mut self) {
        self.dirty_domain_impacts = self.dirty_domain_impacts();
    }

    pub fn recompute_derived_fields(&mut self) {
        self.recompute_totals();
        self.recompute_schedule_mask();
        self.recompute_schedule_impacts();
        self.recompute_dirty_domain_impacts();
    }

    pub fn with_recomputed_totals(mut self) -> Self {
        self.recompute_totals();
        self
    }

    pub fn with_recomputed_schedule_mask(mut self) -> Self {
        self.recompute_schedule_mask();
        self
    }

    pub fn with_recomputed_schedule_impacts(mut self) -> Self {
        self.recompute_schedule_impacts();
        self
    }

    pub fn with_recomputed_dirty_domain_impacts(mut self) -> Self {
        self.recompute_dirty_domain_impacts();
        self
    }

    pub fn with_recomputed_derived_fields(mut self) -> Self {
        self.recompute_derived_fields();
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsNodeProjection {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub component: String,
    pub control_id: Option<String>,
    pub frame: UiFrame,
    pub dirty: UiEcsDirtyDomains,
    pub interaction: UiEcsInteractionState,
    pub render_command_count: u64,
    pub hit_entry_count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsDirtyDomains {
    pub layout: bool,
    pub text: bool,
    pub input: bool,
    pub picking: bool,
    pub accessibility: bool,
    pub render: bool,
    pub style: bool,
    pub visible_range: bool,
}

impl UiEcsDirtyDomains {
    pub const fn from_dirty_flags(dirty: UiDirtyFlags) -> Self {
        Self {
            layout: dirty.layout,
            text: dirty.text,
            input: dirty.input,
            picking: dirty.hit_test || dirty.input || dirty.layout || dirty.visible_range,
            accessibility: dirty.input
                || dirty.style
                || dirty.text
                || dirty.layout
                || dirty.visible_range,
            render: dirty.render
                || dirty.style
                || dirty.text
                || dirty.layout
                || dirty.visible_range,
            style: dirty.style,
            visible_range: dirty.visible_range,
        }
    }

    pub const fn any(self) -> bool {
        self.layout
            || self.text
            || self.input
            || self.picking
            || self.accessibility
            || self.render
            || self.style
            || self.visible_range
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            layout: self.layout || other.layout,
            text: self.text || other.text,
            input: self.input || other.input,
            picking: self.picking || other.picking,
            accessibility: self.accessibility || other.accessibility,
            render: self.render || other.render,
            style: self.style || other.style,
            visible_range: self.visible_range || other.visible_range,
        }
    }

    pub const fn contains(self, domain: UiEcsDirtyDomainKind) -> bool {
        match domain {
            UiEcsDirtyDomainKind::Layout => self.layout,
            UiEcsDirtyDomainKind::Text => self.text,
            UiEcsDirtyDomainKind::Input => self.input,
            UiEcsDirtyDomainKind::Picking => self.picking,
            UiEcsDirtyDomainKind::Accessibility => self.accessibility,
            UiEcsDirtyDomainKind::Render => self.render,
            UiEcsDirtyDomainKind::Style => self.style,
            UiEcsDirtyDomainKind::VisibleRange => self.visible_range,
        }
    }

    pub const fn structural_change() -> Self {
        Self {
            layout: true,
            picking: true,
            accessibility: true,
            render: true,
            ..Self::empty()
        }
    }

    pub const fn interaction_change() -> Self {
        Self {
            input: true,
            accessibility: true,
            render: true,
            ..Self::empty()
        }
    }

    pub const fn render_change() -> Self {
        Self {
            render: true,
            ..Self::empty()
        }
    }

    pub const fn picking_change() -> Self {
        Self {
            picking: true,
            ..Self::empty()
        }
    }

    const fn empty() -> Self {
        Self {
            layout: false,
            text: false,
            input: false,
            picking: false,
            accessibility: false,
            render: false,
            style: false,
            visible_range: false,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum UiEcsDirtyDomainKind {
    #[default]
    Layout,
    Text,
    Input,
    Picking,
    Accessibility,
    Render,
    Style,
    VisibleRange,
}

impl UiEcsDirtyDomainKind {
    pub const ORDER: [Self; 8] = [
        Self::Layout,
        Self::Text,
        Self::Input,
        Self::Picking,
        Self::Accessibility,
        Self::Render,
        Self::Style,
        Self::VisibleRange,
    ];

    pub const fn ordered() -> &'static [Self; 8] {
        &Self::ORDER
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsInteractionState {
    pub visible: bool,
    pub enabled: bool,
    pub disabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub captured: bool,
    pub focusable: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub checked: bool,
    pub selected: bool,
    pub expanded: bool,
    pub popup_open: bool,
    pub dragging: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionTotals {
    pub node_count: u64,
    pub dirty_node_count: u64,
    pub layout_dirty_count: u64,
    pub text_dirty_count: u64,
    pub input_dirty_count: u64,
    pub picking_dirty_count: u64,
    pub accessibility_dirty_count: u64,
    pub render_dirty_count: u64,
    pub focused_count: u64,
    pub hovered_count: u64,
    pub pressed_count: u64,
    pub disabled_count: u64,
    pub render_command_count: u64,
    pub hit_entry_count: u64,
}

impl UiEcsProjectionTotals {
    pub fn from_nodes(nodes: &[UiEcsNodeProjection]) -> Self {
        let mut totals = Self {
            node_count: nodes.len() as u64,
            ..Self::default()
        };
        for node in nodes {
            totals.dirty_node_count += u64::from(node.dirty.any());
            totals.layout_dirty_count += u64::from(node.dirty.layout);
            totals.text_dirty_count += u64::from(node.dirty.text);
            totals.input_dirty_count += u64::from(node.dirty.input);
            totals.picking_dirty_count += u64::from(node.dirty.picking);
            totals.accessibility_dirty_count += u64::from(node.dirty.accessibility);
            totals.render_dirty_count += u64::from(node.dirty.render);
            totals.focused_count += u64::from(node.interaction.focused);
            totals.hovered_count += u64::from(node.interaction.hovered);
            totals.pressed_count += u64::from(node.interaction.pressed);
            totals.disabled_count += u64::from(node.interaction.disabled);
            totals.render_command_count += node.render_command_count;
            totals.hit_entry_count += node.hit_entry_count;
        }
        totals
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionDelta {
    pub previous_tree_id: UiTreeId,
    pub current_tree_id: UiTreeId,
    pub changes: Vec<UiEcsProjectionNodeChange>,
    pub totals: UiEcsProjectionDeltaTotals,
    pub schedule_mask: UiEcsProjectionScheduleMask,
    pub schedule_impacts: Vec<UiEcsProjectionScheduleImpact>,
    pub dirty_domain_impacts: Vec<UiEcsDirtyDomainImpact>,
}

impl UiEcsProjectionDelta {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn change(&self, node_id: UiNodeId) -> Option<&UiEcsProjectionNodeChange> {
        self.changes.iter().find(|change| change.node_id == node_id)
    }

    pub fn changes_by_kind(
        &self,
        kind: UiEcsProjectionChangeKind,
    ) -> Vec<&UiEcsProjectionNodeChange> {
        self.changes
            .iter()
            .filter(|change| change.kind == kind)
            .collect()
    }

    pub fn node_ids_by_change_kind(&self, kind: UiEcsProjectionChangeKind) -> Vec<UiNodeId> {
        self.changes_by_kind(kind)
            .into_iter()
            .map(|change| change.node_id)
            .collect()
    }

    pub fn schedule_mask(&self) -> UiEcsProjectionScheduleMask {
        projection_schedule_mask_from_changes(&self.changes)
    }

    pub fn schedule_impacts(&self) -> Vec<UiEcsProjectionScheduleImpact> {
        projection_schedule_impacts_from_changes(&self.changes)
    }

    pub fn schedule_impact(&self, stage: UiPipelineStage) -> Option<UiEcsProjectionScheduleImpact> {
        self.schedule_impacts()
            .into_iter()
            .find(|impact| impact.stage == stage)
    }

    pub fn node_ids_requiring_stage(&self, stage: UiPipelineStage) -> Vec<UiNodeId> {
        self.schedule_impact(stage)
            .map(|impact| impact.node_ids)
            .unwrap_or_default()
    }

    pub fn changes_requiring_stage(
        &self,
        stage: UiPipelineStage,
    ) -> Vec<&UiEcsProjectionNodeChange> {
        self.changes
            .iter()
            .filter(|change| {
                UiEcsProjectionScheduleMask::from_dirty_domains(change.domains)
                    .requires_stage(stage)
            })
            .collect()
    }

    pub fn dirty_domain_impacts(&self) -> Vec<UiEcsDirtyDomainImpact> {
        projection_dirty_domain_impacts_from_changes(&self.changes)
    }

    pub fn dirty_domain_impact(
        &self,
        domain: UiEcsDirtyDomainKind,
    ) -> Option<UiEcsDirtyDomainImpact> {
        self.dirty_domain_impacts()
            .into_iter()
            .find(|impact| impact.domain == domain)
    }

    pub fn node_ids_in_dirty_domain(&self, domain: UiEcsDirtyDomainKind) -> Vec<UiNodeId> {
        self.dirty_domain_impact(domain)
            .map(|impact| impact.node_ids)
            .unwrap_or_default()
    }

    pub fn changes_in_dirty_domain(
        &self,
        domain: UiEcsDirtyDomainKind,
    ) -> Vec<&UiEcsProjectionNodeChange> {
        self.changes
            .iter()
            .filter(|change| change.domains.contains(domain))
            .collect()
    }

    pub fn derived_fields_are_fresh(&self) -> bool {
        self.totals == UiEcsProjectionDeltaTotals::from_changes(&self.changes)
            && self.schedule_mask == self.schedule_mask()
            && self.schedule_impacts == self.schedule_impacts()
            && self.dirty_domain_impacts == self.dirty_domain_impacts()
    }

    pub fn component_structure_changed(&self) -> bool {
        self.changes
            .iter()
            .any(UiEcsProjectionNodeChange::changes_component_structure)
    }

    pub fn interaction_only(&self) -> bool {
        !self.changes.is_empty()
            && self
                .changes
                .iter()
                .all(UiEcsProjectionNodeChange::is_interaction_only)
    }

    pub fn component_structure_change_node_ids(&self) -> Vec<UiNodeId> {
        self.changes
            .iter()
            .filter(|change| change.changes_component_structure())
            .map(|change| change.node_id)
            .collect()
    }

    pub fn interaction_change_node_ids(&self) -> Vec<UiNodeId> {
        self.changes
            .iter()
            .filter(|change| change.is_interaction_change())
            .map(|change| change.node_id)
            .collect()
    }

    pub fn interaction_only_change_node_ids(&self) -> Vec<UiNodeId> {
        self.changes
            .iter()
            .filter(|change| change.is_interaction_only())
            .map(|change| change.node_id)
            .collect()
    }

    pub fn render_only_change_node_ids(&self) -> Vec<UiNodeId> {
        self.changes
            .iter()
            .filter(|change| change.is_render_only_change())
            .map(|change| change.node_id)
            .collect()
    }

    pub fn recompute_schedule_mask(&mut self) {
        self.schedule_mask = self.schedule_mask();
    }

    pub fn recompute_schedule_impacts(&mut self) {
        self.schedule_impacts = self.schedule_impacts();
    }

    pub fn recompute_dirty_domain_impacts(&mut self) {
        self.dirty_domain_impacts = self.dirty_domain_impacts();
    }

    pub fn recompute_derived_fields(&mut self) {
        self.recompute_totals();
        self.recompute_schedule_mask();
        self.recompute_schedule_impacts();
        self.recompute_dirty_domain_impacts();
    }

    pub fn recompute_totals(&mut self) {
        self.totals = UiEcsProjectionDeltaTotals::from_changes(&self.changes);
    }

    pub fn with_recomputed_totals(mut self) -> Self {
        self.recompute_totals();
        self
    }

    pub fn with_recomputed_schedule_mask(mut self) -> Self {
        self.recompute_schedule_mask();
        self
    }

    pub fn with_recomputed_schedule_impacts(mut self) -> Self {
        self.recompute_schedule_impacts();
        self
    }

    pub fn with_recomputed_dirty_domain_impacts(mut self) -> Self {
        self.recompute_dirty_domain_impacts();
        self
    }

    pub fn with_recomputed_derived_fields(mut self) -> Self {
        self.recompute_derived_fields();
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionNodeChange {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub kind: UiEcsProjectionChangeKind,
    pub domains: UiEcsDirtyDomains,
    pub reasons: Vec<UiEcsProjectionChangeReason>,
}

impl UiEcsProjectionNodeChange {
    pub fn changes_component_structure(&self) -> bool {
        matches!(
            self.kind,
            UiEcsProjectionChangeKind::Added | UiEcsProjectionChangeKind::Removed
        ) || self.reasons.iter().any(|reason| {
            matches!(
                reason,
                UiEcsProjectionChangeReason::NodePath
                    | UiEcsProjectionChangeReason::Parent
                    | UiEcsProjectionChangeReason::Children
                    | UiEcsProjectionChangeReason::Component
            )
        })
    }

    pub fn is_interaction_change(&self) -> bool {
        self.reasons
            .contains(&UiEcsProjectionChangeReason::Interaction)
    }

    pub fn is_render_only_change(&self) -> bool {
        self.domains.render
            && !self.domains.layout
            && !self.domains.text
            && !self.domains.input
            && !self.domains.picking
            && !self.domains.accessibility
            && !self.domains.style
            && !self.domains.visible_range
    }

    pub fn is_interaction_only(&self) -> bool {
        self.kind == UiEcsProjectionChangeKind::Updated
            && !self.changes_component_structure()
            && !self.reasons.is_empty()
            && self
                .reasons
                .iter()
                .all(|reason| *reason == UiEcsProjectionChangeReason::Interaction)
            && self.domains.input
            && self.domains.accessibility
            && self.domains.render
            && !self.domains.layout
            && !self.domains.text
            && !self.domains.picking
            && !self.domains.style
            && !self.domains.visible_range
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEcsProjectionChangeKind {
    Added,
    Removed,
    #[default]
    Updated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiEcsProjectionChangeReason {
    Added,
    Removed,
    NodePath,
    Parent,
    Children,
    Component,
    ControlId,
    Frame,
    DirtyDomains,
    Interaction,
    RenderCommandCount,
    HitEntryCount,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionDeltaTotals {
    pub changed_node_count: u64,
    pub added_count: u64,
    pub removed_count: u64,
    pub updated_count: u64,
    pub layout_dirty_count: u64,
    pub text_dirty_count: u64,
    pub input_dirty_count: u64,
    pub picking_dirty_count: u64,
    pub accessibility_dirty_count: u64,
    pub render_dirty_count: u64,
    pub component_structure_change_count: u64,
    pub interaction_change_count: u64,
    pub render_only_change_count: u64,
}

impl UiEcsProjectionDeltaTotals {
    pub fn from_changes(changes: &[UiEcsProjectionNodeChange]) -> Self {
        let mut totals = Self {
            changed_node_count: changes.len() as u64,
            ..Self::default()
        };
        for change in changes {
            match change.kind {
                UiEcsProjectionChangeKind::Added => totals.added_count += 1,
                UiEcsProjectionChangeKind::Removed => totals.removed_count += 1,
                UiEcsProjectionChangeKind::Updated => totals.updated_count += 1,
            }
            totals.layout_dirty_count += u64::from(change.domains.layout);
            totals.text_dirty_count += u64::from(change.domains.text);
            totals.input_dirty_count += u64::from(change.domains.input);
            totals.picking_dirty_count += u64::from(change.domains.picking);
            totals.accessibility_dirty_count += u64::from(change.domains.accessibility);
            totals.render_dirty_count += u64::from(change.domains.render);
            totals.component_structure_change_count +=
                u64::from(change.changes_component_structure());
            totals.interaction_change_count += u64::from(change.is_interaction_change());
            totals.render_only_change_count += u64::from(change.is_render_only_change());
        }
        totals
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionScheduleMask {
    pub input_collect: bool,
    pub focus: bool,
    pub widget_behavior: bool,
    pub text_measure: bool,
    pub layout: bool,
    pub post_layout: bool,
    pub picking: bool,
    pub a11y_extract: bool,
    pub render_extract: bool,
    pub batch_prepare: bool,
}

impl UiEcsProjectionScheduleMask {
    pub const fn from_dirty_domains(domains: UiEcsDirtyDomains) -> Self {
        let text_layout = domains.text;
        let layout = domains.layout || text_layout || domains.style || domains.visible_range;
        let render = domains.render || layout || domains.text || domains.style;
        Self {
            input_collect: domains.input,
            focus: domains.input || domains.layout || domains.visible_range,
            widget_behavior: domains.input,
            text_measure: domains.text,
            layout,
            post_layout: layout,
            picking: domains.picking || layout,
            a11y_extract: domains.accessibility || layout || domains.text,
            render_extract: render,
            batch_prepare: render,
        }
    }

    pub const fn is_empty(self) -> bool {
        !self.input_collect
            && !self.focus
            && !self.widget_behavior
            && !self.text_measure
            && !self.layout
            && !self.post_layout
            && !self.picking
            && !self.a11y_extract
            && !self.render_extract
            && !self.batch_prepare
    }

    pub const fn requires_stage(self, stage: UiPipelineStage) -> bool {
        match stage {
            UiPipelineStage::InputCollect => self.input_collect,
            UiPipelineStage::Focus => self.focus,
            UiPipelineStage::WidgetBehavior => self.widget_behavior,
            UiPipelineStage::TextMeasure => self.text_measure,
            UiPipelineStage::Layout => self.layout,
            UiPipelineStage::PostLayout => self.post_layout,
            UiPipelineStage::Picking => self.picking,
            UiPipelineStage::A11yExtract => self.a11y_extract,
            UiPipelineStage::RenderExtract => self.render_extract,
            UiPipelineStage::BatchPrepare => self.batch_prepare,
            UiPipelineStage::FocusInteraction
            | UiPipelineStage::ContentMeasure
            | UiPipelineStage::PostLayoutStack
            | UiPipelineStage::HitGrid
            | UiPipelineStage::PaintSubmit
            | UiPipelineStage::Diagnostics => false,
        }
    }

    pub fn pipeline_stages(self) -> Vec<UiPipelineStage> {
        UiPipelineStage::ordered()
            .iter()
            .copied()
            .filter(|stage| self.requires_stage(*stage))
            .collect()
    }

    pub fn dirty_reasons(self) -> Vec<UiPipelineDirtyReason> {
        let mut reasons = Vec::new();
        if self.input_collect {
            reasons.push(UiPipelineDirtyReason::Input);
        }
        if self.focus {
            reasons.push(UiPipelineDirtyReason::Focus);
        }
        if self.widget_behavior {
            reasons.push(UiPipelineDirtyReason::WidgetBehavior);
        }
        if self.text_measure {
            reasons.push(UiPipelineDirtyReason::Text);
        }
        if self.layout || self.post_layout {
            reasons.push(UiPipelineDirtyReason::Layout);
        }
        if self.picking {
            reasons.push(UiPipelineDirtyReason::Picking);
        }
        if self.a11y_extract {
            reasons.push(UiPipelineDirtyReason::A11y);
        }
        if self.render_extract || self.batch_prepare {
            reasons.push(UiPipelineDirtyReason::Render);
        }
        reasons
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsProjectionScheduleImpact {
    pub stage: UiPipelineStage,
    pub required: bool,
    pub dirty_reasons: Vec<UiPipelineDirtyReason>,
    pub node_ids: Vec<UiNodeId>,
    pub node_count: u64,
}

impl Default for UiEcsProjectionScheduleImpact {
    fn default() -> Self {
        Self {
            stage: UiPipelineStage::InputCollect,
            required: false,
            dirty_reasons: Vec::new(),
            node_ids: Vec::new(),
            node_count: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiEcsDirtyDomainImpact {
    pub domain: UiEcsDirtyDomainKind,
    pub active: bool,
    pub node_ids: Vec<UiNodeId>,
    pub node_count: u64,
}

impl Default for UiEcsDirtyDomainImpact {
    fn default() -> Self {
        Self {
            domain: UiEcsDirtyDomainKind::Layout,
            active: false,
            node_ids: Vec::new(),
            node_count: 0,
        }
    }
}
