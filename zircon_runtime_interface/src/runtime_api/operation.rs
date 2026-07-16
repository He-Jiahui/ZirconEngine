use serde::{Deserialize, Serialize};

use crate::{ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeSessionHandle, ZrStatus};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZrRuntimeOperationHandle(u64);

impl ZrRuntimeOperationHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeOperationSubmitRequestV1 {
    pub abi_version: u32,
    pub operation_id: String,
    pub payload: serde_json::Value,
}

impl ZrRuntimeOperationSubmitRequestV1 {
    pub fn new(
        abi_version: u32,
        operation_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            abi_version,
            operation_id: operation_id.into(),
            payload,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrRuntimeOperationPhase {
    Queued,
    Running,
    Completed,
    Failed,
}

impl ZrRuntimeOperationPhase {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrRuntimeOperationProgressV1 {
    pub abi_version: u32,
    pub handle: ZrRuntimeOperationHandle,
    pub phase: ZrRuntimeOperationPhase,
    pub completed_work: u64,
    pub total_work: u64,
    pub message: String,
}

impl ZrRuntimeOperationProgressV1 {
    pub fn new(
        abi_version: u32,
        handle: ZrRuntimeOperationHandle,
        phase: ZrRuntimeOperationPhase,
        completed_work: u64,
        total_work: u64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            abi_version,
            handle,
            phase,
            completed_work,
            total_work,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ZrRuntimeOperationOutcomeV1 {
    Succeeded { output: serde_json::Value },
    Failed { error: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeOperationResultV1 {
    pub abi_version: u32,
    pub handle: ZrRuntimeOperationHandle,
    pub operation_id: String,
    pub outcome: ZrRuntimeOperationOutcomeV1,
}

impl ZrRuntimeOperationResultV1 {
    pub fn succeeded(
        abi_version: u32,
        handle: ZrRuntimeOperationHandle,
        operation_id: impl Into<String>,
        output: serde_json::Value,
    ) -> Self {
        Self {
            abi_version,
            handle,
            operation_id: operation_id.into(),
            outcome: ZrRuntimeOperationOutcomeV1::Succeeded { output },
        }
    }

    pub fn failed(
        abi_version: u32,
        handle: ZrRuntimeOperationHandle,
        operation_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            abi_version,
            handle,
            operation_id: operation_id.into(),
            outcome: ZrRuntimeOperationOutcomeV1::Failed {
                error: error.into(),
            },
        }
    }

    pub fn succeeded_output(&self) -> Option<&serde_json::Value> {
        match &self.outcome {
            ZrRuntimeOperationOutcomeV1::Succeeded { output } => Some(output),
            ZrRuntimeOperationOutcomeV1::Failed { .. } => None,
        }
    }

    pub fn failure(&self) -> Option<&str> {
        match &self.outcome {
            ZrRuntimeOperationOutcomeV1::Succeeded { .. } => None,
            ZrRuntimeOperationOutcomeV1::Failed { error } => Some(error),
        }
    }
}

pub type ZrRuntimeSubmitOperationFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrByteSlice,
    *mut ZrRuntimeOperationHandle,
) -> ZrStatus;
pub type ZrRuntimePollOperationFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;
pub type ZrRuntimeHarvestOperationFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;
