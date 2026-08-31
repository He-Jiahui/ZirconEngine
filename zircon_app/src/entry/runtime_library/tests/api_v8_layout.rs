use super::super::loaded_runtime::runtime_api_field_available;
use super::{
    fake_drain_host_requests, fake_profile_control, fake_tick_frame, valid_runtime_api_table,
};
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeDrainHostRequestsFnV2, ZrRuntimeProfileControlFnV2, ZrRuntimeTickFrameFnV2,
};
use zircon_runtime_interface::ZrRuntimeApiV8;

#[test]
fn runtime_api_profile_control_is_optional_after_present_prefix() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV8>();
    let before_profile = core::mem::offset_of!(ZrRuntimeApiV8, profile_control);
    let api = ZrRuntimeApiV8 {
        profile_control: Some(fake_profile_control as _),
        ..ZrRuntimeApiV8::empty()
    };

    assert!(runtime_api_field_available(
        full_size,
        before_profile,
        core::mem::size_of_val(&api.profile_control)
    ));
    assert!(!runtime_api_field_available(
        before_profile,
        before_profile,
        core::mem::size_of_val(&api.profile_control)
    ));
}

#[test]
fn runtime_api_tick_frame_follows_profile_control_in_v8_layout() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV8>();
    let before_tick = core::mem::offset_of!(ZrRuntimeApiV8, tick_frame);
    let api = ZrRuntimeApiV8 {
        tick_frame: Some(fake_tick_frame as _),
        ..ZrRuntimeApiV8::empty()
    };

    assert_eq!(
        before_tick,
        core::mem::offset_of!(ZrRuntimeApiV8, profile_control)
            + core::mem::size_of::<Option<ZrRuntimeProfileControlFnV2>>()
    );
    assert!(runtime_api_field_available(
        full_size,
        before_tick,
        core::mem::size_of::<Option<ZrRuntimeTickFrameFnV2>>()
    ));
    assert!(!runtime_api_field_available(
        before_tick,
        before_tick,
        core::mem::size_of_val(&api.tick_frame)
    ));
}

#[test]
fn runtime_api_drain_host_requests_is_optional_after_tick_frame() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV8>();
    let before_drain = core::mem::offset_of!(ZrRuntimeApiV8, drain_host_requests);
    let api = ZrRuntimeApiV8 {
        drain_host_requests: Some(fake_drain_host_requests as _),
        ..ZrRuntimeApiV8::empty()
    };

    assert_eq!(
        before_drain,
        core::mem::offset_of!(ZrRuntimeApiV8, tick_frame)
            + core::mem::size_of::<Option<ZrRuntimeTickFrameFnV2>>()
    );
    assert!(runtime_api_field_available(
        full_size,
        before_drain,
        core::mem::size_of::<Option<ZrRuntimeDrainHostRequestsFnV2>>()
    ));
    assert!(!runtime_api_field_available(
        before_drain,
        before_drain,
        core::mem::size_of_val(&api.drain_host_requests)
    ));
}

#[test]
fn runtime_operation_api_precedes_the_v8_world_sync_tail() {
    let api = ZrRuntimeApiV8::empty();
    let full_size = core::mem::size_of::<ZrRuntimeApiV8>();
    for (offset, field_size) in [
        (
            core::mem::offset_of!(ZrRuntimeApiV8, submit_operation),
            core::mem::size_of_val(&api.submit_operation),
        ),
        (
            core::mem::offset_of!(ZrRuntimeApiV8, poll_operation),
            core::mem::size_of_val(&api.poll_operation),
        ),
        (
            core::mem::offset_of!(ZrRuntimeApiV8, harvest_operation),
            core::mem::size_of_val(&api.harvest_operation),
        ),
    ] {
        assert!(runtime_api_field_available(full_size, offset, field_size));
        assert!(!runtime_api_field_available(offset, offset, field_size));
    }
}

#[test]
fn runtime_api_pointer_rejects_oversized_frozen_v8_table() {
    let mut api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8);
    api.size_bytes += 1;

    let error = unsafe { super::super::loaded_runtime::validate_runtime_api_pointer(&api) }
        .expect_err("the frozen V8 table must not accept same-version extensions");

    assert_eq!(
        error.to_string(),
        format!(
            "runtime API table size {} does not match frozen v8 layout of {} bytes",
            api.size_bytes,
            core::mem::size_of::<ZrRuntimeApiV8>()
        )
    );
}
