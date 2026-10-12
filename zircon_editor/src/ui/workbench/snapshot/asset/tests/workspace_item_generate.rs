use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::resource::ResourceKind;

use super::super::AssetTypeProjectionSnapshot;
use super::*;

fn item(index: usize) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: format!("asset-{index}"),
        locator: format!("res://asset-{index}.zdata"),
        display_name: format!("Asset {index}"),
        file_name: format!("asset-{index}.zdata"),
        extension: "zdata".to_string(),
        kind: ResourceKind::Data,
        asset_type: AssetTypeProjectionSnapshot::default(),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected: index % 3 == 0,
        resource_state: None,
        resource_revision: None,
    }
}

fn legacy_project_items_reusing(
    source: &AssetWorkspaceItemGeneration,
    previous_source: &AssetWorkspaceItemGeneration,
    previous_projected: &AssetWorkspaceItemGeneration,
) -> AssetWorkspaceItemGeneration {
    let mut chunks = Vec::with_capacity(source.chunks.len());
    let mut selected_indices = previous_projected.selected_indices.to_vec();
    for (index, chunk) in source.chunks.iter().enumerate() {
        if Arc::ptr_eq(chunk, &previous_source.chunks[index]) {
            chunks.push(Arc::clone(&previous_projected.chunks[index]));
        } else {
            let chunk = project_chunk(chunk, &mut |_| {});
            replace_chunk_selected_indices(&mut selected_indices, index, &chunk);
            chunks.push(chunk);
        }
    }
    AssetWorkspaceItemGeneration {
        chunks: chunks.into(),
        len: source.len,
        indices_by_uuid: Arc::clone(&source.indices_by_uuid),
        indices_by_locator: Arc::clone(&source.indices_by_locator),
        selected_indices: selected_indices.into(),
    }
}

#[test]
fn optimization_batch_gz_editor581_full_reuse_shares_projected_arrays() {
    let source = (0..256).map(item).collect::<AssetWorkspaceItemGeneration>();
    let projected = source.project_items(|item| item.display_name.push_str(" projected"));
    let next = source.project_items_reusing(&source, &projected, |item| {
        item.display_name.push_str(" should-not-run")
    });

    assert!(next.shares_items_with(&projected));
    assert!(next.shares_item_identity_with(&projected));
    assert!(next.shares_selected_indices_with(&projected));
    assert_eq!(next[42].display_name, "Asset 42 projected");
}

#[test]
fn from_iterator_streams_rows_without_changing_chunk_or_index_semantics() {
    let generation = (0..130)
        .map(item)
        .filter(|item| item.uuid.starts_with("asset-"))
        .collect::<AssetWorkspaceItemGeneration>();

    assert_eq!(generation.len(), 130);
    assert_eq!(
        generation.get(0).map(|item| item.uuid.as_str()),
        Some("asset-0")
    );
    assert_eq!(
        generation.get(64).map(|item| item.uuid.as_str()),
        Some("asset-64")
    );
    assert_eq!(
        generation.get(129).map(|item| item.uuid.as_str()),
        Some("asset-129")
    );
    assert_eq!(generation.selected_index("asset-129"), Some(129));
    assert_eq!(generation.locator_index("res://asset-64.zdata"), Some(64));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_editor741_streaming_generation_benchmark() {
    fn legacy_with_temporary_items(
        items: impl IntoIterator<Item = AssetItemSnapshot>,
    ) -> AssetWorkspaceItemGeneration {
        let items = items.into_iter().collect::<Vec<_>>();
        items.into()
    }

    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_with_temporary_items((0..8_192).map(item)));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        black_box(
            (0..8_192)
                .map(item)
                .collect::<AssetWorkspaceItemGeneration>(),
        );
        optimized_samples.push(started.elapsed().as_nanos());
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR741_ITEM_GENERATION_STREAM_BENCH_V1 item_count=8192 legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=8000",
        legacy_p95, optimized_p95
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_000),
        "streaming generation P95 {optimized_p95} ns exceeded 80% of legacy {legacy_p95} ns"
    );
}

#[test]
fn replacement_grouping_reserves_bounded_chunk_capacity() {
    let source = include_str!("../asset_workspace_item_generation.rs");
    let replace_existing_items = source
        .split_once("    pub(crate) fn replace_existing_items(")
        .and_then(|(_, remainder)| remainder.split_once("    pub(crate) fn project_items("))
        .map(|(body, _)| body)
        .expect("asset generation must keep a dedicated replacement path");

    assert!(replace_existing_items.contains("let (lower_bound, _) = replacements.size_hint();"));
    assert!(replace_existing_items.contains("lower_bound.min(self.chunks.len())"));
    assert!(replace_existing_items
        .contains("HashMap::<usize, Vec<(usize, AssetItemSnapshot)>>::with_capacity"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_gz_editor581_project_reuse_performance_evidence() {
    let source = (0..8_192)
        .map(item)
        .collect::<AssetWorkspaceItemGeneration>();
    let projected = source.project_items(|item| item.display_name.push_str(" projected"));
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_project_items_reusing(
            black_box(&source),
            black_box(&source),
            black_box(&projected),
        ));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        black_box(source.project_items_reusing(black_box(&source), black_box(&projected), |_| {}));
        optimized_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR581_PROJECT_REUSE_BENCH_V1 item_count={} chunks={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        source.len,
        source.chunks.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "projected asset reuse P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
