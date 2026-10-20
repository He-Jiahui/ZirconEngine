use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::RenderPluginRendererOutputs;
use zircon_runtime::graphics::{
    GraphicsError, RenderPassBufferUploadSink, RuntimePrepareCollector,
    RuntimePrepareCollectorContext, RuntimePrepareDeviceEpoch,
};

use crate::virtual_geometry::types::VirtualGeometryPrepareFrame;

use super::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackFuture, VirtualGeometryGpuResources,
};

#[derive(Default)]
struct State {
    prepare: Option<VirtualGeometryPrepareFrame>,
    resources: Option<(RuntimePrepareDeviceEpoch, VirtualGeometryGpuResources)>,
    readback: Option<VirtualGeometryGpuReadbackFuture>,
    completed: Option<VirtualGeometryGpuReadback>,
}

/// Test-only bridge for the legacy uploader contract. Inputs stay plugin-owned and all results
/// come from the uploader's submitted GPU buffers through the runtime readback queue.
#[derive(Default)]
pub(crate) struct VirtualGeometryUploaderTestCollector {
    state: Mutex<State>,
}

impl VirtualGeometryUploaderTestCollector {
    pub(crate) fn set_prepare(&self, prepare: VirtualGeometryPrepareFrame) {
        self.set_prepare_option(Some(prepare));
    }

    pub(crate) fn set_prepare_option(&self, prepare: Option<VirtualGeometryPrepareFrame>) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .prepare = prepare;
    }

    /// Completes both uploader readbacks once and retains their independent completed DTO.
    pub(crate) fn completed_readback(
        &self,
    ) -> Result<Option<VirtualGeometryGpuReadback>, GraphicsError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(readback) = &state.completed {
            return Ok(Some(readback.clone()));
        }
        if state.readback.is_none() {
            return Ok(None);
        }
        if !state
            .readback
            .as_ref()
            .is_some_and(VirtualGeometryGpuReadbackFuture::is_ready)
        {
            return Err(GraphicsError::BufferMap(
                "uploader page-table and completion readbacks are not ready".into(),
            ));
        }
        let readback = state
            .readback
            .take()
            .expect("verified actual uploader readback")
            .try_collect()
            .ok_or_else(|| {
                GraphicsError::BufferMap("uploader readback readiness changed".into())
            })??;
        state.completed = Some(readback.clone());
        Ok(Some(readback))
    }

    pub(crate) fn take_readback(&self) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        self.completed_readback()?.ok_or_else(|| {
            GraphicsError::BufferMap("no uploader prepare submission completed".into())
        })
    }

    /// Takes the completed GPU readback once, preserving the exact plugin-owned bytes for
    /// test-only reader facades. Callers must consume it after the framework submission has
    /// finished; no synthetic/default payload is produced.
    pub(crate) fn take_gpu_readback(&self) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        self.take_readback()
    }
}

impl RuntimePrepareCollector for VirtualGeometryUploaderTestCollector {
    fn requests_gpu_readback(&self) -> bool {
        true
    }

    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.completed = None;
        let Some(prepare) = state.prepare.take() else {
            state.readback = None;
            return Ok(RenderPluginRendererOutputs::default());
        };
        if !context.gpu_work_admitted() {
            return Err(GraphicsError::BufferMap(
                "uploader test frame was not admitted for GPU work".to_owned(),
            ));
        }

        let epoch = context.device_epoch();
        let page_budget = context
            .frame_extract()
            .geometry
            .virtual_geometry
            .as_ref()
            .map(|extract| extract.page_budget);
        let mut recording = context.gpu_recording_context();
        let resources = match state.resources.take() {
            Some((resource_epoch, resources)) if resource_epoch == epoch => resources,
            _ => VirtualGeometryGpuResources::new(recording.device),
        };
        let pending = resources.execute_prepare(
            recording.device,
            &mut recording.buffer_uploads,
            recording.encoder,
            Some(&prepare),
            page_budget,
        )?;
        drop(recording);

        state.resources = Some((epoch, resources));
        state.readback = pending
            .map(|pending| pending.enqueue(context))
            .transpose()?;
        Ok(RenderPluginRendererOutputs::default())
    }
}

pub(crate) fn uploader_test_collector_registration(
    collector: Arc<VirtualGeometryUploaderTestCollector>,
) -> zircon_runtime::graphics::RuntimePrepareCollectorRegistration {
    zircon_runtime::graphics::RuntimePrepareCollectorRegistration::new_collector(
        "virtual-geometry.uploader-test-readback",
        collector,
    )
}
