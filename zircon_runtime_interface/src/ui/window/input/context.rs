use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{
    UiDeviceId, UiInputEventMetadata, UiInputModifiers, UiPointerId, UiPointerSource, UiSurfaceId,
    UiUserId,
};

use super::super::{UiWindowEventMetadata, UiWindowMetrics};

/// Platform event adapters use this context to attach stable window/user/device
/// identity before handing normalized events to the shared UI dispatcher.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowInputContext {
    pub metadata: UiInputEventMetadata,
    /// Most recently applied surface metrics, when the platform event source
    /// needs to preserve DPI while translating a physical-size-only resize.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_metrics: Option<UiWindowMetrics>,
}

impl PartialEq for UiWindowInputContext {
    fn eq(&self, other: &Self) -> bool {
        self.metadata == other.metadata
            && window_metrics_equal(self.window_metrics, other.window_metrics)
    }
}

impl Eq for UiWindowInputContext {}

impl UiWindowInputContext {
    pub fn from_window_metadata(metadata: &UiWindowEventMetadata) -> Self {
        let mut input = UiInputEventMetadata::new(metadata.timestamp, metadata.sequence);
        input.window_id = Some(metadata.window_id.clone());
        input.synthetic = metadata.synthetic;
        Self {
            metadata: input,
            window_metrics: None,
        }
    }

    pub fn with_user_id(mut self, user_id: UiUserId) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    pub fn with_device_id(mut self, device_id: UiDeviceId) -> Self {
        self.metadata.device_id = Some(device_id);
        self
    }

    pub fn with_surface_id(mut self, surface_id: UiSurfaceId) -> Self {
        self.metadata.surface_id = Some(surface_id);
        self
    }

    pub fn with_pointer_id(mut self, pointer_id: UiPointerId) -> Self {
        self.metadata.pointer_id = Some(pointer_id);
        self
    }

    pub fn with_pointer_source(mut self, pointer_source: UiPointerSource) -> Self {
        self.metadata.pointer_source = pointer_source;
        self
    }

    pub fn with_modifiers(mut self, modifiers: UiInputModifiers) -> Self {
        self.metadata.modifiers = modifiers;
        self
    }

    pub fn with_window_metrics(mut self, metrics: UiWindowMetrics) -> Self {
        self.window_metrics = Some(metrics);
        self
    }
}

fn window_metrics_equal(left: Option<UiWindowMetrics>, right: Option<UiWindowMetrics>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.logical_size.width.to_bits() == right.logical_size.width.to_bits()
                && left.logical_size.height.to_bits() == right.logical_size.height.to_bits()
                && left.physical_size == right.physical_size
                && left.scale_factor.to_bits() == right.scale_factor.to_bits()
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}
