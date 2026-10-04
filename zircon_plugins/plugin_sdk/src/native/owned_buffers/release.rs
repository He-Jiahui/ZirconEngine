use super::super::{
    callback_status, NativePluginCallbackStatusV3, NativePluginOwnedByteBufferV3,
    NATIVE_EMPTY_CSTR, NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS,
    NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::registry::{lock_registry, REGISTRY};

pub(in super::super) fn release(
    buffer: NativePluginOwnedByteBufferV3,
) -> NativePluginCallbackStatusV3 {
    if buffer.data.is_null() {
        return if buffer.len == 0 && buffer.capacity == 0 && buffer.owner_token == 0 {
            callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
        } else {
            callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS,
            )
        };
    }
    if buffer.len > buffer.capacity || buffer.capacity == 0 || buffer.owner_token == 0 {
        return callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS,
        );
    }
    let Some(registry) = REGISTRY.get() else {
        return callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS,
        );
    };
    let allocation = { lock_registry(registry).consume(buffer) };
    match allocation {
        Some(bytes) => {
            // Vec<u8> has no user destructor. Deallocate the real allocation,
            // using its real capacity, only after releasing the registry lock.
            drop(bytes);
            callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
        }
        None => callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS,
        ),
    }
}
