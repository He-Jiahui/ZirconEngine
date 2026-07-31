use crate::version::ZIRCON_RUNTIME_ABI_VERSION_V1;

pub const ZR_RUNTIME_FRAME_DEMAND_IDLE_V1: u32 = 0;
pub const ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1: u32 = 1;
pub const ZR_RUNTIME_FRAME_DEMAND_AFTER_V1: u32 = 2;

/// Raw ABI carrier for the runtime's next-frame request.
///
/// `kind` deliberately remains a `u32`. Consumers must check it before
/// constructing any crate-local Rust enum.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeFrameDemandV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub delay_nanoseconds: u64,
}

impl ZrRuntimeFrameDemandV1 {
    pub const fn idle() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
            delay_nanoseconds: 0,
        }
    }

    pub const fn immediate() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
            delay_nanoseconds: 0,
        }
    }

    pub const fn after(delay_nanoseconds: u64) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
            delay_nanoseconds,
        }
    }

    pub const fn has_known_kind(self) -> bool {
        matches!(
            self.kind,
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1
                | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1
                | ZR_RUNTIME_FRAME_DEMAND_AFTER_V1
        )
    }

    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && match self.kind {
                ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => {
                    self.delay_nanoseconds == 0
                }
                ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => true,
                _ => false,
            }
    }
}
