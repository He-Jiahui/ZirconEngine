use crate::buffer::ZrByteSlice;
use crate::version::{ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2};

/// Session-scoped host wake callback carried across the runtime DLL boundary.
///
/// `token` is opaque to the runtime. A sink is either fully disabled
/// (`token == 0`, `wake == None`) or fully registered (`token != 0`,
/// `wake == Some`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeWakeSinkV1 {
    pub abi_version: u32,
    pub token: u64,
    pub wake: Option<unsafe extern "C" fn(u64)>,
}

impl ZrRuntimeWakeSinkV1 {
    pub const fn disabled() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 0,
            wake: None,
        }
    }

    pub const fn new(token: u64, wake: unsafe extern "C" fn(u64)) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token,
            wake: Some(wake),
        }
    }

    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && matches!((self.token, self.wake), (0, None) | (1.., Some(_)))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeSessionConfigV2 {
    pub abi_version: u32,
    pub profile: ZrByteSlice,
    pub project_manifest: ZrByteSlice,
    pub wake_sink: ZrRuntimeWakeSinkV1,
}

impl ZrRuntimeSessionConfigV2 {
    pub const fn empty() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            profile: ZrByteSlice::empty(),
            project_manifest: ZrByteSlice::empty(),
            wake_sink: ZrRuntimeWakeSinkV1::disabled(),
        }
    }

    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V2 && self.wake_sink.is_valid()
    }
}
