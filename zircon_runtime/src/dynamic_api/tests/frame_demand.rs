use super::support::*;

unsafe extern "C" fn test_wake(_token: u64) {}

#[test]
fn create_session_rejects_bad_wake_sink_pairs_and_version_before_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();

    for wake_sink in [
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            token: 0,
            wake: None,
        },
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 1,
            wake: None,
        },
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 0,
            wake: Some(test_wake),
        },
    ] {
        let status = unsafe {
            create_session(
                ZrRuntimeSessionConfigV3 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                    profile: ZrByteSlice::from_static(b"headless"),
                    project_root: ZrByteSlice::empty(),
                    play_scene: ZrByteSlice::empty(),
                    play_report_pipe: ZrByteSlice::empty(),
                    wake_sink,
                },
                &mut session,
            )
        };

        assert_session_status(
            status,
            ZrStatusCode::InvalidArgument,
            "invalid runtime wake sink",
        );
        assert!(!session.is_valid());
    }
}

#[test]
fn tick_frame_requires_output_and_writes_a_checked_demand_carrier() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");
    let session = create_test_session(api);

    assert_session_status(
        unsafe { tick_frame(session, core::ptr::null_mut()) },
        ZrStatusCode::InvalidArgument,
        "missing runtime frame demand output",
    );

    let mut demand = ZrRuntimeFrameDemandV1 {
        abi_version: 0,
        kind: u32::MAX,
        delay_nanoseconds: u64::MAX,
    };
    let status = unsafe { tick_frame(session, &mut demand) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(demand.is_valid(), "{demand:?}");
    assert!(demand.has_known_kind(), "{demand:?}");
}
