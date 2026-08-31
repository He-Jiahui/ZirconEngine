use crate::ui::{event_ui::UiNodeId, pipeline::UiPipelineStage};

use super::{
    UiEcsDirtyDomainImpact, UiEcsDirtyDomainKind, UiEcsDirtyDomains, UiEcsNodeProjection,
    UiEcsProjectionNodeChange, UiEcsProjectionScheduleImpact, UiEcsProjectionScheduleMask,
    projection_stage_dirty_reasons,
};

pub(super) fn projection_schedule_impact_from_nodes(
    nodes: &[UiEcsNodeProjection],
    stage: UiPipelineStage,
) -> Option<UiEcsProjectionScheduleImpact> {
    projection_schedule_impact_from_domains(
        nodes.iter().map(|node| (node.node_id, node.dirty)),
        stage,
    )
}

pub(super) fn projection_schedule_impact_from_changes(
    changes: &[UiEcsProjectionNodeChange],
    stage: UiPipelineStage,
) -> Option<UiEcsProjectionScheduleImpact> {
    projection_schedule_impact_from_domains(
        changes
            .iter()
            .map(|change| (change.node_id, change.domains)),
        stage,
    )
}

fn projection_schedule_impact_from_domains<I>(
    domains_by_node: I,
    stage: UiPipelineStage,
) -> Option<UiEcsProjectionScheduleImpact>
where
    I: IntoIterator<Item = (UiNodeId, UiEcsDirtyDomains)>,
{
    if !stage.is_runtime_schedule_stage() {
        return None;
    }

    let mut node_ids = Vec::new();
    let mut impacted_domains = UiEcsDirtyDomains::default();
    for (node_id, domains) in domains_by_node {
        let node_mask = UiEcsProjectionScheduleMask::from_dirty_domains(domains);
        if !node_mask.requires_stage(stage) {
            continue;
        }
        node_ids.push(node_id);
        impacted_domains = impacted_domains.union(domains);
    }

    if node_ids.is_empty() {
        return None;
    }
    node_ids.sort_unstable();
    node_ids.dedup();
    Some(UiEcsProjectionScheduleImpact {
        stage,
        required: true,
        node_count: node_ids.len() as u64,
        dirty_reasons: projection_stage_dirty_reasons(stage, impacted_domains),
        node_ids,
    })
}

pub(super) fn projection_dirty_domain_impact_from_nodes(
    nodes: &[UiEcsNodeProjection],
    domain: UiEcsDirtyDomainKind,
) -> Option<UiEcsDirtyDomainImpact> {
    projection_dirty_domain_impact_from_domains(
        nodes.iter().map(|node| (node.node_id, node.dirty)),
        domain,
    )
}

pub(super) fn projection_dirty_domain_impact_from_changes(
    changes: &[UiEcsProjectionNodeChange],
    domain: UiEcsDirtyDomainKind,
) -> Option<UiEcsDirtyDomainImpact> {
    projection_dirty_domain_impact_from_domains(
        changes
            .iter()
            .map(|change| (change.node_id, change.domains)),
        domain,
    )
}

fn projection_dirty_domain_impact_from_domains<I>(
    domains_by_node: I,
    domain: UiEcsDirtyDomainKind,
) -> Option<UiEcsDirtyDomainImpact>
where
    I: IntoIterator<Item = (UiNodeId, UiEcsDirtyDomains)>,
{
    let mut node_ids: Vec<_> = domains_by_node
        .into_iter()
        .filter_map(|(node_id, domains)| domains.contains(domain).then_some(node_id))
        .collect();
    if node_ids.is_empty() {
        return None;
    }
    node_ids.sort_unstable();
    node_ids.dedup();
    Some(UiEcsDirtyDomainImpact {
        domain,
        active: true,
        node_count: node_ids.len() as u64,
        node_ids,
    })
}
