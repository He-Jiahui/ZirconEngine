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

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeOperationPhase {
    Queued = 1,
    Preparing = 2,
    ReadyToApply = 3,
    Completed = 4,
    Failed = 5,
    Cancelled = 6,
    Expired = 7,
    Harvested = 8,
}

impl ZrRuntimeOperationPhase {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Queued),
            2 => Some(Self::Preparing),
            3 => Some(Self::ReadyToApply),
            4 => Some(Self::Completed),
            5 => Some(Self::Failed),
            6 => Some(Self::Cancelled),
            7 => Some(Self::Expired),
            8 => Some(Self::Harvested),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Expired | Self::Harvested
        )
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeOperationDetailKindV2 {
    None = 0,
    QueueDepth = 1,
    AdmissionCountLimit = 2,
    AdmissionByteLimit = 3,
    DeadlineElapsed = 4,
    Cancelled = 5,
    WorkerPanic = 6,
    OwnerApplyFailed = 7,
    TerminalTtlElapsed = 8,
    Harvested = 9,
    WorkerChannelLost = 10,
}

impl ZrRuntimeOperationDetailKindV2 {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::QueueDepth),
            2 => Some(Self::AdmissionCountLimit),
            3 => Some(Self::AdmissionByteLimit),
            4 => Some(Self::DeadlineElapsed),
            5 => Some(Self::Cancelled),
            6 => Some(Self::WorkerPanic),
            7 => Some(Self::OwnerApplyFailed),
            8 => Some(Self::TerminalTtlElapsed),
            9 => Some(Self::Harvested),
            10 => Some(Self::WorkerChannelLost),
            _ => None,
        }
    }
}

/// Fixed-layout, allocation-free operation status returned by the current poll ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeOperationStatusV2 {
    pub abi_version: u32,
    pub phase: u32,
    pub detail_kind: u32,
    pub reserved: u32,
    pub handle: ZrRuntimeOperationHandle,
    pub completed_work: u64,
    pub total_work: u64,
    pub detail_value: u64,
}

impl ZrRuntimeOperationStatusV2 {
    pub const fn new(
        handle: ZrRuntimeOperationHandle,
        phase: ZrRuntimeOperationPhase,
        completed_work: u64,
        total_work: u64,
        detail_kind: ZrRuntimeOperationDetailKindV2,
        detail_value: u64,
    ) -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_ABI_VERSION_V2,
            phase: phase.raw(),
            detail_kind: detail_kind.raw(),
            reserved: 0,
            handle,
            completed_work,
            total_work,
            detail_value,
        }
    }

    pub const fn phase(&self) -> Option<ZrRuntimeOperationPhase> {
        ZrRuntimeOperationPhase::from_raw(self.phase)
    }

    pub const fn detail_kind(&self) -> Option<ZrRuntimeOperationDetailKindV2> {
        ZrRuntimeOperationDetailKindV2::from_raw(self.detail_kind)
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
pub type ZrRuntimePollOperationFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus;
pub type ZrRuntimeHarvestOperationFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;
