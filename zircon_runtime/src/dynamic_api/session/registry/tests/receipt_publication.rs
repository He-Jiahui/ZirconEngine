//! Windows-only test scheduling/data for one real public startup attempt.
//!
//! 为真实公开启动失败附着调度屏障，观察同一 owner、原回执 Mutex 和 worker TLS 退出。
//! hook 只安排次序与记录真实值，不修改清理返回值或原子成功状态；公开回归入口位于
//! `construction/tests/receipt_publication.rs`，运行与生成夹具由该测试的父进程负责。

use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use crate::dynamic_api::session::{RuntimeDynamicSessionError, RuntimeProjectError};
use zircon_runtime_interface::ZrRuntimeSessionHandle;

use super::runtime::startup_observation::RuntimeStartupObservation;
use super::{CleanupReceipt, OwnerShutdownReceipt, RECEIPT_PENDING, RECEIPT_SUCCEEDED};

// 见证只保存身份、原错误与关闭观察，不能扩展为源码或不受限日志副本。
const MAX_WITNESS_BYTES: usize = 32 * 1024;

// 按仍存活的真实 receipt 地址隔离普通 owner 用例，不改公开启动回归的调度。
static WAIT_GUARD_BARRIERS: Mutex<Vec<Arc<WaitGuardBarrier>>> = Mutex::new(Vec::new());

struct WaitGuardBarrier {
    receipt: usize,
    released: mpsc::SyncSender<(usize, Instant)>,
    publisher_locked: mpsc::SyncSender<()>,
    release_waiter: Mutex<mpsc::Receiver<()>>,
    release_publisher: Mutex<mpsc::Receiver<()>>,
}

/// 登记期间调用方必须保留同一 Arc receipt，且在注销前放行并 join 原 owner。
pub(super) struct WaitGuardRegistration(Arc<WaitGuardBarrier>);

impl Drop for WaitGuardRegistration {
    fn drop(&mut self) {
        WAIT_GUARD_BARRIERS
            .lock()
            .expect("wait guard barriers")
            .retain(|entry| !Arc::ptr_eq(entry, &self.0));
    }
}

/// 只登记次序屏障，不替换 Mutex、原子回执值或原 Shutdown 命令。
pub(super) fn observe_wait_guard_for_test(
    receipt: &CleanupReceipt,
    released: mpsc::SyncSender<(usize, Instant)>,
    publisher_locked: mpsc::SyncSender<()>,
    release_waiter: mpsc::Receiver<()>,
    release_publisher: mpsc::Receiver<()>,
) -> WaitGuardRegistration {
    let barrier = Arc::new(WaitGuardBarrier {
        receipt: receipt as *const CleanupReceipt as usize,
        released,
        publisher_locked,
        release_waiter: Mutex::new(release_waiter),
        release_publisher: Mutex::new(release_publisher),
    });
    let mut barriers = WAIT_GUARD_BARRIERS.lock().expect("wait guard barriers");
    assert!(!barriers
        .iter()
        .any(|entry| entry.receipt == barrier.receipt));
    barriers.push(Arc::clone(&barrier));
    WaitGuardRegistration(barrier)
}

fn wait_guard_barrier(receipt: &CleanupReceipt) -> Option<Arc<WaitGuardBarrier>> {
    WAIT_GUARD_BARRIERS
        .lock()
        .expect("wait guard barriers")
        .iter()
        .find(|entry| entry.receipt == receipt as *const CleanupReceipt as usize)
        .cloned()
}

