use crate::buffer::ZrByteSlice;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrStatusCode {
    Ok = 0,
    Error = 1,
    UnsupportedVersion = 2,
    InvalidArgument = 3,
    NotFound = 4,
    CapabilityDenied = 5,
    Panic = 6,
}

impl ZrStatusCode {
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::Error,
            2 => Self::UnsupportedVersion,
            3 => Self::InvalidArgument,
            4 => Self::NotFound,
            5 => Self::CapabilityDenied,
            6 => Self::Panic,
            _ => Self::Error,
        }
    }

    pub const fn as_raw(self) -> u32 {
        self as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrStatus {
    pub code: u32,
    pub diagnostics: ZrByteSlice,
}

impl ZrStatus {
    pub const fn ok() -> Self {
        Self::new(ZrStatusCode::Ok, ZrByteSlice::empty())
    }

    pub const fn new(code: ZrStatusCode, diagnostics: ZrByteSlice) -> Self {
        Self {
            code: code.as_raw(),
            diagnostics,
        }
    }

    pub const fn status_code(self) -> ZrStatusCode {
        ZrStatusCode::from_raw(self.code)
    }

    pub const fn is_ok(self) -> bool {
        self.code == ZrStatusCode::Ok.as_raw()
    }
}
