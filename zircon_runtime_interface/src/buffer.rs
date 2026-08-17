use core::ptr;

use crate::handles::ZrRuntimeAllocationId;
use crate::runtime_api::{
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};
use crate::status::ZrStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrByteSliceError {
    NullWithNonZeroLength,
    LengthExceedsAddressSpace { len: usize },
    LengthExceedsLimit { len: usize, limit: usize },
}

impl ZrByteSliceError {
    pub const fn is_limit_exceeded(self) -> bool {
        matches!(self, Self::LengthExceedsLimit { .. })
    }
}

pub const ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1: usize = 128;
pub const ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1: usize = 4 * 1024;
pub const ZR_RUNTIME_SESSION_PROFILE_MAX_ENCODED_BYTES_V1: usize = 64;
pub const ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1: usize = 32 * 1024;
pub const ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;
pub const ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;
pub const ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1: usize = 16_384;

pub const ZR_RUNTIME_FRAME_MAX_DIMENSION_V1: u32 = 16_384;
pub const ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimePayloadLimitV1 {
    pub max_encoded_bytes: usize,
    pub max_items: usize,
    pub max_nesting_depth: usize,
    pub max_processing_time_micros: u64,
    pub allow_empty: bool,
}

impl ZrRuntimePayloadLimitV1 {
    pub const fn new(
        max_encoded_bytes: usize,
        max_items: usize,
        max_processing_time_micros: u64,
    ) -> Self {
        Self {
            max_encoded_bytes,
            max_items,
            max_nesting_depth: ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1,
            max_processing_time_micros,
            allow_empty: false,
        }
    }

    pub const fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }
}

pub const ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 1_024, 25_000);
pub const ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 64, 10_000);
pub const ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
pub const ZR_RUNTIME_WORLD_QUERY_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
pub const ZR_RUNTIME_WORLD_WATCH_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 1_024, 10_000);
pub const ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 4_096, 10_000);

pub const ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(16 * 1024 * 1024, 65_536, 250_000);
pub const ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 256, 10_000).allow_empty();
pub const ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(16 * 1024 * 1024, 65_536, 250_000).allow_empty();
pub const ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
pub const ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
        10_000,
    )
    .allow_empty();
pub const ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
pub const ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000).allow_empty();

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrByteSlice {
    pub data: *const u8,
    pub len: usize,
}

impl ZrByteSlice {
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }

    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Validates and views this ABI slice as a Rust byte slice.
    ///
    /// # Safety
    ///
    /// After this method validates the carrier shape and length, a non-empty `data`
    /// pointer must still point to `len` initialized bytes that remain valid for the
    /// returned lifetime.
    pub unsafe fn checked_slice<'a>(self, limit: usize) -> Result<&'a [u8], ZrByteSliceError> {
        if self.len == 0 {
            return Ok(&[]);
        }
        if self.data.is_null() {
            return Err(ZrByteSliceError::NullWithNonZeroLength);
        }
        if self.len > isize::MAX as usize {
            return Err(ZrByteSliceError::LengthExceedsAddressSpace { len: self.len });
        }
        if self.len > limit {
            return Err(ZrByteSliceError::LengthExceedsLimit {
                len: self.len,
                limit,
            });
        }
        Ok(unsafe { core::slice::from_raw_parts(self.data, self.len) })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrByteBufferRef {
    pub data: *mut u8,
    pub capacity: usize,
    pub written: *mut usize,
}

impl ZrByteBufferRef {
    pub const fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            capacity: 0,
            written: ptr::null_mut(),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.data.is_null() && self.capacity == 0
    }
}

pub type ZrFreeBytesFn = unsafe extern "C" fn(ZrOwnedByteBuffer) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrOwnedByteBuffer {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<ZrFreeBytesFn>,
}

impl ZrOwnedByteBuffer {
    pub const fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.data.is_null() && self.len == 0 && self.capacity == 0
    }
}

/// Immutable runtime-owned output whose allocation is released by opaque ID.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct ZrOwnedResultV2 {
    pub data: *const u8,
    pub len: u64,
    pub allocation: ZrRuntimeAllocationId,
}

impl ZrOwnedResultV2 {
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
            allocation: ZrRuntimeAllocationId::invalid(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.data.is_null() && self.len == 0 && !self.allocation.is_valid()
    }
}