/// 此 hook 只在真实初始 guard 已 drop 后调用；放行后仍走原 Acquire 观察循环。
pub(super) fn wait_guard_released(receipt: &CleanupReceipt, deadline: Instant) {
    if let Some(barrier) = wait_guard_barrier(receipt) {
        barrier
            .released
            .send((receipt as *const CleanupReceipt as usize, deadline))
            .expect("observe actual guard release");
        barrier
            .release_waiter
            .lock()
            .expect("waiter barrier")
            .recv_timeout(Duration::from_secs(5))
            .expect("bounded test scheduling before atomic wait");
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// 原公开调用的完整 typed primary 摘要；工厂与清理再次观察时必须保持相同。
struct Primary {
    /// 已分配但失败调用尚未发布的句柄，不以分配本身声称槽位可查。
    allocated_handle: u64,
    diagnostic: String,
    step: &'static str,
    read_path: PathBuf,
    read_source: String,
    read_source_debug: String,
    read_os_error: Option<i32>,
    read_kind: std::io::ErrorKind,
}

#[derive(Default)]
/// 调度事实与屏障标志；全部由 Schedule.state 锁保护，真实 engine 状态另外读取。
struct State {
    /// 原 Arc 地址仅用于身份比较，不将整数转换回可解引用指针。
    owner: usize,
    receipt: usize,
    /// 同一真实 owner worker 与启动清理使用的原 deadline。
    worker: Option<ThreadId>,
    deadline: Option<Instant>,
    primary: Option<Primary>,
    primary_records: usize,
    /// 实际命令准入与回调执行次数，防止用重新调用获得假的完成证据。
    admitted: usize,
    callback_entries: usize,
    callback_completions: usize,
    callback_completed: bool,
    /// 发布者在真实 wait Mutex 内、成功 store 前到达屏障。
    publisher_locked: bool,
    would_block: usize,
    would_block_state: Option<u8>,
    release_publisher: bool,
    /// success、terminal、TLS 均分别读取或观察，不相互代替。
    published_state: Option<u8>,
    terminal: bool,
    tls_entered: bool,
    release_tls: bool,
    tls_completed: bool,
    tls_worker: Option<ThreadId>,
    /// startup 清理实际返回的回执与原 Core 关闭报告摘要。
    owner_receipt: Option<OwnerShutdownReceipt>,
    core: Option<serde_json::Value>,
}

/// 一次父进程拥有的公开失败调用的共享调度；路径只供本次有界见证输出使用。
pub(super) struct Schedule {
    /// 与父进程 fixture/精确子进程对应的 nonce，随所有见证返回。
    nonce: String,
    /// 已规范化的原 Play 路径，核对 typed error 确实来自同一项目。
    play_path: PathBuf,
    /// 父进程交付的独占输出位置；owner_returned 只 create_new，不创建目录。
    witness: PathBuf,
    state: Mutex<State>,
    changed: Condvar,
}

/// 控制线程、调用线程与 worker 共享同一调度身份。
pub(super) type Publication = Arc<Schedule>;

thread_local! {
    // CURRENT 只在一次调度的调用/worker 作用域内有效；EXIT_LATCH 留到真实 TLS 析构。
    static CURRENT: RefCell<Option<Publication>> = const { RefCell::new(None) };
    static EXIT_LATCH: RefCell<Option<WorkerExit>> = const { RefCell::new(None) };
}

/// 退出作用域时恢复原 TLS 调度，不从 EXIT_LATCH 取出真实线程退出屏障。
pub(super) struct RestoreCurrent(Option<Publication>);

impl Drop for RestoreCurrent {
    fn drop(&mut self) {
        CURRENT.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

/// 使用已附着的原 engine deadline 等待 ready；label 只提供失败诊断，不另起预算。
fn wait_engine(schedule: &Publication, label: &str, ready: impl Fn(&State) -> bool) {
    let mut state = schedule.state.lock().expect("publication schedule");
    while !ready(&state) {
        let deadline = state.deadline.expect("original engine deadline");
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "original engine deadline: {label}");
        let (next, timeout) = schedule
            .changed
            .wait_timeout(state, remaining)
            .expect("publication schedule wait");
        state = next;
        assert!(
            !timeout.timed_out() || ready(&state),
            "engine wait: {label}"
        );
    }
}

// 所有 worker hook 都核对真实 ThreadId；仅有 owner 地址不足以证明线程归属。
fn assert_worker(state: &State) {
    assert_eq!(state.worker, Some(thread::current().id()));
}

/// 读取本线程当前调度；无调度时正常 engine hook 不产生测试动作。
pub(super) fn current() -> Option<Publication> {
    CURRENT.with(|slot| slot.borrow().clone())
}

/// 在 SessionOwner::create 中附着一次非零地址身份；不允许同一失败尝试绑定第二个 owner。
pub(super) fn bind_owner(identity: usize) -> Option<Publication> {
    let schedule = current()?;
    let mut state = schedule.state.lock().expect("publication schedule");
    assert_eq!(
        state.owner, 0,
        "one actual SessionOwner per failing attempt"
    );
    assert_ne!(identity, 0);
    state.owner = identity;
    drop(state);
    Some(schedule)
}

/// 把创建线程捕获的调度移交给真实 worker TLS，返回作用域恢复 guard。
/// 同时把 WorkerExit 留在 TLS，只有真实线程退出才触发其析构屏障。
pub(super) fn worker_enter(schedule: &Option<Publication>) -> Option<RestoreCurrent> {
    let schedule = schedule.as_ref()?;
    let worker = thread::current().id();
    let previous = CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(
            slot.is_none(),
            "this worker has no other publication schedule"
        );
        slot.replace(Arc::clone(schedule))
    });
    schedule.state.lock().expect("publication schedule").worker = Some(worker);
    EXIT_LATCH.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none());
        *slot = Some(WorkerExit {
            schedule: Arc::clone(schedule),
            worker,
        });
    });
    Some(RestoreCurrent(previous))
}

