use std::collections::BTreeMap;

use crate::ui::{
    event_ui::UiNodeId,
    pipeline::{UiPipelineDirtyReason, UiPipelineStage},
};

use super::{
    UiEcsDirtyDomainImpact, UiEcsDirtyDomainKind, UiEcsDirtyDomains, UiEcsNodeProjection,
    UiEcsProjectionChangeReason, UiEcsProjectionNodeChange, UiEcsProjectionScheduleImpact,
    UiEcsProjectionScheduleMask,
};

pub(super) fn projection_node_map(
    nodes: &[UiEcsNodeProjection],
) -> BTreeMap<UiNodeId, &UiEcsNodeProjection> {
    nodes.iter().map(|node| (node.node_id, node)).collect()
}

pub(super) fn projection_schedule_mask_from_nodes(
    nodes: &[UiEcsNodeProjection],
) -> UiEcsProjectionScheduleMask {
    let domains = nodes
        .iter()
        .fold(UiEcsDirtyDomains::default(), |domains, node| {
            domains.union(node.dirty)
        });
    UiEcsProjectionScheduleMask::from_dirty_domains(domains)
}

pub(super) fn projection_schedule_mask_from_changes(
    changes: &[UiEcsProjectionNodeChange],
) -> UiEcsProjectionScheduleMask {
    let domains = changes
        .iter()
        .fold(UiEcsDirtyDomains::default(), |domains, change| {
            domains.union(change.domains)
        });
    UiEcsProjectionScheduleMask::from_dirty_domains(domains)
}

pub(super) fn projection_schedule_impacts_from_nodes(
    nodes: &[UiEcsNodeProjection],
) -> Vec<UiEcsProjectionScheduleImpact> {
    projection_schedule_impacts_from_domains(nodes.iter().map(|node| (node.node_id, node.dirty)))
}

pub(super) fn projection_schedule_impacts_from_changes(
    changes: &[UiEcsProjectionNodeChange],
) -> Vec<UiEcsProjectionScheduleImpact> {
    projection_schedule_impacts_from_domains(
        changes
            .iter()
            .map(|change| (change.node_id, change.domains)),
    )
}

pub(super) fn projection_dirty_domain_impacts_from_nodes(
    nodes: &[UiEcsNodeProjection],
) -> Vec<UiEcsDirtyDomainImpact> {
    projection_dirty_domain_impacts_from_domains(
        nodes.iter().map(|node| (node.node_id, node.dirty)),
    )
}

pub(super) fn projection_dirty_domain_impacts_from_changes(
    changes: &[UiEcsProjectionNodeChange],
) -> Vec<UiEcsDirtyDomainImpact> {
    projection_dirty_domain_impacts_from_domains(
        changes
            .iter()
            .map(|change| (change.node_id, change.domains)),
    )
}

pub(super) fn projection_dirty_domain_impacts_from_domains<I>(
    domains_by_node: I,
) -> Vec<UiEcsDirtyDomainImpact>
where
    I: IntoIterator<Item = (UiNodeId, UiEcsDirtyDomains)>,
{
    let mut node_ids_by_domain: [Vec<UiNodeId>; UiEcsDirtyDomainKind::ORDER.len()] =
        std::array::from_fn(|_| Vec::new());
    for (node_id, domains) in domains_by_node {
        if !domains.any() {
            continue;
        }
        for (domain_index, domain) in UiEcsDirtyDomainKind::ordered().iter().copied().enumerate() {
            if domains.contains(domain) {
                node_ids_by_domain[domain_index].push(node_id);
            }
        }
    }

    let mut impacts = Vec::new();
    for (domain, mut node_ids) in UiEcsDirtyDomainKind::ordered()
        .iter()
        .copied()
        .zip(node_ids_by_domain)
    {
        if node_ids.is_empty() {
            continue;
        }
        node_ids.sort_unstable();
        node_ids.dedup();
        impacts.push(UiEcsDirtyDomainImpact {
            domain,
            active: true,
            node_count: node_ids.len() as u64,
            node_ids,
        });
    }

    impacts
}

pub(super) fn projection_schedule_impacts_from_domains<I>(
    domains_by_node: I,
) -> Vec<UiEcsProjectionScheduleImpact>
where
    I: IntoIterator<Item = (UiNodeId, UiEcsDirtyDomains)>,
{
    let mut buckets: [ProjectionScheduleImpactBucket; UiPipelineStage::ORDER.len()] =
        std::array::from_fn(|_| ProjectionScheduleImpactBucket::default());
    for (node_id, domains) in domains_by_node {
        if !domains.any() {
            continue;
        }
        let node_mask = UiEcsProjectionScheduleMask::from_dirty_domains(domains);
        for (stage_index, stage) in UiPipelineStage::ordered().iter().copied().enumerate() {
            if !node_mask.requires_stage(stage) {
                continue;
            }
            let bucket = &mut buckets[stage_index];
            bucket.node_ids.push(node_id);
            insert_projection_stage_dirty_reasons(&mut bucket.dirty_reasons, stage, domains);
        }
    }

    let mut impacts = Vec::new();
    for (stage, mut bucket) in UiPipelineStage::ordered().iter().copied().zip(buckets) {
        if bucket.node_ids.is_empty() {
            continue;
        }
        bucket.node_ids.sort_unstable();
        bucket.node_ids.dedup();
        impacts.push(UiEcsProjectionScheduleImpact {
            stage,
            required: true,
            node_count: bucket.node_ids.len() as u64,
            dirty_reasons: bucket.dirty_reasons.into_vec(),
            node_ids: bucket.node_ids,
        });
    }

    impacts
}

