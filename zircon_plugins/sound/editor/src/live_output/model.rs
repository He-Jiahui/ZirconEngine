use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::sound::{
    SoundBackendState, SoundBackendStatus, SoundOutputDeviceDescriptor, SoundOutputDeviceInfo,
    SoundOutputDeviceState, SoundOutputDeviceStatus, SoundOutputLatencyStatus,
};

/// Picker row for one output device descriptor exposed to the editor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEditorOutputDeviceRow {
    pub descriptor: SoundOutputDeviceDescriptor,
    pub selected: bool,
    pub is_default: bool,
    pub available: bool,
    pub diagnostic: Option<String>,
}

impl SoundEditorOutputDeviceRow {
    pub(crate) fn from_info(
        info: SoundOutputDeviceInfo,
        selected: &SoundOutputDeviceDescriptor,
    ) -> Self {
        let selected = same_device_descriptor(&info.descriptor, selected);
        Self {
            descriptor: info.descriptor,
            selected,
            is_default: info.is_default,
            available: info.available,
            diagnostic: info.diagnostic,
        }
    }
}

/// Editor-facing projection of the active output device and backend status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEditorOutputStatusModel {
    pub descriptor: SoundOutputDeviceDescriptor,
    pub state: SoundOutputDeviceState,
    pub backend_state: SoundBackendState,
    pub latency: SoundOutputLatencyStatus,
    pub rendered_blocks: u64,
    pub rendered_frames: u64,
    pub callback_count: u64,
    pub last_callback_sequence: Option<u64>,
    pub underrun_count: u64,
    pub last_error: Option<String>,
    pub diagnostics: Vec<String>,
}

impl SoundEditorOutputStatusModel {
    pub(crate) fn from_status(
        status: SoundOutputDeviceStatus,
        backend: &SoundBackendStatus,
    ) -> Self {
        Self {
            descriptor: status.descriptor,
            state: status.state,
            backend_state: backend.state.clone(),
            latency: status.latency,
            rendered_blocks: status.rendered_blocks,
            rendered_frames: status.rendered_frames,
            callback_count: status.callback_count,
            last_callback_sequence: status.last_callback_sequence,
            underrun_count: status.underrun_count,
            last_error: status.last_error,
            diagnostics: status.diagnostics,
        }
    }
}

/// Complete output snapshot consumed by live mixer toolbar bindings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEditorOutputSnapshot {
    pub devices: Vec<SoundEditorOutputDeviceRow>,
    pub status: SoundEditorOutputStatusModel,
    pub backend: SoundBackendStatus,
    pub diagnostics: Vec<String>,
}

/// User or host action that can refresh or change the live output device state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoundEditorOutputAction {
    Refresh,
    Configure(SoundOutputDeviceDescriptor),
    Start,
    Stop,
}

/// Result of applying a live output action, including best-effort failure state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEditorOutputActionReport {
    pub action: SoundEditorOutputAction,
    pub success: bool,
    pub error: Option<String>,
    pub snapshot: Option<SoundEditorOutputSnapshot>,
}

impl SoundEditorOutputActionReport {
    pub(crate) fn success(
        action: SoundEditorOutputAction,
        snapshot: SoundEditorOutputSnapshot,
    ) -> Self {
        Self {
            action,
            success: true,
            error: None,
            snapshot: Some(snapshot),
        }
    }

    pub(crate) fn failure(
        action: SoundEditorOutputAction,
        error: impl Into<String>,
        snapshot: Option<SoundEditorOutputSnapshot>,
    ) -> Self {
        Self {
            action,
            success: false,
            error: Some(error.into()),
            snapshot,
        }
    }
}

pub(crate) fn same_device_descriptor(
    left: &SoundOutputDeviceDescriptor,
    right: &SoundOutputDeviceDescriptor,
) -> bool {
    left.id == right.id && left.backend == right.backend
}