/// Windows worker 的实际 TLS 退出观察；不是 owner 任务体的退出 guard。
struct WorkerExit {
    schedule: Publication,
    worker: ThreadId,
}

impl Drop for WorkerExit {
    fn drop(&mut self) {
        // This value is never extracted: only the actual worker's Windows TLS exit drops it.
        // 先记录 TLS 进入，再由控制线程放行；完成标志仅在同一 worker 的析构末尾写入。
        {
            let mut state = self.schedule.state.lock().expect("publication schedule");
            assert!(
                state.terminal,
                "actual owner task terminal precedes worker TLS exit"
            );
            assert_eq!(state.worker, Some(self.worker));
            state.tls_worker = Some(self.worker);
            state.tls_entered = true;
            self.schedule.changed.notify_all();
        }
        wait_engine(&self.schedule, "same worker TLS release", |state| {
            state.release_tls
        });
        self.schedule
            .state
            .lock()
            .expect("publication schedule")
            .tls_completed = true;
        self.schedule.changed.notify_all();
    }
}

/// 记录 join_failed_startup 生成的唯一原 deadline，唤醒尚等待 budget 附着的控制线程。
pub(super) fn startup_deadline(deadline: Instant) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert!(state.deadline.replace(deadline).is_none());
        schedule.changed.notify_all();
    }
}

/// 清理准入前绑定真实 receipt 地址，并核对它属于原 owner 与同一 deadline。
pub(super) fn prepare_receipt(receipt: &CleanupReceipt, owner: usize, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.owner, owner);
        assert_eq!(state.deadline, Some(deadline));
        assert_eq!(state.receipt, 0);
        state.receipt = receipt as *const CleanupReceipt as usize;
    }
}

/// 仅在实际 Shutdown 命令入队后计数；同一公开失败调用必须恰好准入一次。
pub(super) fn admitted(receipt: &CleanupReceipt) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
        state.admitted += 1;
        assert_eq!(state.admitted, 1);
        schedule.changed.notify_all();
    }
}

/// 记录原 cleanup 在真实 worker 的唯一进入，不创建替代清理动作。
pub(super) fn callback_entered(deadline: Instant) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert_eq!(state.deadline, Some(deadline));
        state.callback_entries += 1;
        assert_eq!(state.callback_entries, 1);
    }
}

