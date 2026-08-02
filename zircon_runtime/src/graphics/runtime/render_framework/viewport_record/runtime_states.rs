use crate::graphics::{HybridGiRuntimeState, VirtualGeometryRuntimeState};

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn ensure_hybrid_gi_runtime(
        &mut self,
        key: &ViewportCameraHistoryKey,
        provider: &dyn crate::graphics::HybridGiRuntimeProvider,
    ) -> &mut (dyn HybridGiRuntimeState + 'static) {
        self.hybrid_gi_runtimes
            .entry(key.clone())
            .or_insert_with(|| provider.create_state())
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
        self.virtual_geometry_runtimes
            .entry(key.clone())
            .or_insert_with(|| provider.create_state())
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
    use std::fmt;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderCameraTarget, RenderViewportDescriptor, RenderViewportRect,
        ViewportCameraSnapshot,
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
