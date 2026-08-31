/// First stable runtime dynamic-library ABI version.
pub const ZIRCON_RUNTIME_ABI_VERSION_V1: u32 = 1;

/// ABI version for the changed V2 runtime session config carrier.
pub const ZIRCON_RUNTIME_ABI_VERSION_V2: u32 = 2;

/// ABI version for the V3 runtime session configuration carrier.
pub const ZIRCON_RUNTIME_ABI_VERSION_V3: u32 = 3;

/// Current runtime dynamic-library function-table version, generated from the frozen InterfaceSpec.
pub use crate::runtime_build_set::ZIRCON_RUNTIME_API_VERSION_V8;