/// 保存 shutdown_owner_state_until 的真实返回值，供发布前屏障核对成功条件。
pub(super) fn callback_completed(deadline: Instant, completed: bool) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert_eq!(state.deadline, Some(deadline));
        state.callback_completions += 1;
        assert_eq!(state.callback_completions, 1);
        state.callback_completed = completed;
    }
}

/// publish 持有真实回执 wait Mutex、尚未 store 时暂停，安排等待方的实际锁竞争。
/// 不替换 value 或写 receipt.state；原发布者放行后继续原 publish 实现。
pub(super) fn publisher_locked(receipt: &CleanupReceipt, value: u8) {
    if let Some(barrier) = wait_guard_barrier(receipt) {
        assert_eq!(
            receipt.state.load(std::sync::atomic::Ordering::Acquire),
            RECEIPT_PENDING
        );
        barrier
            .publisher_locked
            .send(())
            .expect("actual publisher lock");
        barrier
            .release_publisher
            .lock()
            .expect("publisher barrier")
            .recv_timeout(Duration::from_secs(5))
            .expect("bounded original publisher release");
    }
    if let Some(schedule) = receipt.publication.as_ref() {
        {
            let mut state = schedule.state.lock().expect("publication schedule");
            assert_worker(&state);
            assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
            assert_eq!(
                value, RECEIPT_SUCCEEDED,
                "this real callback completed successfully"
            );
            assert_eq!(state.callback_completions, 1);
            assert!(state.callback_completed);
            state.publisher_locked = true;
            schedule.changed.notify_all();
        }
        // The caller still holds the actual CleanupReceipt::wait Mutex, before its store.
        // 本屏障锁的是调度状态；原 CleanupReceipt.wait guard 始终由上层 publish 持有。
        wait_engine(schedule, "release original receipt publisher", |state| {
            state.release_publisher
        });
    }
}

/// 原 wait_until 尝试 try_lock 前，等待实际发布者进入持锁屏障。
pub(super) fn before_wait(receipt: &CleanupReceipt, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        assert_eq!(
            schedule
                .state
                .lock()
                .expect("publication schedule")
                .deadline,
            Some(deadline)
        );
        wait_engine(
            schedule,
            "actual publisher owns the receipt Mutex",
            |state| state.publisher_locked,
        );
    }
}

/// 仅在真实 try_lock 返回 WouldBlock 后读原子 Pending 并计数，不伪造错误分支。
pub(super) fn actual_would_block(receipt: &CleanupReceipt, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.deadline, Some(deadline));
        assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
        assert!(state.publisher_locked);
        let actual = receipt.state.load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(actual, RECEIPT_PENDING, "publish has not stored success");
        state.would_block += 1;
        state.would_block_state = Some(actual);
        schedule.changed.notify_all();
    }
}

/// 原 Release store 之后读取实际发布值，与 callback_completed 分开观察。
pub(super) fn published(receipt: &CleanupReceipt) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        state.published_state = Some(receipt.state.load(std::sync::atomic::Ordering::Acquire));
        schedule.changed.notify_all();
    }
}

/// owner 任务实际发布 terminal 后记录，不能据此声称 worker TLS 或池 join 已完成。
pub(super) fn terminal_published(schedule: &Option<Publication>, terminal: bool) {
    if let Some(schedule) = schedule {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert!(terminal);
        state.terminal = terminal;
        schedule.changed.notify_all();
    }
}

