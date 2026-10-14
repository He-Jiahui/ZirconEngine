//! Windows watcher 线程 TLS 析构未结束时，短销毁必须保留会话和重试责任。
//!
//! 本模块由 registry/tests/cases 的 windows 条件接入；使用真实线程退出屏障。

use std::cell::RefCell;

use super::*;

/// 放在真实 worker 的 TLS 中，Drop 通知进入析构并等待测试主线程放行。
struct ThreadExitLatch {
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

impl Drop for ThreadExitLatch {
    fn drop(&mut self) {
        // 有限等待防止回归把测试 worker 永久留住；回执仍须确认真正的线程退出。
        let _ = self.entered.send(());
        let _ = self.release.recv_timeout(Duration::from_secs(2));
    }
}

thread_local! {
    // 不从 TLS 取出此值，确保屏障发生在操作系统线程终止前的真实 TLS 析构阶段。
    static EXIT_LATCH: RefCell<Option<ThreadExitLatch>> = const { RefCell::new(None) };
}

#[test]
/// watcher 主函数退出后仍堵在 TLS Drop；短销毁与零预算重试均不得提前删除槽位。
fn astra_life_a4_destroy_retains_project_watcher_during_tls_destruction() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let core = session.runtime.handle();
    let manager_handle = crate::asset::project_asset_manager_handle(&core).unwrap();
    let manager = resolve_manager_service(&core, manager_handle).unwrap();
    let (stop_tx, stop_rx) = crossbeam_channel::bounded(1);
    let (entered, entered_rx) = mpsc::sync_channel(1);
    let (release, release_rx) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        EXIT_LATCH.with(|slot| {
            *slot.borrow_mut() = Some(ThreadExitLatch {
                entered,
                release: release_rx,
            });
        });
        let _ = stop_rx.recv_timeout(Duration::from_secs(2));
    });
    manager.install_project_watcher_for_test(AssetWatcher::from_test_worker(stop_tx, worker));
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());

    let started_at = Instant::now();
    let first = destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code();
    let elapsed = started_at.elapsed();
    let tls_entered = entered_rx.recv_timeout(Duration::from_secs(1));
    let closing = session_is_closing(handle);
    let retry = destroy_session_slot_with_timeout(handle, Duration::ZERO).status_code();
    let retry_closing = session_is_closing(handle);

    // Release and reap before asserting so a regression cannot strand a session worker.
    // 先解除 TLS 屏障并让最终 destroy 回收 worker，再断言失败分类和重复销毁结果。
    let _ = release.send(());
    let completed = destroy_session_slot(handle).status_code();
    let repeated = destroy_session_slot(handle).status_code();

    assert!(tls_entered.is_ok());
    assert!(elapsed < Duration::from_secs(1));
    assert_eq!(first, ZrStatusCode::Error);
    assert!(closing);
    assert_eq!(retry, ZrStatusCode::Error);
    assert!(retry_closing);
    assert_eq!(completed, ZrStatusCode::Ok);
    assert_eq!(repeated, ZrStatusCode::NotFound);
}
