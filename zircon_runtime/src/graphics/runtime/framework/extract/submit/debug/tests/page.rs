use super::unique_entity_count;

#[test]
fn page_inspections_use_prebuilt_page_and_resident_indices() {
    let source = include_str!("../page.rs");

    assert!(source.contains("build_page_size_index"));
    assert!(source.contains("resident_by_page_id"));
    assert!(!source.contains(concat!("extract", ".pages", ".iter()", ".find")));
    assert!(!source.contains(concat!("resident_page_inspections", ".iter()", ".find")));
}

#[test]
fn optimization_batch_20260830cu_unique_entity_hash_count_preserves_duplicate_semantics() {
    let entities = [9, 2, 9, 4, 2, 7, 4, 7, 11];

    assert_eq!(unique_entity_count(entities.into_iter(), entities.len()), 5);
    assert_eq!(unique_entity_count(std::iter::empty(), 0), 0);
}

#[test]
fn optimization_batch_20260830cu_unique_entity_hash_count_source_contract() {
    let source = include_str!("../page.rs");
    let unique_count = source
        .split("fn unique_extract_entity_count")
        .nth(1)
        .expect("unique entity count implementation")
        .split("pub(super) fn build_resident_page_inspections")
        .next()
        .expect("bounded unique entity count implementation");

    assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(unique_count.contains("HashSet::with_capacity(capacity)"));
    assert!(!unique_count.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cu_runtime_unique_entity_hash_count_p95() {
    fn measure(entities: &[u64], count: impl Fn(&[u64]) -> usize) -> u128 {
        let started = std::time::Instant::now();
        for _ in 0..16 {
            std::hint::black_box(count(std::hint::black_box(entities)));
        }
        started.elapsed().as_nanos()
    }

    let entities = (0..65_536_u64)
        .map(|index| index.wrapping_mul(2_654_435_761) % 32_768)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        let legacy = |values: &[u64]| {
            values
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        };
        let optimized =
            |values: &[u64]| unique_entity_count(values.iter().copied(), values.len()) as usize;
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&entities, legacy));
            optimized_samples.push(measure(&entities, optimized));
        } else {
            optimized_samples.push(measure(&entities, optimized));
            legacy_samples.push(measure(&entities, legacy));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "RUNTIME397_UNIQUE_ENTITY_HASH_COUNT_BENCH_V1 entities={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        entities.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "hash unique-count P95 {optimized_p95} ns exceeded 70% of tree unique-count {legacy_p95} ns"
    );
}

fn sparse_debug_domain(
    raw_count: usize,
) -> crate::core::framework::render::RenderVirtualGeometryExtract {
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
        RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance,
        RenderVirtualGeometryPage,
    };
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); raw_count];
    for (index, id, page) in [(0, 20, 200), (1, 30, 300), (70, 70, 200), (71, 71, 300)] {
        clusters[index] = RenderVirtualGeometryCluster {
            entity: 5,
            cluster_id: id,
            page_id: page,
            ..Default::default()
        };
    }
    clusters[1].hierarchy_node_id = Some(72);
    RenderVirtualGeometryExtract {
        clusters,
        instances: vec![RenderVirtualGeometryInstance {
            entity: 5,
            cluster_count: 2,
            page_count: 2,
            ..Default::default()
        }],
        pages: vec![
            RenderVirtualGeometryPage {
                page_id: 200,
                ..Default::default()
            },
            RenderVirtualGeometryPage {
                page_id: 300,
                ..Default::default()
            },
        ],
        hierarchy_nodes: vec![
            RenderVirtualGeometryHierarchyNode {
                node_id: 72,
                child_count: 1,
                cluster_start: 1,
                cluster_count: 1,
                ..Default::default()
            },
            RenderVirtualGeometryHierarchyNode {
                node_id: 7,
                cluster_start: 70,
                cluster_count: 2,
                ..Default::default()
            },
        ],
        hierarchy_child_ids: vec![7],
        ..Default::default()
    }
}

#[test]
fn sparse_debug_catalog_preserves_raw_census_and_normal_candidate_ordinals() {
    use std::collections::BTreeSet;
    for raw_count in [72, 91] {
        let extract = sparse_debug_domain(raw_count);
        let plan = crate::graphics::VisibilityVirtualGeometryPageUploadPlan::default();
        let snapshot =
            super::build_cull_input_snapshot(&extract, &plan, 0, 0, &BTreeSet::new()).unwrap();
        assert_eq!(snapshot.cluster_count, raw_count as u32);
        assert_eq!(snapshot.visible_cluster_count, raw_count as u32);
        assert_eq!(snapshot.visible_entity_count, 1);
        let (catalog, provenance) = super::super::build_lossless_candidate_catalog(
            &extract,
            &BTreeSet::from([200]),
            &BTreeSet::from([300]),
            &BTreeSet::new(),
        )
        .unwrap();
        assert_eq!(
            catalog
                .iter()
                .map(|row| (row.cluster_id, row.cluster_ordinal))
                .collect::<Vec<_>>(),
            vec![(20, 0), (30, 1)]
        );
        assert_eq!(provenance, vec![(0, 0, 2), (0, 1, 2)]);
        assert_eq!(
            extract
                .checked_active_cluster_rows(&BTreeSet::new())
                .unwrap()
                .len(),
            4
        );
        assert_eq!(extract.clusters.len(), raw_count);
    }
}

#[test]
fn sparse_debug_legacy_catalog_uses_real_visible_entities_and_rejects_invalid_domains() {
    use std::collections::BTreeSet;
    let mut extract = sparse_debug_domain(72);
    extract.clusters[8].entity = 7;
    extract.clusters[8].cluster_id = 80;
    extract.clusters[9].entity = 8;
    extract.clusters[9].cluster_id = 90;
    let visible = BTreeSet::from([7]);
    let plan = crate::graphics::VisibilityVirtualGeometryPageUploadPlan::default();
    let snapshot = super::build_cull_input_snapshot(&extract, &plan, 0, 0, &visible).unwrap();
    assert_eq!(snapshot.visible_entity_count, 2);
    assert_eq!(snapshot.visible_cluster_count, 72);
    let (catalog, provenance) = super::super::build_lossless_candidate_catalog(
        &extract,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &visible,
    )
    .unwrap();
    assert_eq!(
        catalog.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![5, 5, 7]
    );
    assert_eq!(provenance, vec![(0, 0, 2), (0, 1, 2), (u32::MAX, 8, 1)]);
    extract.hierarchy_nodes[1].cluster_count = 3;
    assert!(super::build_cull_input_snapshot(&extract, &plan, 0, 0, &visible).is_err());
    assert!(super::super::build_lossless_candidate_catalog(
        &extract,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &visible,
    )
    .is_none());
}