/// 在原 worker 捕获真实 ReadPlayScene/InvalidData 错误，重复观察须保留全部 primary 字段。
/// 路径与原项目的 canonical Play 路径核对；allocated_handle 仅是分配身份。
pub(super) fn record_primary(primary: &RuntimeDynamicSessionError, allocated_handle: u64) {
    let Some(schedule) = current() else {
        return;
    };
    let RuntimeDynamicSessionError::ProjectStep {
        step: "load Play scene override",
        source: RuntimeProjectError::ReadPlayScene { path, source },
    } = primary
    else {
        panic!("expected the original typed ordinary ReadPlayScene error: {primary:?}");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        std::fs::canonicalize(path).expect("actual read path"),
        schedule.play_path
    );
    let captured = Primary {
        allocated_handle,
        diagnostic: primary.to_string(),
        step: "load Play scene override",
        read_path: path.clone(),
        read_source: source.to_string(),
        read_source_debug: format!("{source:?}"),
        read_os_error: source.raw_os_error(),
        read_kind: source.kind(),
    };
    let mut state = schedule.state.lock().expect("publication schedule");
    assert_worker(&state);
    if let Some(original) = state.primary.as_ref() {
        assert_eq!(
            original, &captured,
            "cleanup must preserve the first complete typed primary"
        );
    } else {
        state.primary = Some(captured);
    }
    state.primary_records += 1;
    assert!(state.primary_records <= 2, "factory and one cleanup only");
}

/// 汇总已记录事实；公开 create 返回前不查 Store，返回后才观察失败句柄是否发布。
fn snapshot(schedule: &Publication, public_call_returned: bool) -> serde_json::Value {
    let state = schedule.state.lock().expect("publication schedule");
    let primary = state.primary.as_ref().expect("first typed primary");
    serde_json::json!({
        "nonce": schedule.nonce, "owner": state.owner, "receipt": state.receipt,
        "worker": format!("{:?}", state.worker.expect("actual worker")),
        "primary": {"allocated_handle": primary.allocated_handle, "diagnostic": primary.diagnostic,
            "variant": "ProjectStep/ReadPlayScene", "step": primary.step,
            "read_path": primary.read_path, "read_source": primary.read_source,
            "read_source_debug": primary.read_source_debug, "read_os_error": primary.read_os_error,
            "read_kind": format!("{:?}", primary.read_kind), "records": state.primary_records},
        "same_original_deadline": state.deadline.is_some(), "admitted": state.admitted,
        "callback_entries": state.callback_entries, "callback_completions": state.callback_completions,
        "callback_completed": state.callback_completed, "publisher_locked": state.publisher_locked,
        "actual_would_block": state.would_block, "would_block_state": state.would_block_state,
        "published_state": state.published_state, "actual_terminal": state.terminal,
        "tls_entered": state.tls_entered, "tls_completed": state.tls_completed,
        "tls_same_worker": state.tls_worker == state.worker && state.tls_worker.is_some(),
        "owner_receipt": format!("{:?}", state.owner_receipt.expect("actual shutdown return")),
        "allocated_handle_published": if public_call_returned { Some(crate::dynamic_api::session::registry::session_handle_is_published_for_test(ZrRuntimeSessionHandle::new(primary.allocated_handle))) } else { None },
        "actual_core": state.core,
    })
}

