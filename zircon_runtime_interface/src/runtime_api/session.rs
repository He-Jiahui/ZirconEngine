use crate::buffer::ZrByteSlice;
use crate::version::{ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V3};

/// Session-scoped host wake callback carried across the runtime DLL boundary.
///
/// `token` is opaque to the runtime. A sink is either fully disabled
/// (`token == 0`, `wake == None`) or fully registered (`token != 0`,
/// `wake == Some`). The callback must return promptly. Synchronous destruction of the same
/// session from inside `wake` is rejected so the callback cannot wait on its own quiescence.
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
pub struct ZrRuntimeSessionConfigV3 {
    pub abi_version: u32,
    pub profile: ZrByteSlice,
    /// Physical project-root anchor. All project-local paths are resolved below this root.
    pub project_root: ZrByteSlice,
    /// Optional project-relative versioned DynamicScene document for Play-in-Editor startup.
    pub play_scene: ZrByteSlice,
    /// Optional logical startup report outlet name.
    pub play_report_pipe: ZrByteSlice,
    pub wake_sink: ZrRuntimeWakeSinkV1,
}

impl ZrRuntimeSessionConfigV3 {
    pub const fn empty() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
            profile: ZrByteSlice::empty(),
            project_root: ZrByteSlice::empty(),
            play_scene: ZrByteSlice::empty(),
            play_report_pipe: ZrByteSlice::empty(),
            wake_sink: ZrRuntimeWakeSinkV1::disabled(),
        }
    }

    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V3 && self.wake_sink.is_valid()
    }
}
