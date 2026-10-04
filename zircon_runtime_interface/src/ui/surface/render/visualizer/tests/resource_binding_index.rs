use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::surface::{UiRenderResourceKind, UiResourceUvRect};

const SAMPLE_COUNT: usize = 11;

fn image_resource(id: impl Into<String>) -> UiRenderResourceKey {
    UiRenderResourceKey::new(UiRenderResourceKind::Image, id)
}

fn add_linear(
    bindings: &mut Vec<UiRenderVisualizerResourceBinding>,
    resource: UiRenderResourceKey,
    paint_index: Option<usize>,
    batch_index: Option<usize>,
) {
    if let Some(binding) = bindings
        .iter_mut()
        .find(|binding| binding.resource == resource)
    {
        if let Some(paint_index) = paint_index {
            if !binding.paint_indices.contains(&paint_index) {
                binding.paint_indices.push(paint_index);
            }
        }
        if let Some(batch_index) = batch_index {
            if !binding.batch_indices.contains(&batch_index) {
                binding.batch_indices.push(batch_index);
            }
        }
        return;
    }
    bindings.push(UiRenderVisualizerResourceBinding {
        resource,
        paint_indices: paint_index.into_iter().collect(),
        batch_indices: batch_index.into_iter().collect(),
    });
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[SAMPLE_COUNT - 1]
}

#[test]
fn runtime_interface03_batch5_exact_resources_merge_while_same_id_variants_remain_separate() {
    let base = image_resource("atlas").with_revision(1);
    let revised = image_resource("atlas").with_revision(2);
    let mut index = ResourceBindingIndex::default();

    index.add(&base, Some(4), Some(2));
    index.add(&base, Some(4), Some(2));
    index.add(&base, Some(8), Some(1));
    index.add(&base, Some(6), Some(2));
    index.add(&revised, Some(9), Some(3));

    let bindings = index.into_bindings();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].resource, base);
    assert_eq!(bindings[0].paint_indices, vec![4, 8, 6]);
    assert_eq!(bindings[0].batch_indices, vec![2, 1]);
    assert_eq!(bindings[1].resource, revised);
}

#[test]
fn runtime_interface03_batch5_nan_resource_keys_preserve_non_reflexive_binding_semantics() {
    let resource =
        image_resource("nan-atlas").with_uv_rect(UiResourceUvRect::new(f32::NAN, 0.0, 1.0, 1.0));
    let mut index = ResourceBindingIndex::default();

    index.add(&resource, Some(1), None);
    index.add(&resource, Some(2), None);

    assert_eq!(index.into_bindings().len(), 2);
}

#[test]
#[ignore = "release-only visualizer resource binding index benchmark"]
fn runtime_interface03_batch5_visualizer_resource_binding_index_release_benchmark() {
    const RESOURCE_COUNT: usize = 4_096;
    let resources = (0..RESOURCE_COUNT)
        .map(|index| image_resource(format!("resource-{index}")))
        .collect::<Vec<_>>();
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            let mut bindings = Vec::new();
            for (paint_index, resource) in resources.iter().enumerate() {
                add_linear(&mut bindings, resource.clone(), Some(paint_index), None);
            }
            black_box(bindings);
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            let mut bindings = ResourceBindingIndex::default();
            for (paint_index, resource) in resources.iter().enumerate() {
                bindings.add(resource, Some(paint_index), None);
            }
            black_box(bindings.into_bindings());
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            linear_samples.push(measure_linear());
        }
    }

    let linear_p95 = p95(&mut linear_samples);
    let indexed_p95 = p95(&mut indexed_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_VISUALIZER_RESOURCE_BINDING_INDEX_BENCH_V1 resources={RESOURCE_COUNT} samples={SAMPLE_COUNT} linear_p50_ns={} indexed_p50_ns={} linear_p95_ns={linear_p95} indexed_p95_ns={indexed_p95}",
        linear_samples[SAMPLE_COUNT / 2],
        indexed_samples[SAMPLE_COUNT / 2],
    );
    assert!(
        indexed_p95.saturating_mul(5) <= linear_p95.saturating_mul(4),
        "indexed binding admission must improve P95 by at least 20%: linear={linear_p95}ns indexed={indexed_p95}ns",
    );
}

#[test]
#[ignore = "release-only borrowed visualizer resource admission benchmark"]
fn runtime_interface03_batch5_visualizer_borrowed_resource_admission_release_benchmark() {
    const ASSOCIATION_COUNT: usize = 4_096;
    let resource = image_resource("shared-atlas");
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            let mut bindings = Vec::new();
            for paint_index in 0..ASSOCIATION_COUNT {
                add_linear(
                    &mut bindings,
                    resource.clone(),
                    Some(paint_index),
                    Some(paint_index / 8),
                );
            }
            black_box(bindings);
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            let mut bindings = ResourceBindingIndex::default();
            for paint_index in 0..ASSOCIATION_COUNT {
                bindings.add(&resource, Some(paint_index), Some(paint_index / 8));
            }
            black_box(bindings.into_bindings());
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            linear_samples.push(measure_linear());
        }
    }

    let linear_p95 = p95(&mut linear_samples);
    let borrowed_p95 = p95(&mut borrowed_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_VISUALIZER_BORROWED_RESOURCE_ADMISSION_BENCH_V1 associations={ASSOCIATION_COUNT} samples={SAMPLE_COUNT} linear_p50_ns={} borrowed_p50_ns={} linear_p95_ns={linear_p95} borrowed_p95_ns={borrowed_p95}",
        linear_samples[SAMPLE_COUNT / 2],
        borrowed_samples[SAMPLE_COUNT / 2],
    );
    assert!(
        borrowed_p95.saturating_mul(5) <= linear_p95.saturating_mul(4),
        "borrowed binding admission must improve P95 by at least 20%: linear={linear_p95}ns borrowed={borrowed_p95}ns",
    );
}
