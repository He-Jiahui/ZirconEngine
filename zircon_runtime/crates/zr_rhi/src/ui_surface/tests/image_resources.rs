use std::hint::black_box;
use std::time::Instant;

use super::compact_image_resources;
use crate::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceImagePayload, UiSurfaceImageResource,
    UiSurfaceImageResourceTable, UiSurfaceRect,
};

#[test]
fn clearing_image_resource_table_removes_all_resource_generations() {
    let mut resources = UiSurfaceImageResourceTable::default();
    for generation in [4, 5] {
        resources.insert(
            "atlas://editor/icons".to_owned(),
            UiSurfaceImageResource {
                generation,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: vec![generation as u8; 16].into(),
            },
        );
    }

    resources.clear();

    assert!(resources.is_empty());
    assert_eq!(resources.len(), 0);
}

#[test]
fn shared_image_commands_move_rgba_into_one_resource_entry() {
    let image = |z_index| UiSurfaceCommand {
        z_index,
        frame: UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                resource_generation: 23,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![z_index as u8; 16]),
                atlas_uv: None,
            },
        },
    };

    let (commands, resources) = compact_image_resources(vec![image(0), image(1)]);

    assert_eq!(resources.len(), 1);
    let resource = resources
        .get("atlas://editor/icons", 23)
        .expect("shared atlas generation is canonical");
    assert_eq!(resource.generation, 23);
    assert_eq!(resource.rgba.as_ref(), &[0; 16]);
    assert!(commands.iter().all(|command| matches!(
        &command.kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    )));
}

#[test]
fn distinct_image_generations_remain_separate_canonical_resource_payloads() {
    let command = |generation, rgba| UiSurfaceCommand {
        z_index: generation as i32,
        frame: UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                resource_generation: generation,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(rgba),
                atlas_uv: None,
            },
        },
    };

    let (commands, resources) =
        compact_image_resources(vec![command(4, vec![4; 16]), command(5, vec![5; 16])]);

    assert_eq!(resources.len(), 2);
    assert_eq!(
        resources
            .get("atlas://editor/icons", 4)
            .expect("older generation remains addressable")
            .rgba
            .as_ref(),
        &[4; 16]
    );
    assert_eq!(
        resources
            .get("atlas://editor/icons", 5)
            .expect("newer generation remains addressable")
            .rgba
            .as_ref(),
        &[5; 16]
    );
    assert!(commands.iter().all(|command| matches!(
        &command.kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    )));
}

fn image_resource(generation: u64, value: u8) -> UiSurfaceImageResource {
    UiSurfaceImageResource {
        generation,
        width: 1,
        height: 1,
        upload_bytes: 4,
        rgba: vec![value; 4].into(),
    }
}

#[test]
fn optimization_batch_hj_runtime586_image_resource_extend_moves_generation_groups() {
    let mut current = UiSurfaceImageResourceTable::default();
    current.insert("atlas://shared".to_owned(), image_resource(1, 1));
    current.insert("atlas://retained".to_owned(), image_resource(7, 7));

    let mut incoming = UiSurfaceImageResourceTable::default();
    incoming.insert("atlas://shared".to_owned(), image_resource(1, 9));
    incoming.insert("atlas://shared".to_owned(), image_resource(2, 2));

    current.extend(incoming);

    assert_eq!(current.len(), 3);
    assert_eq!(
        current
            .get("atlas://shared", 1)
            .expect("incoming duplicate generation wins")
            .rgba
            .as_ref(),
        &[9; 4]
    );
    assert_eq!(
        current
            .get("atlas://shared", 2)
            .expect("incoming generation is moved")
            .rgba
            .as_ref(),
        &[2; 4]
    );
    assert!(current.get("atlas://retained", 7).is_some());
}

#[test]
fn optimization_batch_hj_runtime586_image_resource_extend_uses_group_append() {
    let source = include_str!("../image_resources.rs");
    let production = source.split("mod tests {").next().unwrap();
    let extend = production
        .split("pub fn extend")
        .nth(1)
        .expect("image resource table extend")
        .split("pub(super) fn compact_image_resources")
        .next()
        .expect("extend ends before compaction");

    assert!(extend.contains("resources.by_resource_key"));
    assert!(extend.contains(".append(&mut generations)"));
    assert!(!extend.contains("resources.into_entries()"));
}

fn resource_table(
    key_count: usize,
    first_generation: u64,
    generation_count: usize,
) -> UiSurfaceImageResourceTable {
    let mut table = UiSurfaceImageResourceTable::default();
    for key_index in 0..key_count {
        let key = format!(
            "atlas://runtime586/{key_index:04}/{}",
            "long-resource-identity-segment".repeat(4)
        );
        for generation_offset in 0..generation_count {
            let generation = first_generation + generation_offset as u64;
            table.insert(key.clone(), image_resource(generation, generation as u8));
        }
    }
    table
}

fn legacy_extend(
    mut current: UiSurfaceImageResourceTable,
    resources: UiSurfaceImageResourceTable,
) -> UiSurfaceImageResourceTable {
    for (resource_key, resource) in resources.into_entries() {
        current.insert(resource_key, resource);
    }
    current
}

fn optimized_extend(
    mut current: UiSurfaceImageResourceTable,
    resources: UiSurfaceImageResourceTable,
) -> UiSurfaceImageResourceTable {
    current.extend(resources);
    current
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hj_runtime586_image_resource_group_extend_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const KEY_COUNT: usize = 256;
    const CURRENT_GENERATIONS: usize = 16;
    const INCOMING_GENERATIONS: usize = 64;

    let current = resource_table(KEY_COUNT, 0, CURRENT_GENERATIONS);
    let incoming = resource_table(KEY_COUNT, CURRENT_GENERATIONS as u64, INCOMING_GENERATIONS);

    let expected = legacy_extend(current.clone(), incoming.clone());
    let optimized = optimized_extend(current.clone(), incoming.clone());
    assert_eq!(optimized, expected);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_current = current.clone();
        let legacy_incoming = incoming.clone();
        let optimized_current = current.clone();
        let optimized_incoming = incoming.clone();
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_extend(legacy_current, legacy_incoming));
            legacy_samples.push(started.elapsed().as_nanos().max(1));

            let started = Instant::now();
            black_box(optimized_extend(optimized_current, optimized_incoming));
            optimized_samples.push(started.elapsed().as_nanos().max(1));
        } else {
            let started = Instant::now();
            black_box(optimized_extend(optimized_current, optimized_incoming));
            optimized_samples.push(started.elapsed().as_nanos().max(1));

            let started = Instant::now();
            black_box(legacy_extend(legacy_current, legacy_incoming));
            legacy_samples.push(started.elapsed().as_nanos().max(1));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME586_IMAGE_RESOURCE_GROUP_EXTEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             keys={KEY_COUNT} current_generations_per_key={CURRENT_GENERATIONS} \
             incoming_generations_per_key={INCOMING_GENERATIONS} \
             legacy_key_hashes_per_sample={} optimized_key_hashes_per_sample={KEY_COUNT} \
             legacy_key_clones_per_sample={} optimized_key_clones_per_sample=0 \
             legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}",
        KEY_COUNT * INCOMING_GENERATIONS,
        KEY_COUNT * INCOMING_GENERATIONS,
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 50,
        "group append P95 {optimized_p95}ns exceeded 50% of entry-wise P95 {legacy_p95}ns"
    );
}