pub(super) fn projection_stage_dirty_reasons(
    stage: UiPipelineStage,
    domains: UiEcsDirtyDomains,
) -> Vec<UiPipelineDirtyReason> {
    let node_mask = UiEcsProjectionScheduleMask::from_dirty_domains(domains);
    if !node_mask.requires_stage(stage) {
        return Vec::new();
    }

    let mut reasons = ProjectionDirtyReasonMask::default();
    insert_projection_stage_dirty_reasons(&mut reasons, stage, domains);
    reasons.into_vec()
}

#[derive(Default)]
struct ProjectionScheduleImpactBucket {
    node_ids: Vec<UiNodeId>,
    dirty_reasons: ProjectionDirtyReasonMask,
}

#[derive(Clone, Copy, Default)]
struct ProjectionDirtyReasonMask(u16);

impl ProjectionDirtyReasonMask {
    fn insert(&mut self, reason: UiPipelineDirtyReason) {
        self.0 |= Self::bit(reason);
    }

    fn into_vec(self) -> Vec<UiPipelineDirtyReason> {
        let ordered = [
            UiPipelineDirtyReason::Input,
            UiPipelineDirtyReason::Focus,
            UiPipelineDirtyReason::WidgetBehavior,
            UiPipelineDirtyReason::Text,
            UiPipelineDirtyReason::Style,
            UiPipelineDirtyReason::Layout,
            UiPipelineDirtyReason::LayoutMetrics,
            UiPipelineDirtyReason::Picking,
            UiPipelineDirtyReason::HitGrid,
            UiPipelineDirtyReason::A11y,
            UiPipelineDirtyReason::Render,
            UiPipelineDirtyReason::Template,
            UiPipelineDirtyReason::Window,
            UiPipelineDirtyReason::HostRequest,
            UiPipelineDirtyReason::Diagnostics,
        ];
        let mut reasons = Vec::with_capacity(self.0.count_ones() as usize);
        reasons.extend(ordered.into_iter().filter(|reason| self.contains(*reason)));
        reasons
    }

    fn contains(self, reason: UiPipelineDirtyReason) -> bool {
        self.0 & Self::bit(reason) != 0
    }

    fn bit(reason: UiPipelineDirtyReason) -> u16 {
        1 << match reason {
            UiPipelineDirtyReason::Input => 0,
            UiPipelineDirtyReason::Focus => 1,
            UiPipelineDirtyReason::WidgetBehavior => 2,
            UiPipelineDirtyReason::Text => 3,
            UiPipelineDirtyReason::Style => 4,
            UiPipelineDirtyReason::Layout => 5,
            UiPipelineDirtyReason::LayoutMetrics => 6,
            UiPipelineDirtyReason::Picking => 7,
            UiPipelineDirtyReason::HitGrid => 8,
            UiPipelineDirtyReason::A11y => 9,
            UiPipelineDirtyReason::Render => 10,
            UiPipelineDirtyReason::Template => 11,
            UiPipelineDirtyReason::Window => 12,
            UiPipelineDirtyReason::HostRequest => 13,
            UiPipelineDirtyReason::Diagnostics => 14,
        }
    }
}

