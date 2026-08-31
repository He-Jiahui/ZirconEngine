use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::{
    binding::UiBindingMutationOutcome,
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use crate::ui::surface::{UiBindingMutationTransaction, UiPropertyMutationRequest, UiSurface};

#[test]
fn binding_mutation_transaction_rolls_back_every_writable_domain() {
    let mut surface = binding_transaction_surface();
    let before = surface.clone();
    let base_generation = surface.invalidation_generations().generation;
    let transaction = UiBindingMutationTransaction::prepare(&surface, 3);

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "text",
            UiValue::String("changed".to_string()),
        ))
        .expect("transaction fixture property should mutate");
    surface.focus.focused = Some(UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    surface.component_states.set_focused(UiNodeId::new(2), true);
    surface.navigation.navigation_root = Some(UiNodeId::new(2));

    let receipt = transaction.rollback(&mut surface);

    assert_eq!(surface, before);
    assert_eq!(receipt.base_generation, base_generation);
    assert_eq!(receipt.target_count, 3);
    assert_eq!(receipt.applied_target_count, 0);
    assert_eq!(receipt.outcome, UiBindingMutationOutcome::RolledBack);
}

#[test]
fn binding_mutation_transaction_snapshot_p95_beats_whole_surface_clone() {
    const SAMPLE_PAIRS: usize = 21;
    const CLONES_PER_SAMPLE: usize = 64;
    const RETAINED_FRAME_NODES: usize = 4_096;

    let mut surface = binding_transaction_surface();
    let arranged_seed = surface.arranged_tree.nodes.clone();
    assert!(!arranged_seed.is_empty());
    while surface.arranged_tree.nodes.len() < RETAINED_FRAME_NODES {
        let remaining = RETAINED_FRAME_NODES - surface.arranged_tree.nodes.len();
        surface
            .arranged_tree
            .nodes
            .extend(arranged_seed.iter().take(remaining).cloned());
    }

    let _ = sample_full_surface_clone(&surface, 1);
    let _ = sample_binding_transaction(&surface, 1);

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_full_surface_clone(&surface, CLONES_PER_SAMPLE));
            optimized_samples_us.push(sample_binding_transaction(&surface, CLONES_PER_SAMPLE));
        } else {
            optimized_samples_us.push(sample_binding_transaction(&surface, CLONES_PER_SAMPLE));
            legacy_samples_us.push(sample_full_surface_clone(&surface, CLONES_PER_SAMPLE));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(75),
        "binding transaction P95 {optimized_p95_us}us must improve whole-surface clone P95 {legacy_p95_us}us by at least 25%"
    );
    println!(
        "PERF-RUNTIME74-BINDING-TRANSACTION sample_pairs={SAMPLE_PAIRS} clones_per_sample={CLONES_PER_SAMPLE} retained_frame_nodes={RETAINED_FRAME_NODES} legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=25 staged_surface_clones=0 snapshot_domain_groups=5",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn binding_transaction_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.binding.transaction"));
    surface
        .tree
        .insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MaterialButton".to_string(),
                    attributes: [(
                        "text".to_string(),
                        toml::Value::String("stable".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("transaction fixture child should insert");
    surface.rebuild();
    surface
}

fn sample_full_surface_clone(surface: &UiSurface, clone_count: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..clone_count {
        black_box(surface.clone());
    }
    started.elapsed().as_micros()
}

fn sample_binding_transaction(surface: &UiSurface, clone_count: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..clone_count {
        let transaction = UiBindingMutationTransaction::prepare(black_box(surface), 1);
        black_box(transaction.commit(0, 1, Vec::new(), false));
    }
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(95).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn joined_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
