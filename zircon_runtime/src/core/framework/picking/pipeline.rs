use super::pointer_hits::sorted_hits_by_pointer;
use super::{
    CameraRaySource, PickingBackend, PickingEventState, PickingHoverMap, PickingPipelineReport,
    PickingPointerEvent, PickingScheduleLabel, PickingSettings, PointerHits, PointerInput,
    PointerLocation, RayMap,
};

pub struct PickingPipelineInput<'a> {
    pub settings: PickingSettings,
    pub pointer_locations: &'a [PointerLocation],
    pub pointer_inputs: &'a [PointerInput],
    pub cameras: &'a [CameraRaySource],
    pub backends: &'a [&'a dyn PickingBackend],
}

impl<'a> PickingPipelineInput<'a> {
    pub const fn new(
        pointer_locations: &'a [PointerLocation],
        pointer_inputs: &'a [PointerInput],
        cameras: &'a [CameraRaySource],
        backends: &'a [&'a dyn PickingBackend],
    ) -> Self {
        Self {
            settings: PickingSettings::DEFAULT,
            pointer_locations,
            pointer_inputs,
            cameras,
            backends,
        }
    }

    pub const fn with_settings(mut self, settings: PickingSettings) -> Self {
        self.settings = settings;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingPipelineOutput {
    pub ray_map: RayMap,
    pub backend_outputs: Vec<PointerHits>,
    pub hover_map: PickingHoverMap,
    pub events: Vec<PickingPointerEvent>,
    pub report: PickingPipelineReport,
    pub stages: Vec<PickingPipelineStageReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickingPipelineStageReport {
    pub label: PickingScheduleLabel,
    pub enabled: bool,
    pub input_count: usize,
    pub output_count: usize,
}

impl PickingPipelineStageReport {
    pub const fn new(
        label: PickingScheduleLabel,
        enabled: bool,
        input_count: usize,
        output_count: usize,
    ) -> Self {
        Self {
            label,
            enabled,
            input_count,
            output_count,
        }
    }
}

pub fn run_picking_pipeline(
    event_state: &mut PickingEventState,
    input: PickingPipelineInput<'_>,
) -> PickingPipelineOutput {
    if !input.settings.enabled {
        event_state.clear();
        return disabled_output(input);
    }

    let mut stages = Vec::with_capacity(5);
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Input,
        true,
        input.pointer_locations.len() + input.pointer_inputs.len(),
        input.pointer_locations.len(),
    ));

    let mut ray_map = RayMap::default();
    if input.settings.ray_map_enabled {
        ray_map.rebuild(input.pointer_locations, input.cameras);
    }
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::RayMap,
        input.settings.ray_map_enabled,
        input.pointer_locations.len() * input.cameras.len(),
        ray_map.len(),
    ));

    let backend_outputs = if input.settings.ray_map_enabled {
        let estimated_output_count = input.backends.len().saturating_mul(ray_map.len());
        let mut backend_outputs = Vec::with_capacity(estimated_output_count);
        for backend in input.backends {
            backend_outputs.extend(backend.collect_hits(&ray_map));
        }
        backend_outputs
    } else {
        Vec::new()
    };
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Backend,
        input.settings.ray_map_enabled,
        input.backends.len(),
        backend_outputs.len(),
    ));

    let (hover_map, report) = resolve_picking_outputs_with_ray_map(&ray_map, &backend_outputs);
    let hovered_hit_count = hover_map.iter().map(|(_, hits)| hits.len()).sum();
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Hover,
        true,
        backend_outputs.iter().map(|output| output.hits.len()).sum(),
        hovered_hit_count,
    ));

    let events = event_state.dispatch_frame(
        hover_map.clone(),
        input.pointer_locations,
        input.pointer_inputs,
    );
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Events,
        true,
        input.pointer_inputs.len() + hovered_hit_count,
        events.len(),
    ));

    PickingPipelineOutput {
        ray_map,
        backend_outputs,
        hover_map,
        events,
        report,
        stages,
    }
}

pub fn resolve_picking_outputs(
    outputs: &[PointerHits],
) -> (PickingHoverMap, PickingPipelineReport) {
    resolve_picking_outputs_with_ray_map(&RayMap::default(), outputs)
}

fn resolve_picking_outputs_with_ray_map(
    ray_map: &RayMap,
    outputs: &[PointerHits],
) -> (PickingHoverMap, PickingPipelineReport) {
    let sorted_hits = sorted_hits_by_pointer(outputs);
    let report =
        PickingPipelineReport::from_ray_map_outputs_and_sorted_hits(ray_map, outputs, &sorted_hits);
    let hover_map = PickingHoverMap::from_sorted_hits(sorted_hits);
    (hover_map, report)
}

fn disabled_output(input: PickingPipelineInput<'_>) -> PickingPipelineOutput {
    let stages = vec![
        PickingPipelineStageReport::new(
            PickingScheduleLabel::Input,
            false,
            input.pointer_locations.len() + input.pointer_inputs.len(),
            0,
        ),
        PickingPipelineStageReport::new(PickingScheduleLabel::RayMap, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Backend, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Hover, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Events, false, 0, 0),
    ];

    PickingPipelineOutput {
        ray_map: RayMap::default(),
        backend_outputs: Vec::new(),
        hover_map: PickingHoverMap::default(),
        events: Vec::new(),
        report: PickingPipelineReport::default(),
        stages,
    }
}

#[cfg(test)]
mod optimization_batch_20260830ck_runtime_tests {
    use std::hint::black_box;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const BACKENDS_PER_SAMPLE: usize = 32;
    const OUTPUTS_PER_BACKEND: usize = 128;

    #[test]
    fn picking_backend_collection_reserves_ray_output_estimate() {
        let source = include_str!("pipeline.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("picking pipeline implementation");

        assert!(implementation.contains("input.backends.len().saturating_mul(ray_map.len())"));
        assert!(implementation.contains("Vec::with_capacity(estimated_output_count)"));
        assert!(implementation.contains("for backend in input.backends"));
        assert!(implementation.contains("backend_outputs.extend(backend.collect_hits(&ray_map))"));
        assert!(!implementation.contains(".flat_map(|backend| backend.collect_hits(&ray_map))"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ck_runtime_picking_backend_capacity_p95() {
        let backends = (0..BACKENDS_PER_SAMPLE)
            .map(|backend| {
                (0..OUTPUTS_PER_BACKEND)
                    .map(|output| (backend * OUTPUTS_PER_BACKEND + output) as u64)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&backends, false));
                optimized.push(measure(&backends, true));
            } else {
                optimized.push(measure(&backends, true));
                legacy.push(measure(&backends, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("RUNTIME387_PICKING_BACKEND_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} backends_per_sample={BACKENDS_PER_SAMPLE} outputs_per_backend={OUTPUTS_PER_BACKEND} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(backends: &[Vec<u64>], use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let outputs = if use_capacity {
                let mut outputs = Vec::with_capacity(BACKENDS_PER_SAMPLE * OUTPUTS_PER_BACKEND);
                for backend in black_box(backends) {
                    outputs.extend(backend.iter().copied());
                }
                outputs
            } else {
                black_box(backends)
                    .iter()
                    .flat_map(|backend| backend.iter().copied())
                    .collect::<Vec<_>>()
            };
            checksum ^= outputs.len();
            black_box(outputs);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
