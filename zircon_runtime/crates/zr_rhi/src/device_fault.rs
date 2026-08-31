use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::{DeviceGeneration, DeviceId};

const HEALTHY: u8 = 0;
const RECORDING: u8 = 1;
const FAULTED: u8 = 2;

/// Normalized first-fault classification independent of a concrete graphics backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceFaultKind {
    OutOfMemory,
    Validation,
    Internal,
    DeviceLostUnknown,
    DeviceDestroyed,
}

/// Immutable first-fault record tied to one device generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceFaultRecord {
    pub device_id: DeviceId,
    pub generation: DeviceGeneration,
    pub kind: DeviceFaultKind,
    pub message: String,
}

/// Fail-closed result of checking whether a device generation may admit new work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceAdmissionError {
    FaultRecording,
    Faulted { kind: DeviceFaultKind },
}

/// First-fault gate. The healthy admission path is one acquire load with no allocation or lock.
pub struct DeviceFaultGate {
    device_id: DeviceId,
    generation: DeviceGeneration,
    state: AtomicU8,
    first_fault: Mutex<Option<DeviceFaultRecord>>,
}

impl DeviceFaultGate {
    pub fn new(device_id: DeviceId, generation: DeviceGeneration) -> Self {
        Self {
            device_id,
            generation,
            state: AtomicU8::new(HEALTHY),
            first_fault: Mutex::new(None),
        }
    }

    /// Fast path used by future resource, command, and submission admission owners.
    pub fn ensure_admission(&self) -> Result<(), DeviceAdmissionError> {
        match self.state.load(Ordering::Acquire) {
            HEALTHY => Ok(()),
            RECORDING => Err(DeviceAdmissionError::FaultRecording),
            FAULTED => Err(DeviceAdmissionError::Faulted {
                kind: self
                    .first_fault()
                    .map(|fault| fault.kind)
                    .unwrap_or(DeviceFaultKind::Internal),
            }),
            _ => unreachable!("device fault gate state is constrained to known values"),
        }
    }

    /// Records exactly one failure. Competing callbacks fail closed without allocating or locking.
    pub fn record_first(&self, kind: DeviceFaultKind, message: impl Into<String>) -> bool {
        if self
            .state
            .compare_exchange(HEALTHY, RECORDING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }

        let record = DeviceFaultRecord {
            device_id: self.device_id,
            generation: self.generation,
            kind,
            message: message.into(),
        };
        let mut first_fault = self
            .first_fault
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *first_fault = Some(record);
        self.state.store(FAULTED, Ordering::Release);
        true
    }

    /// Slow-path diagnostic snapshot. The returned string is cloned only for observers.
    pub fn first_fault(&self) -> Option<DeviceFaultRecord> {
        self.first_fault
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}
