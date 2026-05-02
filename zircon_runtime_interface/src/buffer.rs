use core::ptr;

use crate::status::ZrStatus;

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

    pub unsafe fn as_slice<'a>(self) -> &'a [u8] {
        if self.data.is_null() || self.len == 0 {
            &[]
        } else {
            core::slice::from_raw_parts(self.data, self.len)
        }
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