/// 由 startup_observation 在真实 shutdown 返回后记录回执和原 Core 报告。
/// 此时公共 create 仍在 Store 回调中；只生成有界、独占且持久化的 pre-fatal/Joined 见证。
pub(super) fn owner_returned(
    receipt: OwnerShutdownReceipt,
    observation: &RuntimeStartupObservation,
) {
    let Some(schedule) = current() else {
        return;
    };
    {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.primary_records, 2);
        assert_eq!(
            state.primary.as_ref().expect("first primary").diagnostic,
            observation
                .primary_diagnostic
                .as_ref()
                .expect("original diagnostic")
                .as_str()
        );
        state.owner_receipt = Some(receipt);
        state.core = Some(serde_json::json!({
            "identity": observation.core_identity, "registered_modules": observation.registered_module_count,
            "all_modules_unloaded": observation.all_modules_unloaded, "pending": observation.core_pending,
            "cleanup_completed": observation.owner_cleanup_completed,
            "report_present": observation.core_shutdown_report.is_some(),
            "all_owned_graph_workers_joined": observation.core_shutdown_report.as_ref().is_some_and(|report| !report.has_in_flight_work() && report.timer_joined && report.scopes.iter().all(|scope| scope.is_quiescent()) && report.worker_shutdowns.len() == 3 && report.worker_shutdowns.iter().all(|workers| workers.expected_worker_count > 0 && workers.all_joined())),
        }));
    }
    // Never reenter the Store from its in-progress create callback. No slot result is claimed here.
    // 先落盘真实回执，现有 fatal 分支可能立即 abort；父进程按 nonce 与内容辨认该失败。
    let raw = serde_json::to_vec(&snapshot(&schedule, false)).expect("bounded witness encoding");
    assert!(
        raw.len() <= MAX_WITNESS_BYTES,
        "bounded exact-child witness"
    );
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&schedule.witness)
        .expect("exclusive parent-owned witness");
    file.write_all(&raw)
        .expect("write exact pre-fatal/Joined witness");
    file.flush().expect("flush witness");
    file.sync_all()
        .expect("durable witness before existing fatal branch");
}

/// 先用父进程 watchdog 等 engine deadline 附着，再沿原 deadline 放行发布与同 worker TLS。
/// 只改变测试屏障标志，真实回执和 terminal 仍由原 engine 代码发布。
fn control(schedule: Publication, outer_deadline: Instant) {
    {
        let mut state = schedule.state.lock().expect("publication schedule");
        while state.deadline.is_none() {
            let remaining = outer_deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "outer child watchdog before engine shutdown starts"
            );
            let (next, timeout) = schedule
                .changed
                .wait_timeout(state, remaining)
                .expect("deadline attachment");
            state = next;
            assert!(!timeout.timed_out() || state.deadline.is_some());
        }
    }
    wait_engine(&schedule, "real Mutex contention branch", |state| {
        state.would_block > 0
    });
    schedule
        .state
        .lock()
        .expect("publication schedule")
        .release_publisher = true;
    schedule.changed.notify_all();
    wait_engine(&schedule, "actual worker TLS entry", |state| {
        state.tls_entered
    });
    schedule
        .state
        .lock()
        .expect("publication schedule")
        .release_tls = true;
    schedule.changed.notify_all();
    wait_engine(&schedule, "actual worker TLS completion", |state| {
        state.tls_completed
    });
}

/// 给一次真实公开失败调用附着调度，正常返回后 join 控制线程并生成完整观察。
///
/// # 参数与生命周期
/// nonce、play_path、witness 来自父进程拥有的同一 fixture；action 执行实际公开调用。
/// outer_deadline 只供父级 watchdog 前置等待，不延长 engine 原清理预算。
/// 调度作用域恢复 CURRENT；worker 的 EXIT_LATCH 仍留到真实 TLS 析构。
///
/// # 返回值
/// action 的原结果和真实事实快照；若现有 fatal 策略 abort，父进程读取先前的独占见证。
pub(in crate::dynamic_api::session) fn with_schedule_for_test<R>(
    nonce: String,
    play_path: PathBuf,
    witness: PathBuf,
    outer_deadline: Instant,
    action: impl FnOnce() -> R,
) -> (R, serde_json::Value) {
    let schedule = Arc::new(Schedule {
        nonce,
        play_path,
        witness,
        state: Mutex::new(State::default()),
        changed: Condvar::new(),
    });
    let previous = CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "one exact public failing call per schedule");
        slot.replace(Arc::clone(&schedule))
    });
    let restore = RestoreCurrent(previous);
    let for_control = Arc::clone(&schedule);
    let controller = thread::spawn(move || control(for_control, outer_deadline));
    let result = action();
    controller.join().expect("original-deadline controller");
    drop(restore);
    (result, snapshot(&schedule, true))
}
