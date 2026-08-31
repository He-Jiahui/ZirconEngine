use std::ops::Range;
use std::sync::{Arc, Mutex};

use crate::graphics::backend::RenderBackend;
use crate::graphics::{GraphicsError, RuntimePrepareDeviceEpoch};

/// One runtime-prepare readback request, qualified by the source collector's device epoch.
pub(crate) struct RuntimePrepareGpuReadbackRequest {
    name: String,
    buffer: wgpu::Buffer,
    range: Range<u64>,
    device_epoch: RuntimePrepareDeviceEpoch,
    completion: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
}

impl RuntimePrepareGpuReadbackRequest {
    pub(super) fn new(
        name: String,
        buffer: &wgpu::Buffer,
        range: Range<u64>,
        device_epoch: RuntimePrepareDeviceEpoch,
        completion: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
    ) -> Self {
        Self {
            name,
            buffer: buffer.clone(),
            range,
            device_epoch,
            completion,
        }
    }

    pub(crate) fn register(self, backend: &RenderBackend) -> Result<bool, GraphicsError> {
        let Self {
            name,
            buffer,
            range,
            device_epoch,
            completion,
        } = self;
        let actual_profile = backend.device_profile();
        let actual_device_id = actual_profile.device_id();
        let actual_generation = actual_profile.generation();
        if device_epoch.device_id() != actual_device_id
            || device_epoch.generation() != actual_generation
        {
            let error = GraphicsError::RuntimePrepareDeviceEpochMismatch {
                expected_device_id: device_epoch.device_id(),
                expected_generation: device_epoch.generation(),
                actual_device_id,
                actual_generation,
            };
            *completion
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Err(error.to_string()));
            return Err(error);
        }
        let byte_len = range.end.checked_sub(range.start).ok_or_else(|| {
            GraphicsError::BufferMap(format!(
                "runtime prepare GPU readback `{name}` has an inverted source range"
            ))
        })?;
        let callback_completion = Arc::clone(&completion);
        let result = backend.enqueue_product_diagnostic_buffer(
            &buffer,
            range.start,
            byte_len,
            Box::new(move |result| {
                *callback_completion
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
            }),
        );
        match result {
            Ok(true) => Ok(true),
            Ok(false) => {
                *completion
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Err(format!(
                    "runtime prepare GPU readback `{name}` was rejected by the product diagnostic budget"
                )));
                Ok(false)
            }
            Err(error) => {
                *completion
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    Some(Err(error.to_string()));
                Err(error)
            }
        }
    }

    pub(crate) fn fail(self, error: impl Into<String>) {
        *self
            .completion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Err(error.into()));
    }
}
