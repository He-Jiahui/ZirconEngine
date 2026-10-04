use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_message::DocumentId;
use crate::ui::workbench::view::ViewInstanceId;

use super::{dirty_details, DirtyCloseView};

const RENDERS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor888_close_prompt_details_direct_append_preserves_exact_text() {
    let views = [
        dirty_view(1, ""),
        dirty_view(2, "Material"),
        dirty_view(3, "Scene"),
        dirty_view(4, "Script"),
    ];

    assert_eq!(dirty_details(&[], false), "");
    assert_eq!(dirty_details(&[], true), "Active Scene");
    assert_eq!(dirty_details(&views[..2], false), ", Material");
    assert_eq!(dirty_details(&views[..3], false), ", Material, Scene");
    assert_eq!(dirty_details(&views, false), ", Material, Scene, ...");
    assert_eq!(dirty_details(&views[..2], true), "Active Scene, , Material");
    assert_eq!(
        dirty_details(&views[..3], true),
        "Active Scene, , Material, ..."
    );

    for includes_project_scene in [false, true] {
        for count in 0..=views.len() {
            assert_eq!(
                dirty_details(&views[..count], includes_project_scene),
                legacy_dirty_details(&views[..count], includes_project_scene)
            );
        }
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor888_close_prompt_details_direct_append_benchmark() {
    let views = [
        dirty_view(1, "Main Scene"),
        dirty_view(2, "Player Material"),
        dirty_view(3, "World Settings"),
        dirty_view(4, "Game Script"),
    ];
    assert_eq!(
        dirty_details(&views, false),
        legacy_dirty_details(&views, false)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&views, legacy_dirty_details));
            optimized.push(measure(&views, dirty_details));
        } else {
            optimized.push(measure(&views, dirty_details));
            legacy.push(measure(&views, legacy_dirty_details));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR888_CLOSE_PROMPT_DETAILS_DIRECT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} renders_per_sample={RENDERS_PER_SAMPLE} visible_names_per_render=3 legacy_temporary_vector_slots_per_sample=12288 optimized_temporary_vector_slots_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "direct close-detail append P95 {optimized_p95}ns must stay within 10% of collect/join P95 {legacy_p95}ns"
    );
}

fn dirty_view(document_id: u64, title: &str) -> DirtyCloseView {
    DirtyCloseView {
        document_id: DocumentId::new(document_id),
        dirty_generation: document_id,
        close_revision: crate::core::editor_event::DocumentCloseRevision {
            external_generation: document_id,
            ..Default::default()
        },
        instance_id: ViewInstanceId::new(format!("editor.asset#{document_id}")),
        title: title.to_string(),
    }
}

fn legacy_dirty_details(views: &[DirtyCloseView], includes_project_scene: bool) -> String {
    let mut names = includes_project_scene
        .then_some("Active Scene")
        .into_iter()
        .chain(views.iter().map(|view| view.title.as_str()))
        .take(3)
        .collect::<Vec<_>>()
        .join(", ");
    if views.len() + usize::from(includes_project_scene) > 3 {
        names.push_str(", ...");
    }
    names
}

fn measure(views: &[DirtyCloseView], render: fn(&[DirtyCloseView], bool) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RENDERS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(views), false)).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
