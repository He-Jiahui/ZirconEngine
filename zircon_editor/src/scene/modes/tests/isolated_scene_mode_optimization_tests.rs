use std::hint::black_box;
use std::time::Instant;

use crate::core::plugin::run_editor_plugin_boundary;

#[test]
fn optimization_batch_dh_scene_mode_boundary_borrows_owner_source() {
    let source = include_str!("../isolated_scene_mode.rs");

    assert!(!source.contains("let owner_id = self.owner_id.clone();"));
    assert!(source.contains("let owner_id = self.owner_id.as_str();"));
    assert!(source.contains("run_editor_plugin_boundary(self.owner_id.as_str()"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dh_scene_mode_borrowed_owner_boundary_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const BOUNDARIES_PER_SAMPLE: usize = 65_536;

    let owner_id = format!("plugin.editor.scene-mode.{}", "boundary-owner-".repeat(8));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_owner_boundary(
                &owner_id,
                BOUNDARIES_PER_SAMPLE,
                true,
            ));
            optimized_samples.push(measure_owner_boundary(
                &owner_id,
                BOUNDARIES_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples.push(measure_owner_boundary(
                &owner_id,
                BOUNDARIES_PER_SAMPLE,
                false,
            ));
            legacy_samples.push(measure_owner_boundary(
                &owner_id,
                BOUNDARIES_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "EDITOR344_SCENE_MODE_BORROWED_OWNER_BOUNDARY_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "borrowed scene-mode owner p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_owner_boundary(owner_id: &String, boundaries: usize, clone_owner: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..boundaries {
        let value = if clone_owner {
            let cloned_owner = black_box(owner_id).clone();
            run_editor_plugin_boundary(&cloned_owner, "scene mode update", || {
                Ok::<_, String>(7_u64)
            })
        } else {
            run_editor_plugin_boundary(black_box(owner_id.as_str()), "scene mode update", || {
                Ok::<_, String>(7_u64)
            })
        }
        .expect("successful boundary");
        checksum = checksum.wrapping_add(value);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
