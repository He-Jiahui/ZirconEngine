use super::*;

#[test]
fn projection_impact_buckets_preserve_canonical_order_and_deduplicate_nodes() {
    let domains_by_node = [
        (
            UiNodeId::new(9),
            UiEcsDirtyDomains {
                render: true,
                ..UiEcsDirtyDomains::default()
            },
        ),
        (
            UiNodeId::new(3),
            UiEcsDirtyDomains {
                text: true,
                ..UiEcsDirtyDomains::default()
            },
        ),
        (
            UiNodeId::new(9),
            UiEcsDirtyDomains {
                input: true,
                ..UiEcsDirtyDomains::default()
            },
        ),
        (
            UiNodeId::new(3),
            UiEcsDirtyDomains {
                text: true,
                ..UiEcsDirtyDomains::default()
            },
        ),
    ];

    let schedule_impacts = projection_schedule_impacts_from_domains(domains_by_node);
    assert_eq!(
        schedule_impacts
            .iter()
            .map(|impact| impact.stage)
            .collect::<Vec<_>>(),
        UiPipelineStage::ordered().to_vec()
    );
    let render_extract = schedule_impacts
        .iter()
        .find(|impact| impact.stage == UiPipelineStage::RenderExtract)
        .expect("render extract impact");
    assert_eq!(
        render_extract.node_ids,
        vec![UiNodeId::new(3), UiNodeId::new(9)]
    );
    assert_eq!(
        render_extract.dirty_reasons,
        vec![
            UiPipelineDirtyReason::Text,
            UiPipelineDirtyReason::Layout,
            UiPipelineDirtyReason::Render,
        ]
    );

    let domain_impacts = projection_dirty_domain_impacts_from_domains(domains_by_node);
    assert_eq!(
        domain_impacts
            .iter()
            .map(|impact| impact.domain)
            .collect::<Vec<_>>(),
        vec![
            UiEcsDirtyDomainKind::Text,
            UiEcsDirtyDomainKind::Input,
            UiEcsDirtyDomainKind::Render,
        ]
    );
    assert_eq!(domain_impacts[0].node_ids, vec![UiNodeId::new(3)]);
    assert_eq!(domain_impacts[1].node_ids, vec![UiNodeId::new(9)]);
    assert_eq!(domain_impacts[2].node_ids, vec![UiNodeId::new(9)]);
}
