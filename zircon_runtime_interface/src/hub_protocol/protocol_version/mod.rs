mod serde;

pub use serde::HubProtocolVersionV1;

/// The only file-mailbox protocol revision currently supported by Hub and Editor.
pub const HUB_PROTOCOL_VERSION_V1: u32 = 1;
