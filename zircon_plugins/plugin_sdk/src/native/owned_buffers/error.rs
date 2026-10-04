use super::super::{
    callback_status, NativePluginCallbackStatusV3, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
};

/// Why an SDK byte allocation could not be published to a native consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginOwnedBytesErrorKind {
    AllocationIdsExhausted,
    RegistryCapacity,
}

/// An unpublished allocation. The caller can recover the original bytes.
pub struct NativePluginOwnedBytesError {
    pub(super) kind: NativePluginOwnedBytesErrorKind,
    pub(super) bytes: Vec<u8>,
}

impl NativePluginOwnedBytesError {
    pub fn kind(&self) -> NativePluginOwnedBytesErrorKind {
        self.kind
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Maps registration failure to the existing callback status ABI.
    pub fn status(&self) -> NativePluginCallbackStatusV3 {
        let diagnostics: &'static [u8] = match self.kind {
            NativePluginOwnedBytesErrorKind::AllocationIdsExhausted => {
                b"native plugin SDK allocation IDs exhausted\0"
            }
            NativePluginOwnedBytesErrorKind::RegistryCapacity => {
                b"native plugin SDK allocation registry capacity unavailable\0"
            }
        };
        callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, diagnostics)
    }
}

impl std::fmt::Debug for NativePluginOwnedBytesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginOwnedBytesError")
            .field("kind", &self.kind)
            .field("len", &self.bytes.len())
            .field("capacity", &self.bytes.capacity())
            .finish()
    }
}
