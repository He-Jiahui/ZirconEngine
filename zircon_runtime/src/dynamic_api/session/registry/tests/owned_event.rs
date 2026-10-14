//! 从真实事件 FFI 验证负载复制、调用线程诊断和短销毁重试下的交付责任。
//!
//! 队列准入与事件执行分开观察，已准入动作的槽位 guard 持续保留到结果取得。

use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
};

use super::super::session_store::{destroy_session_slot_with_timeout, session_active_actions};
use super::super::{
    destroy_session_slot, insert_session_with_wake, session_is_closing, with_session,
    RuntimeWakeRegistration,
};
use crate::core::framework::input::{FileDragDropEvent, InputEvent};
use crate::core::manager::resolve_manager_service;
use crate::dynamic_api::session::ffi;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;

#[test]
/// owner 返回的失败诊断应在调用线程可读，随后真实销毁回收同一会话。
fn owned_event_preserves_status_diagnostics_on_the_calling_thread() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    // SAFETY: 事件不借用负载，handle 对应仍存活的真实测试会话；调用同步完成。
    let unsupported = unsafe {
        ffi::handle_event(
            handle,
            ZrRuntimeEventV1::new(
                u32::MAX,
                u32::MAX,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ),
        )
    };
    assert_eq!(unsupported.status_code(), ZrStatusCode::UnsupportedVersion);
    // SAFETY: 诊断由刚完成的调用提供，当前线程没有再次覆盖它，且长度在读取预算内。
    assert_eq!(
        unsafe { unsupported.diagnostics.checked_slice(4096) }.unwrap(),
        b"unsupported runtime ABI version"
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
/// 无效句柄仍先经过 FFI 载体检查；超预算与空指针非零长度都必须在分派前被拒绝。
fn owned_event_rejects_oversized_and_invalid_payloads_before_dispatch() {
    let invalid_handle = ZrRuntimeSessionHandle::new(0);
    let oversized = vec![b'x'; ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 + 1];
    // SAFETY: Vec 保持整个调用期间可读；超预算载体在 checked_slice 解引用前被拒绝。
    let oversized_status = unsafe {
        ffi::handle_event(
            invalid_handle,
            ZrRuntimeEventV1::file_dropped(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                ZrByteSlice {
                    data: oversized.as_ptr(),
                    len: oversized.len(),
                },
            ),
        )
    };
    assert_eq!(oversized_status.status_code(), ZrStatusCode::LimitExceeded);
    // SAFETY: 此分支返回静态诊断，读取在预算内且不延长提供方借用。
    assert_eq!(
        unsafe { oversized_status.diagnostics.checked_slice(4096) }.unwrap(),
        b"runtime event payload exceeds limit"
    );

    // SAFETY: 此测试特意传入空指针非零长度；checked_slice 的形状拒绝分支不会解引用它。
    let invalid_status = unsafe {
        ffi::handle_event(
            invalid_handle,
            ZrRuntimeEventV1::file_dropped(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                ZrByteSlice {
                    data: ptr::null(),
                    len: 1,
                },
            ),
        )
    };
    assert_eq!(invalid_status.status_code(), ZrStatusCode::InvalidArgument);
    // SAFETY: 无效载体分支返回静态诊断，预算内读取不依赖被拒绝的负载指针。
    assert_eq!(
        unsafe { invalid_status.diagnostics.checked_slice(4096) }.unwrap(),
        b"invalid runtime event payload slice"
    );
}

#[test]
/// owner 被前一动作占用时，拥有型事件跨两次短 destroy 仍保留准入责任并只交付一次。
fn astra_life_a4_blocked_owner_payload_event_survives_short_destroy_retries() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let input_manager = session.input_manager.clone();
    let core = session.runtime.handle().downgrade();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let (owner_entered_tx, owner_entered_rx) = mpsc::sync_channel(1);
    let (release_owner_tx, release_owner_rx) = mpsc::sync_channel(1);
    let owner_action = thread::spawn(move || {
        with_session(handle, move |_| {
            owner_entered_tx.send(()).unwrap();
            release_owner_rx.recv().unwrap();
            ZrStatus::ok()
        })
        .status_code()
    });
    owner_entered_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    let event_action = thread::spawn(move || {
        let path = b"C:/tmp/asset.png".to_vec();
        let event = ZrRuntimeEventV1::file_dropped(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ZrByteSlice {
                data: path.as_ptr(),
                len: path.len(),
            },
        );
        // SAFETY: path 的 Vec 在此同步调用返回前存活，FFI 在排队前复制字节；
        // 返回码读取不持有或延长原事件负载的借用。
        unsafe { ffi::handle_event(handle, event) }.status_code()
    });
    // 两个活动 guard 证明阻塞动作和事件都已登记，随后才测试 Closing 的短等待。
    let admission_deadline = Instant::now() + Duration::from_secs(2);
    while session_active_actions(handle) != Some(2) {
        assert!(
            Instant::now() < admission_deadline,
            "event was not admitted"
        );
        thread::yield_now();
    }
    assert!(!event_action.is_finished());

    let started_at = Instant::now();
    assert_eq!(
        destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code(),
        ZrStatusCode::Error
    );
    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert!(session_is_closing(handle));
    assert_eq!(
        destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code(),
        ZrStatusCode::Error
    );
    assert!(!event_action.is_finished());

    // 先放行并 join 自有调用线程，再检查原 Core 的输入记录，最终完整销毁会话。
    release_owner_tx.send(()).unwrap();
    assert_eq!(owner_action.join().unwrap(), ZrStatusCode::Ok);
    assert_eq!(event_action.join().unwrap(), ZrStatusCode::Ok);
    let expected = InputEvent::FileDragDrop(FileDragDropEvent::Dropped {
        path: "C:/tmp/asset.png".to_owned(),
    });
    assert_eq!(
        resolve_manager_service(
            &core
                .upgrade()
                .expect("session still owns its runtime before final destroy"),
            input_manager.clone(),
        )
        .expect("input observation must be admitted before final destroy")
        .drain_events()
        .iter()
        .filter(|event| *event == &expected)
        .count(),
        1
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}
