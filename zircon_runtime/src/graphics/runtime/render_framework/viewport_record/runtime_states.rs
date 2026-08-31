use crate::graphics::{HybridGiRuntimeState, VirtualGeometryRuntimeState};

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn ensure_hybrid_gi_runtime(
        &mut self,
        key: &ViewportCameraHistoryKey,
        provider: &dyn crate::graphics::HybridGiRuntimeProvider,
    ) -> &mut (dyn HybridGiRuntimeState + 'static) {
        if let Some(runtime) = self.hybrid_gi_runtimes.get_mut(key) {
            return runtime.as_mut();
        }
        self.hybrid_gi_runtimes
            .insert(key.clone(), provider.create_state());
        self.hybrid_gi_runtimes
            .get_mut(key)
            .expect("inserted Hybrid GI runtime")
            .as_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_hybrid_gi_runtimes(&mut self) {
        self.hybrid_gi_runtimes.clear();
    }

    pub(in crate::graphics::runtime::render_framework) fn hybrid_gi_runtime_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut (dyn HybridGiRuntimeState + 'static)> {
        self.hybrid_gi_runtimes.get_mut(key).map(Box::as_mut)
    }

    pub(in crate::graphics::runtime::render_framework) fn ensure_virtual_geometry_runtime(
        &mut self,
        key: &ViewportCameraHistoryKey,
        provider: &dyn crate::graphics::VirtualGeometryRuntimeProvider,
    ) -> &mut (dyn VirtualGeometryRuntimeState + 'static) {
        if let Some(runtime) = self.virtual_geometry_runtimes.get_mut(key) {
            return runtime.as_mut();
        }
        self.virtual_geometry_runtimes
            .insert(key.clone(), provider.create_state());
        self.virtual_geometry_runtimes
            .get_mut(key)
            .expect("inserted virtual geometry runtime")
            .as_mut()
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_virtual_geometry_runtimes(
        &mut self,
    ) {
        self.virtual_geometry_runtimes.clear();
    }

    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_runtime_mut(
        &mut self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&mut (dyn VirtualGeometryRuntimeState + 'static)> {
        self.virtual_geometry_runtimes.get_mut(key).map(Box::as_mut)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt;
    use std::hint::black_box;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::time::Instant;

    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderCameraTarget, RenderLayerSet, RenderViewportDescriptor,
        RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::UVec2;
    use crate::graphics::{
        HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
        HybridGiRuntimeProvider, HybridGiRuntimeState, HybridGiRuntimeUpdate,
        VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
        VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
        VirtualGeometryRuntimeState, VirtualGeometryRuntimeUpdate,
    };

    use super::super::camera_history_key::ViewportCameraHistoryKey;
    use super::ViewportRecord;

    const SAMPLE_PAIRS: usize = 17;
    const LOOKUPS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fk_runtime467_existing_runtime_lookup_borrows_camera_key() {
        let source = include_str!("runtime_states.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(!production.contains(".entry(key.clone())"));

        let provider = CountingHybridGiProvider::default();
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let key = wide_camera_key(47);
        record.ensure_hybrid_gi_runtime(&key, &provider);
        record.ensure_hybrid_gi_runtime(&key, &provider);
        assert_eq!(provider.created_count(), 1);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fk_runtime467_borrowed_runtime_key_lookup_benchmark() {
        let key = wide_camera_key(53);
        for _ in 0..4 {
            black_box(measure_lookup(legacy_existing_lookup, &key));
            black_box(measure_lookup(borrowed_existing_lookup, &key));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_lookup(legacy_existing_lookup, &key));
                optimized_samples.push(measure_lookup(borrowed_existing_lookup, &key));
            } else {
                optimized_samples.push(measure_lookup(borrowed_existing_lookup, &key));
                legacy_samples.push(measure_lookup(legacy_existing_lookup, &key));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_existing_lookup<'a>(
        runtimes: &'a mut HashMap<ViewportCameraHistoryKey, u64>,
        key: &ViewportCameraHistoryKey,
    ) -> &'a mut u64 {
        runtimes.entry(key.clone()).or_default()
    }

    fn borrowed_existing_lookup<'a>(
        runtimes: &'a mut HashMap<ViewportCameraHistoryKey, u64>,
        key: &ViewportCameraHistoryKey,
    ) -> &'a mut u64 {
        if let Some(runtime) = runtimes.get_mut(key) {
            return runtime;
        }
        runtimes.insert(key.clone(), 0);
        runtimes.get_mut(key).expect("inserted runtime")
    }

    fn measure_lookup(
        mut lookup: impl for<'a> FnMut(
            &'a mut HashMap<ViewportCameraHistoryKey, u64>,
            &ViewportCameraHistoryKey,
        ) -> &'a mut u64,
        key: &ViewportCameraHistoryKey,
    ) -> u128 {
        let mut runtimes = HashMap::new();
        runtimes.insert(key.clone(), 0_u64);
        let started = Instant::now();
        for _ in 0..LOOKUPS_PER_SAMPLE {
            let value = lookup(&mut runtimes, black_box(key));
            *value = black_box(*value).wrapping_add(1);
        }
        black_box(runtimes);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME467_BORROWED_RUNTIME_KEY_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn viewport_record_keeps_hybrid_gi_runtime_per_camera_key() {
        let provider = CountingHybridGiProvider::default();
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(1, UVec2::ZERO);
        let right_key = camera_key(1, UVec2::new(32, 0));

        record.ensure_hybrid_gi_runtime(&left_key, &provider);
        record.ensure_hybrid_gi_runtime(&left_key, &provider);
        record.ensure_hybrid_gi_runtime(&right_key, &provider);

        assert_eq!(provider.created_count(), 2);
        assert!(record.hybrid_gi_runtime_mut(&left_key).is_some());
        assert!(record.hybrid_gi_runtime_mut(&right_key).is_some());
    }

    #[test]
    fn viewport_record_keeps_virtual_geometry_runtime_per_camera_key() {
        let provider = CountingVirtualGeometryProvider::default();
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(2, UVec2::ZERO);
        let right_key = camera_key(2, UVec2::new(32, 0));

        record.ensure_virtual_geometry_runtime(&left_key, &provider);
        record.ensure_virtual_geometry_runtime(&left_key, &provider);
        record.ensure_virtual_geometry_runtime(&right_key, &provider);

        assert_eq!(provider.created_count(), 2);
        assert!(record.virtual_geometry_runtime_mut(&left_key).is_some());
        assert!(record.virtual_geometry_runtime_mut(&right_key).is_some());
    }

    fn camera_key(entity: u64, position: UVec2) -> ViewportCameraHistoryKey {
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        descriptor.target = RenderCameraTarget::PrimarySurface;
        descriptor.viewport_rect = Some(RenderViewportRect::new(position, UVec2::new(32, 64)));
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }

    fn wide_camera_key(entity: u64) -> ViewportCameraHistoryKey {
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        descriptor.culling_mask = RenderLayerSet::from_layers([0, 40, 80, 120, 160]);
        descriptor.volume_mask = RenderLayerSet::from_layers([1, 41, 81, 121, 161]);
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }

    #[derive(Clone, Default)]
    struct CountingHybridGiProvider {
        created: Arc<AtomicUsize>,
    }

    impl CountingHybridGiProvider {
        fn created_count(&self) -> usize {
            self.created.load(Ordering::SeqCst)
        }
    }

    impl fmt::Debug for CountingHybridGiProvider {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("CountingHybridGiProvider")
        }
    }

    impl HybridGiRuntimeProvider for CountingHybridGiProvider {
        fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
            self.created.fetch_add(1, Ordering::SeqCst);
            Box::<CountingHybridGiState>::default()
        }
    }

    #[derive(Default)]
    struct CountingHybridGiState;

    impl HybridGiRuntimeState for CountingHybridGiState {
        fn prepare_frame(
            &mut self,
            _input: HybridGiRuntimePrepareInput<'_>,
        ) -> HybridGiRuntimePrepareOutput {
            HybridGiRuntimePrepareOutput::default()
        }

        fn update_after_render(
            &mut self,
            _feedback: HybridGiRuntimeFeedback,
        ) -> HybridGiRuntimeUpdate {
            HybridGiRuntimeUpdate::default()
        }
    }

    #[derive(Clone, Default)]
    struct CountingVirtualGeometryProvider {
        created: Arc<AtomicUsize>,
    }

    impl CountingVirtualGeometryProvider {
        fn created_count(&self) -> usize {
            self.created.load(Ordering::SeqCst)
        }
    }

    impl fmt::Debug for CountingVirtualGeometryProvider {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("CountingVirtualGeometryProvider")
        }
    }

    impl VirtualGeometryRuntimeProvider for CountingVirtualGeometryProvider {
        fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
            self.created.fetch_add(1, Ordering::SeqCst);
            Box::<CountingVirtualGeometryState>::default()
        }
    }

    #[derive(Debug, Default)]
    struct CountingVirtualGeometryState;

    impl VirtualGeometryRuntimeState for CountingVirtualGeometryState {
        fn prepare_frame(
            &mut self,
            _input: VirtualGeometryRuntimePrepareInput<'_>,
        ) -> VirtualGeometryRuntimePrepareOutput {
            VirtualGeometryRuntimePrepareOutput::default()
        }

        fn update_after_render(
            &mut self,
            _feedback: VirtualGeometryRuntimeFeedback,
        ) -> VirtualGeometryRuntimeUpdate {
            VirtualGeometryRuntimeUpdate::default()
        }
    }
}
