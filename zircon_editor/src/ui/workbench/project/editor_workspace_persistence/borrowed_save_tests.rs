use std::hint::black_box;
use std::time::Instant;

use serde_json::json;
use zircon_runtime_interface::serialization::{write_versioned_text, WriteError};

use crate::ui::workbench::layout::{MainPageId, WorkbenchLayout};
use crate::ui::workbench::project::editor_workspace_document::EditorWorkspaceDocument;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hr_editor_preserves_workspace_json_and_round_trip() {
    let workspace = benchmark_workspace(4, 32);

    let optimized = encode_editor_workspace_document(&workspace).unwrap();
    let cloned = cloned_encode_editor_workspace(&workspace).unwrap();

    assert_eq!(optimized, cloned);
    assert_eq!(
        decode_editor_workspace_document(optimized.as_bytes()).unwrap(),
        workspace
    );
}

#[test]
fn optimization_batch_20260828hr_editor_save_borrows_workspace_without_clone() {
    let persistence_source = include_str!("../editor_workspace_persistence.rs");
    let document_source = include_str!("../editor_workspace_document.rs");
    let save = persistence_source
        .split("fn save_editor_workspace")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("workspace save implementation");

    assert!(document_source.contains("struct EditorWorkspaceDocumentRef<'workspace>"));
    assert!(document_source.contains("editor_workspace: &'workspace ProjectEditorWorkspace"));
    assert!(save.contains("encode_editor_workspace_document(workspace)"));
    assert!(!save.contains("workspace.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hr_editor_borrowed_workspace_save_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let workspace = benchmark_workspace(512, 4 * 1024);

    black_box(encode_editor_workspace_document(&workspace).unwrap());
    black_box(cloned_encode_editor_workspace(&workspace).unwrap());

    let mut cloned_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_cloned = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(cloned_encode_editor_workspace(black_box(&workspace)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(encode_editor_workspace_document(black_box(&workspace)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            cloned_samples.push(measure_cloned());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            cloned_samples.push(measure_cloned());
        }
    }

    let cloned_p95_ns = nearest_rank_percentile(&cloned_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR210_BORROWED_WORKSPACE_SAVE_BENCH_V1 cloned_p95_ns={cloned_p95_ns} optimized_p95_ns={optimized_p95_ns} cloned_samples_ns={} optimized_samples_ns={}",
        join_samples(&cloned_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= cloned_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "borrowed P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of cloned P95 {cloned_p95_ns}ns"
    );
}

fn cloned_encode_editor_workspace(
    workspace: &ProjectEditorWorkspace,
) -> Result<String, WriteError> {
    write_versioned_text(&EditorWorkspaceDocument {
        editor_workspace: workspace.clone(),
    })
}

fn benchmark_workspace(instance_count: usize, payload_bytes: usize) -> ProjectEditorWorkspace {
    let payload = "p".repeat(payload_bytes);
    ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: (0..instance_count)
            .map(|index| ViewInstance {
                instance_id: ViewInstanceId::new(format!("benchmark-view-{index}")),
                descriptor_id: ViewDescriptorId::new("benchmark.descriptor"),
                title: format!("Benchmark View {index}"),
                serializable_payload: json!({"index": index, "payload": payload.as_str()}),
                dirty: index % 2 == 0,
                host: ViewHost::ExclusivePage(MainPageId::new(format!("benchmark-page-{index}"))),
            })
            .collect(),
        focused_view: (instance_count > 0)
            .then(|| ViewInstanceId::new(format!("benchmark-view-{}", instance_count - 1))),
        active_drawers: Vec::new(),
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
