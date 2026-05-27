use std::collections::{BTreeMap, BTreeSet};

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
    let entries = domains_by_node
        .into_iter()
        .filter(|(_, domains)| domains.any())
        .collect::<Vec<_>>();
    let mut impacts = Vec::new();

    for domain in UiEcsDirtyDomainKind::ordered().iter().copied() {
        let node_ids = entries
            .iter()
            .filter_map(|(node_id, domains)| domains.contains(domain).then_some(*node_id))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if node_ids.is_empty() {
            continue;
        }
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
    let entries = domains_by_node
        .into_iter()
        .filter(|(_, domains)| domains.any())
        .collect::<Vec<_>>();
    let aggregate_domains = entries.iter().fold(
        UiEcsDirtyDomains::default(),
        |domains, (_, node_domains)| domains.union(*node_domains),
    );
    let aggregate_mask = UiEcsProjectionScheduleMask::from_dirty_domains(aggregate_domains);
    let mut impacts = Vec::new();

    for stage in UiPipelineStage::ordered().iter().copied() {
        if !aggregate_mask.requires_stage(stage) {
            continue;
        }

        let mut node_ids = BTreeSet::new();
        let mut dirty_reasons = BTreeSet::new();
        for (node_id, domains) in &entries {
            if !UiEcsProjectionScheduleMask::from_dirty_domains(*domains).requires_stage(stage) {
                continue;
            }
            node_ids.insert(*node_id);
            dirty_reasons.extend(projection_stage_dirty_reasons(stage, *domains));
        }

        let node_ids = node_ids.into_iter().collect::<Vec<_>>();
        impacts.push(UiEcsProjectionScheduleImpact {
            stage,
            required: true,
            node_count: node_ids.len() as u64,
            dirty_reasons: dirty_reasons.into_iter().collect(),
            node_ids,
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

    let mut reasons = Vec::new();
    match stage {
        UiPipelineStage::InputCollect => {
            if domains.input {
                reasons.push(UiPipelineDirtyReason::Input);
            }
        }
        UiPipelineStage::Focus => {
            if domains.input {
                reasons.push(UiPipelineDirtyReason::Input);
            }
            push_layout_driver_reasons(&mut reasons, domains);
            reasons.push(UiPipelineDirtyReason::Focus);
        }
        UiPipelineStage::WidgetBehavior => {
            if domains.input {
                reasons.push(UiPipelineDirtyReason::Input);
            }
            reasons.push(UiPipelineDirtyReason::WidgetBehavior);
        }
        UiPipelineStage::TextMeasure => {
            if domains.text {
                reasons.push(UiPipelineDirtyReason::Text);
            }
        }
        UiPipelineStage::Layout | UiPipelineStage::PostLayout => {
            push_layout_driver_reasons(&mut reasons, domains);
        }
        UiPipelineStage::Picking => {
            push_layout_driver_reasons(&mut reasons, domains);
            if domains.picking {
                reasons.push(UiPipelineDirtyReason::Picking);
            }
        }
        UiPipelineStage::A11yExtract => {
            if domains.accessibility {
                reasons.push(UiPipelineDirtyReason::A11y);
            }
            if domains.input {
                reasons.push(UiPipelineDirtyReason::Input);
            }
            push_layout_driver_reasons(&mut reasons, domains);
        }
        UiPipelineStage::RenderExtract | UiPipelineStage::BatchPrepare => {
            if domains.render {
                reasons.push(UiPipelineDirtyReason::Render);
            }
            push_layout_driver_reasons(&mut reasons, domains);
        }
        UiPipelineStage::FocusInteraction
        | UiPipelineStage::ContentMeasure
        | UiPipelineStage::PostLayoutStack
        | UiPipelineStage::HitGrid
        | UiPipelineStage::PaintSubmit
        | UiPipelineStage::Diagnostics => {}
    }

    if reasons.is_empty() {
        reasons.extend(node_mask.dirty_reasons());
    }
    reasons.sort();
    reasons.dedup();
    reasons
}

fn push_layout_driver_reasons(
    reasons: &mut Vec<UiPipelineDirtyReason>,
    domains: UiEcsDirtyDomains,
) {
    if domains.text {
        reasons.push(UiPipelineDirtyReason::Text);
    }
    if domains.style {
        reasons.push(UiPipelineDirtyReason::Style);
    }
    if domains.layout || domains.text || domains.style || domains.visible_range {
        reasons.push(UiPipelineDirtyReason::Layout);
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
