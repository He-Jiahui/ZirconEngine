//! Downstream valid-contract regression draft. Requires the actual SDK native
//! feature; these cases never replay/forge the unsafe public free callback.
use zircon_plugin_sdk::native::{
    owned_bytes, NativePluginOwnedByteBufferV3, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

#[test]
fn native_owned_buffer_payload_survives_registration_until_real_free_callback() {
    let input = b"actual downstream SDK payload".to_vec();
    let original_pointer = input.as_ptr();
    let original_capacity = input.capacity();
    let buffer = owned_bytes(input).expect("register real allocation");
    assert_eq!(buffer.data.cast_const(), original_pointer);
    assert_eq!(buffer.capacity, original_capacity);
    assert_ne!(buffer.owner_token, 0);
    // SAFETY: the actual SDK registry owns this unchanged allocation until free;
    // this is its only read and only release, and the SDK image remains loaded.
    assert_eq!(
        unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) },
        b"actual downstream SDK payload"
    );
    assert_eq!(
        unsafe { buffer.free.unwrap()(buffer) }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
}

#[test]
fn native_owned_buffer_empty_retained_capacity_returns_real_callback() {
    let empty = owned_bytes(Vec::new()).unwrap();
    assert!(empty.data.is_null());
    assert_eq!(empty.owner_token, 0);
    assert!(empty.free.is_none());
    let retained = owned_bytes(Vec::with_capacity(16)).unwrap();
    assert_eq!(retained.len, 0);
    assert!(retained.capacity >= 16);
    assert_ne!(retained.owner_token, 0);
    // SAFETY: one valid release of the actual retained-capacity Vec.
    assert_eq!(
        unsafe { retained.free.unwrap()(retained) }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
}

#[test]
fn native_owned_buffer_layout_preserves_v3_c_field_order() {
    use std::mem::offset_of;
    assert_eq!(offset_of!(NativePluginOwnedByteBufferV3, data), 0);
    assert!(
        offset_of!(NativePluginOwnedByteBufferV3, data)
            < offset_of!(NativePluginOwnedByteBufferV3, len)
    );
    assert!(
        offset_of!(NativePluginOwnedByteBufferV3, len)
            < offset_of!(NativePluginOwnedByteBufferV3, capacity)
    );
    assert!(
        offset_of!(NativePluginOwnedByteBufferV3, capacity)
            < offset_of!(NativePluginOwnedByteBufferV3, owner_token)
    );
    assert!(
        offset_of!(NativePluginOwnedByteBufferV3, owner_token)
            < offset_of!(NativePluginOwnedByteBufferV3, free)
    );
    // This test is a source layout guard, not detached-C or cross-image proof.
}