fn insert_projection_stage_dirty_reasons(
    reasons: &mut ProjectionDirtyReasonMask,
    stage: UiPipelineStage,
    domains: UiEcsDirtyDomains,
) {
    match stage {
        UiPipelineStage::InputCollect => {
            if domains.input {
                reasons.insert(UiPipelineDirtyReason::Input);
            }
        }
        UiPipelineStage::Focus => {
            if domains.input {
                reasons.insert(UiPipelineDirtyReason::Input);
            }
            insert_layout_driver_reasons(reasons, domains);
            reasons.insert(UiPipelineDirtyReason::Focus);
        }
        UiPipelineStage::WidgetBehavior => {
            if domains.input {
                reasons.insert(UiPipelineDirtyReason::Input);
            }
            reasons.insert(UiPipelineDirtyReason::WidgetBehavior);
        }
        UiPipelineStage::TextMeasure => {
            if domains.text {
                reasons.insert(UiPipelineDirtyReason::Text);
            }
        }
        UiPipelineStage::Layout | UiPipelineStage::PostLayout => {
            insert_layout_driver_reasons(reasons, domains);
        }
        UiPipelineStage::Picking => {
            insert_layout_driver_reasons(reasons, domains);
            if domains.picking {
                reasons.insert(UiPipelineDirtyReason::Picking);
            }
        }
        UiPipelineStage::A11yExtract => {
            if domains.accessibility {
                reasons.insert(UiPipelineDirtyReason::A11y);
            }
            if domains.input {
                reasons.insert(UiPipelineDirtyReason::Input);
            }
            insert_layout_driver_reasons(reasons, domains);
        }
        UiPipelineStage::RenderExtract | UiPipelineStage::BatchPrepare => {
            if domains.render {
                reasons.insert(UiPipelineDirtyReason::Render);
            }
            insert_layout_driver_reasons(reasons, domains);
        }
        UiPipelineStage::FocusInteraction
        | UiPipelineStage::ContentMeasure
        | UiPipelineStage::PostLayoutStack
        | UiPipelineStage::HitGrid
        | UiPipelineStage::PaintSubmit
        | UiPipelineStage::Diagnostics => {}
    }
}

fn insert_layout_driver_reasons(
    reasons: &mut ProjectionDirtyReasonMask,
    domains: UiEcsDirtyDomains,
) {
    if domains.text {
        reasons.insert(UiPipelineDirtyReason::Text);
    }
    if domains.style {
        reasons.insert(UiPipelineDirtyReason::Style);
    }
    if domains.layout || domains.text || domains.style || domains.visible_range {
        reasons.insert(UiPipelineDirtyReason::Layout);
    }
}

pub(super) fn projection_node_change_reasons(
    previous: &UiEcsNodeProjection,
    current: &UiEcsNodeProjection,
) -> Vec<UiEcsProjectionChangeReason> {
    let mut reasons = Vec::new();
    if previous.node_path != current.node_path {
        reasons.push(UiEcsProjectionChangeReason::NodePath);
    }
    if previous.parent != current.parent {
        reasons.push(UiEcsProjectionChangeReason::Parent);
    }
    if previous.children != current.children {
        reasons.push(UiEcsProjectionChangeReason::Children);
    }
    if previous.component != current.component {
        reasons.push(UiEcsProjectionChangeReason::Component);
    }
    if previous.control_id != current.control_id {
        reasons.push(UiEcsProjectionChangeReason::ControlId);
    }
    if previous.frame != current.frame {
        reasons.push(UiEcsProjectionChangeReason::Frame);
    }
    if previous.dirty != current.dirty {
        reasons.push(UiEcsProjectionChangeReason::DirtyDomains);
    }
    if previous.interaction != current.interaction {
        reasons.push(UiEcsProjectionChangeReason::Interaction);
    }
    if previous.render_command_count != current.render_command_count {
        reasons.push(UiEcsProjectionChangeReason::RenderCommandCount);
    }
    if previous.hit_entry_count != current.hit_entry_count {
        reasons.push(UiEcsProjectionChangeReason::HitEntryCount);
    }
    reasons
}

pub(super) fn projection_update_domains(
    previous: &UiEcsNodeProjection,
    current: &UiEcsNodeProjection,
    reasons: &[UiEcsProjectionChangeReason],
) -> UiEcsDirtyDomains {
    let mut domains = previous.dirty.union(current.dirty);
    for reason in reasons {
        domains = match reason {
            UiEcsProjectionChangeReason::Added | UiEcsProjectionChangeReason::Removed => {
                domains.union(UiEcsDirtyDomains::structural_change())
            }
            UiEcsProjectionChangeReason::NodePath
            | UiEcsProjectionChangeReason::Parent
            | UiEcsProjectionChangeReason::Children
            | UiEcsProjectionChangeReason::Frame => {
                domains.union(UiEcsDirtyDomains::structural_change())
            }
            UiEcsProjectionChangeReason::Component | UiEcsProjectionChangeReason::ControlId => {
                domains.union(UiEcsDirtyDomains {
                    accessibility: true,
                    render: true,
                    ..UiEcsDirtyDomains::default()
                })
            }
            UiEcsProjectionChangeReason::DirtyDomains => domains,
            UiEcsProjectionChangeReason::Interaction => {
                domains.union(UiEcsDirtyDomains::interaction_change())
            }
            UiEcsProjectionChangeReason::RenderCommandCount => {
                domains.union(UiEcsDirtyDomains::render_change())
            }
            UiEcsProjectionChangeReason::HitEntryCount => {
                domains.union(UiEcsDirtyDomains::picking_change())
            }
        };
    }
    domains
}

#[cfg(test)]
#[path = "tests/compute.rs"]
mod tests;

#[cfg(test)]
#[path = "compute/tests/dirty_reason_performance_tests.rs"]
mod dirty_reason_performance_tests;
