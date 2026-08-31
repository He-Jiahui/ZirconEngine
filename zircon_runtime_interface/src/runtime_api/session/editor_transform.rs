use core::mem::size_of;

use thiserror::Error;

use crate::buffer::{ZrByteSlice, ZrByteSliceError};
use crate::math::{Quat, Transform, Vec3};
use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeEditorTransformPhaseV1 {
    Begin = 1,
    Preview = 2,
    Commit = 3,
    Cancel = 4,
    Apply = 5,
}

impl ZrRuntimeEditorTransformPhaseV1 {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Begin),
            2 => Some(Self::Preview),
            3 => Some(Self::Commit),
            4 => Some(Self::Cancel),
            5 => Some(Self::Apply),
            _ => None,
        }
    }
}

/// Fixed-layout math carrier for editor transform traffic across the runtime ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTransformV1 {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl ZrRuntimeTransformV1 {
    pub fn new(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: transform.rotation.to_array(),
            scale: transform.scale.to_array(),
        }
    }

    pub fn transform(self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_array(self.rotation),
            scale: Vec3::from_array(self.scale),
        }
    }

    pub fn is_valid(self) -> bool {
        self.translation
            .into_iter()
            .chain(self.rotation)
            .chain(self.scale)
            .all(f32::is_finite)
            && self
                .rotation
                .into_iter()
                .map(|component| component * component)
                .sum::<f32>()
                > f32::EPSILON
            && self.scale.into_iter().all(|component| component != 0.0)
    }
}

/// Allocation-free compare-and-set transform write for editor-owned runtime interaction.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeEditorTransformWriteV1 {
    pub abi_version: u32,
    pub phase: u32,
    pub entity: u64,
    pub interaction_id: u64,
    pub sequence: u64,
    pub world_replacement_epoch: u64,
    pub expected: ZrRuntimeTransformV1,
    pub target: ZrRuntimeTransformV1,
    pub reserved: u64,
}

impl ZrRuntimeEditorTransformWriteV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity: u64,
        interaction_id: u64,
        sequence: u64,
        world_replacement_epoch: u64,
        phase: ZrRuntimeEditorTransformPhaseV1,
        expected: Transform,
        target: Transform,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            phase: phase.raw(),
            entity,
            interaction_id,
            sequence,
            world_replacement_epoch,
            expected: ZrRuntimeTransformV1::new(expected),
            target: ZrRuntimeTransformV1::new(target),
            reserved: 0,
        }
    }

    pub const fn phase(self) -> Option<ZrRuntimeEditorTransformPhaseV1> {
        ZrRuntimeEditorTransformPhaseV1::from_raw(self.phase)
    }

    pub fn expected_transform(self) -> Transform {
        self.expected.transform()
    }

    pub fn target_transform(self) -> Transform {
        self.target.transform()
    }

    pub fn validate_editor_transform_write(self) -> bool {
        let Some(phase) = self.phase() else {
            return false;
        };
        let sequence_valid = match phase {
            ZrRuntimeEditorTransformPhaseV1::Begin | ZrRuntimeEditorTransformPhaseV1::Apply => {
                self.sequence == 1
            }
            ZrRuntimeEditorTransformPhaseV1::Preview
            | ZrRuntimeEditorTransformPhaseV1::Commit
            | ZrRuntimeEditorTransformPhaseV1::Cancel => self.sequence > 1,
        };
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.entity != 0
            && self.interaction_id != 0
            && self.world_replacement_epoch != 0
            && sequence_valid
            && self.expected.is_valid()
            && self.target.is_valid()
            && self.reserved == 0
    }

    /// Borrows this request as a payload for one synchronous `handle_event` call.
    pub fn as_payload(&self) -> ZrByteSlice {
        ZrByteSlice {
            data: core::ptr::from_ref(self).cast(),
            len: size_of::<Self>(),
        }
    }

    /// Decodes one fixed-layout request borrowed for the duration of the ABI call.
    ///
    /// # Safety
    ///
    /// A non-empty payload pointer must address initialized bytes for the duration of this call.
    pub unsafe fn from_payload(
        payload: ZrByteSlice,
    ) -> Result<Self, ZrRuntimeEditorTransformError> {
        let bytes = unsafe { payload.checked_slice(size_of::<Self>()) }
            .map_err(ZrRuntimeEditorTransformError::Payload)?;
        if bytes.len() != size_of::<Self>() {
            return Err(ZrRuntimeEditorTransformError::Size {
                observed: bytes.len(),
                expected: size_of::<Self>(),
            });
        }
        let request = unsafe { core::ptr::read_unaligned(bytes.as_ptr().cast::<Self>()) };
        if !request.validate_editor_transform_write() {
            return Err(ZrRuntimeEditorTransformError::InvalidRequest);
        }
        Ok(request)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeEditorTransformError {
    #[error("invalid editor transform payload carrier: {0:?}")]
    Payload(ZrByteSliceError),
    #[error("editor transform payload has {observed} bytes; expected {expected}")]
    Size { observed: usize, expected: usize },
    #[error("editor transform request is structurally invalid")]
    InvalidRequest,
}
