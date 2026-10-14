//! 在一个长期存活的托管任务中拥有会话状态，串行接收动作和清理命令。
//!
//! `S` 由 owner 线程创建、访问和销毁；跨线程传递的是 Send 回调与结果，
//! 因而状态本身不要求 Send。借用回调另用完成信号保持调用栈有效期。
//! 清理回执、任务 terminal 和池 worker 的真实 join 是不同阶段；
//! 控制器须保留到 join 完成，Runtime 的具体资源顺序由 [`runtime::RuntimeSessionOwner`] 负责。

use std::any::Any;
use std::cell::RefCell;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex, TryLockError};
use std::time::Instant;

use crossbeam_channel::{bounded, Receiver, SendTimeoutError, Sender};

#[path = "session_owner/abi_status.rs"]
pub(super) mod abi_status;
#[path = "session_owner/runtime.rs"]
pub(super) mod runtime;

#[cfg(all(test, windows))]
#[path = "tests/receipt_publication.rs"]
pub(in crate::dynamic_api::session) mod receipt_publication;

use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

// 有界队列限制待执行命令；owned 动作的 deadline 只限制这一步准入。
const COMMAND_CAPACITY: usize = 64;
// 回执描述一次清理回调的结果；成功值尚不能证明 worker/TLS 已退出。
const RECEIPT_PENDING: u8 = 0;
const RECEIPT_SUCCEEDED: u8 = 1;
const RECEIPT_RETRYABLE: u8 = 2;
const RECEIPT_PANICKED: u8 = 3;

thread_local! {
    // 以仍存活的 Arc 地址识别当前 owner，避免同步回调等待自身造成死锁。
    static ACTIVE_SESSION_OWNERS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// 可移动到 owner 的一次动作；生命周期参数只约束回调捕获，不让状态跨线程移动。
type OwnerAction<'a, S> = Box<dyn FnOnce(&mut S) + Send + 'a>;
/// 清理返回 true 才销毁状态；false 保留状态供新的清理尝试，panic 则要求显式恢复。
type OwnerCleanup<S> = Box<dyn FnOnce(&mut S) -> bool + Send>;

/// owner 命令的三种所有权形式；只有 Scoped 使用外部调用栈上的回调存储。
enum OwnerCommand<S> {
    Action(OwnerAction<'static, S>),
    Scoped(ScopedAction<S>),
    Shutdown(OwnerCleanup<S>, Arc<CleanupReceipt>),
}

/// 擦除回调类型的借用命令；命令的 Drop 是调用方可以回收栈存储的完成边界。
struct ScopedAction<S> {
    /// 指向 dispatch_scoped 的 Option 回调；命令存活期间调用方不得访问或释放它。
    callback: *mut (),
    /// 与 callback 的具体 Option 类型配对，由 new 在同一泛型实例中建立。
    invoke: unsafe fn(*mut (), &mut S),
    /// 执行或丢弃命令均发送完成信号；不以动作是否产出结果替代此信号。
    completed: Sender<()>,
}

impl<S> ScopedAction<S> {
    // 仅由 dispatch_scoped 创建；返回命令后，调用方负责等 Drop 再离开借用作用域。
    fn new<A>(callback: &mut Option<A>, completed: Sender<()>) -> Self
    where
        A: FnOnce(&mut S) + Send,
    {
        Self {
            callback: (callback as *mut Option<A>).cast::<()>(),
            invoke: invoke_scoped::<S, A>,
            completed,
        }
    }
}

// The callback pointer is only dereferenced by the owner thread, while the
// caller waits for `completed` before reclaiming the stack callback.
// SAFETY: new 要求回调为 Send，并配对其指针与调用函数；只有 owner 解引用一次。
// dispatch_scoped 等待本命令 Drop 后才回收栈回调；发送失败也先同步销毁未入队命令。
unsafe impl<S> Send for ScopedAction<S> {}

impl<S> Drop for ScopedAction<S> {
    fn drop(&mut self) {
        // panic、队列终止和正常调用都经此边界；容量为一且只有本命令发送一次。
        let _ = self.completed.send(());
    }
}

/// 在 owner 上取走一次回调；指针必须来自同一 A 的 ScopedAction::new 且栈存储仍有效。
unsafe fn invoke_scoped<S, A>(callback: *mut (), state: &mut S)
where
    A: FnOnce(&mut S) + Send,
{
    // SAFETY: run_owner 使用 new 配对的函数指针，调用方仍等待完成信号；
    // 此时 owner 独占 Option<A>，take 也阻止同一回调被再次执行。
    let callback = unsafe { &mut *callback.cast::<Option<A>>() };
    callback
        .take()
        .expect("scoped owner callback must be invoked at most once")(state);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 动作准入失败的分类；这些返回值不表示已入队动作被取消。
pub(super) enum OwnerDispatchError {
    /// 关闭已登记，或关闭登记锁当前不可取得。
    Closing,
    /// 当前线程正在同一 owner 的命令中，不能同步等待自身。
    Reentrant,
    /// 命令通道或结果路径已终止，未取得正常结果。
    Terminated,
    /// owned 动作未在准入等待预算内进入命令队列。
    AdmissionIncomplete,
}

/// 一次清理命令共享的回执；原子结果与条件变量共同协调发布和等待。
struct CleanupReceipt {
    /// Release 发布清理结果，Acquire 读取；不编码池 join 是否完成。
    state: AtomicU8,
    /// 发布与条件变量等待共用的锁；当前等待入口的 try_lock 也可能提前返回 Pending。
    wait: Mutex<()>,
    changed: Condvar,
    #[cfg(all(test, windows))]
    publication: Option<receipt_publication::Publication>,
}

impl CleanupReceipt {
    // 回执在准入前创建；测试调度附着于当前真实 owner 尝试，生产配置无此字段。
    fn new() -> Self {
        Self {
            state: AtomicU8::new(RECEIPT_PENDING),
            wait: Mutex::new(()),
            changed: Condvar::new(),
            #[cfg(all(test, windows))]
            publication: receipt_publication::current(),
        }
    }

    // 先持有等待锁再发布结果和唤醒，防止条件检查与等待切换间丢失通知。
    fn publish(&self, state: u8) {
        let _wait = self
            .wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        #[cfg(all(test, windows))]
        receipt_publication::publisher_locked(self, state);
        self.state.store(state, Ordering::Release);
        self.changed.notify_all();
        #[cfg(all(test, windows))]
        receipt_publication::published(self);
    }

    // 返回本次可观察的结果；锁竞争立即返回 Pending，并不证明 deadline 已耗尽。
    fn wait_until(&self, deadline: Instant) -> u8 {
        #[cfg(all(test, windows))]
        receipt_publication::before_wait(self, deadline);
        let wait = match self.wait.try_lock() {
            Ok(wait) => wait,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            #[cfg(all(test, windows))]
            Err(TryLockError::WouldBlock) => {
                receipt_publication::actual_would_block(self, deadline);
                return RECEIPT_PENDING;
            }
            #[cfg(not(all(test, windows)))]
            Err(TryLockError::WouldBlock) => return RECEIPT_PENDING,
        };
        let _ = self
            .changed
            .wait_timeout_while(
                wait,
                deadline.saturating_duration_since(Instant::now()),
                |_| self.state.load(Ordering::Acquire) == RECEIPT_PENDING,
            )
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.state.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 清理回调与任务池关闭的组合结果；只有 Joined/Panicked 分支已确认真实 join。
pub(in crate::dynamic_api::session) enum OwnerShutdownReceipt {
    /// 当前回执或 worker join 尚未取得完成证据，须继续保留控制器。
    Pending,
    /// 回调返回 false，状态仍在 owner 中，允许下一次新的清理尝试。
    Retryable,
    /// 清理回调展开式 panic 被捕获，正常重试不得重跑它。
    Poisoned,
    /// 任务已终止且池已 join，但没有成功清理回执，例如工厂 panic。
    Panicked,
    /// 状态已销毁、成功回执已发布且池 worker 已 join。
    Joined,
}

/// State is created, accessed and destroyed in one persistent managed task.
/// Only owned commands cross threads; S deliberately has no Send bound.
///
/// # 使用时机与生命周期
/// 工厂在单个 owner 任务内生成 S；控制器只管理队列与完成证据。
/// Pending、Retryable、Poisoned 都保留关闭责任和 code_owner，不能直接丢弃控制器。
/// Panicked 也可发生在真实 join 后；Drop 检查 joined 字段，不要求回执必须为 Joined。
///
/// # 参考
/// [`Self::dispatch_scoped`] 保持借用回调有效期；[`Self::shutdown_until`] 区分清理与 join。
pub(super) struct SessionOwner<S> {
    /// 只为本 owner 创建的单 worker 池，真实线程退出由池 census 确认。
    pool: TaskPool,
    /// owner 任务退出其展开边界后发布；早于 worker/TLS 的完整退出。
    terminal: Arc<AtomicBool>,
    /// 保持地址身份稳定直到控制器和 owner 任务均释放引用。
    identity: Arc<()>,
    sender: Sender<OwnerCommand<S>>,
    /// 保存唯一在途清理回执；等待超时不会替换或重复执行其回调。
    shutdown: Mutex<Option<Arc<CleanupReceipt>>>,
    /// 仅在池确认 join 后置位，决定 Drop 是否可以释放控制器。
    joined: AtomicBool,
    /// 清理不完整期间保留代码所有者；上层仍负责真实动态库的存活契约。
    _code_owner: Arc<dyn Any + Send + Sync>,
}

impl<S> Drop for SessionOwner<S> {
    fn drop(&mut self) {
        // 尚无真实 join 时提前释放可能遗留执行代码的线程；现有策略直接终止进程。
        if !self.joined.load(Ordering::Acquire) {
            // DLL owners must retain this controller after an incomplete receipt.
            eprintln!("fatal dynamic session owner dropped before task termination");
            std::process::abort();
        }
    }
}

impl<S: 'static> SessionOwner<S> {
    /// 创建单 worker 池并提交 owner 工厂；返回控制器不等于工厂已经执行成功。
    ///
    /// # 参数
    /// factory 的捕获为 Send + 'static，生成的 S 留在 owner 任务；code_owner 随控制器保留。
    ///
    /// # 返回值
    /// 池创建错误转为 String；工厂 panic 由任务边界捕获，随后经 terminal/join 观察。
    pub(super) fn create(
        factory: impl FnOnce() -> S + Send + 'static,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, String> {
        let pool = TaskPool::try_new(
            TaskPoolDescriptor::io()
                .with_worker_threads(1)
                .with_thread_name("zircon-session-owner"),
        )
        .map_err(|error| error.to_string())?;
        let (sender, receiver) = bounded(COMMAND_CAPACITY);
        let terminal = Arc::new(AtomicBool::new(false));
        let task_terminal = Arc::clone(&terminal);
        let identity = Arc::new(());
        let task_identity = Arc::clone(&identity);
        #[cfg(all(test, windows))]
        let publication_schedule = receipt_publication::bind_owner(Arc::as_ptr(&identity) as usize);
        pool.spawn(move || {
            let _owner_thread = OwnerThreadGuard::enter(&task_identity);
            #[cfg(all(test, windows))]
            let _publication_thread = receipt_publication::worker_enter(&publication_schedule);
            // 包括工厂/owned 动作/状态 Drop 的展开；run_owner 栈销毁后才发布 terminal。
            let _ = catch_unwind(AssertUnwindSafe(|| run_owner(factory, receiver)));
            task_terminal.store(true, Ordering::Release);
            #[cfg(all(test, windows))]
            receipt_publication::terminal_published(
                &publication_schedule,
                task_terminal.load(Ordering::Acquire),
            );
        });
        Ok(Self {
            pool,
            terminal,
            identity,
            sender,
            shutdown: Mutex::new(None),
            joined: AtomicBool::new(false),
            _code_owner: code_owner,
        })
    }

    /// Runs a borrowing callback on the owner and does not return until the callback
    /// has completed or has been dropped by a terminated owner. The erased lifetime
    /// is sound because every exit waits for the command channel to release it.
    ///
    /// # 参数与返回值
    /// action 可借用调用栈，但其捕获与 R 必须能跨线程。返回动作结果或准入/终止错误；
    /// 回调 panic 在完成信号之后回到调用线程继续展开，不让栈借用提前失效。
    ///
    /// # 约束
    /// 同一 owner 的同步重入被拒绝；本方法不为上层登记 SessionSlot 动作计数，也不设执行超时。
    pub(super) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut S) -> R + Send,
    ) -> Result<R, OwnerDispatchError> {
        if self.is_current_thread() {
            return Err(OwnerDispatchError::Reentrant);
        }
        let shutdown = self
            .shutdown
            .try_lock()
            .map_err(|_| OwnerDispatchError::Closing)?;
        if shutdown.is_some() {
            return Err(OwnerDispatchError::Closing);
        }
        let result = Arc::new(Mutex::new(None));
        let result_for_owner = Arc::clone(&result);
        let (completed_tx, completed_rx) = bounded(1);
        let mut callback = Some(move |state: &mut S| {
            let outcome = catch_unwind(AssertUnwindSafe(|| action(state)));
            *result_for_owner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(outcome);
        });
        self.sender
            .send(OwnerCommand::Scoped(ScopedAction::new(
                &mut callback,
                completed_tx,
            )))
            .map_err(|_| OwnerDispatchError::Terminated)?;
        // 命令入队后才放开关闭登记锁；接下来始终等 ScopedAction 的 Drop 完成信号。
        drop(shutdown);
        completed_rx
            .recv()
            .map_err(|_| OwnerDispatchError::Terminated)?;
        let outcome = result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        match outcome.ok_or(OwnerDispatchError::Terminated)? {
            Ok(result) => Ok(result),
            Err(payload) => resume_unwind(payload),
        }
    }

    /// Admission is bounded; the caller owns the result receiver even after timeout.
    ///
    /// # 参数与返回值
    /// deadline 只约束命令入队；action 与 R 为 Send + 'static，成功返回独立结果接收端。
    /// 准入超时返回 AdmissionIncomplete，未入队命令被丢弃；已入队动作不会被此时间取消。
    /// 接收端自行等待执行结果，回调 panic 使 owner 终止并关闭后续结果路径。
    pub(super) fn dispatch<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut S) -> R + Send + 'static,
    ) -> Result<Receiver<R>, OwnerDispatchError> {
        if self.is_current_thread() {
            return Err(OwnerDispatchError::Reentrant);
        }
        let shutdown = self
            .shutdown
            .try_lock()
            .map_err(|_| OwnerDispatchError::Closing)?;
        if shutdown.is_some() {
            return Err(OwnerDispatchError::Closing);
        }
        let (result_tx, result_rx) = bounded(1);
        self.sender
            .send_timeout(
                OwnerCommand::Action(Box::new(move |state| {
                    let result = action(state);
                    let _ = result_tx.send(result);
                })),
                deadline.saturating_duration_since(Instant::now()),
            )
            .map_err(|error| match error {
                SendTimeoutError::Timeout(_) => OwnerDispatchError::AdmissionIncomplete,
                SendTimeoutError::Disconnected(_) => OwnerDispatchError::Terminated,
            })?;
        Ok(result_rx)
    }

    /// A pending cleanup is never invoked twice, including across expired waits.
    ///
    /// # 参数与返回值
    /// cleanup 在 owner 上执行；true 允许销毁 S，false 保留 S，panic 形成 Poisoned。
    /// deadline 供本次回执等待和池关闭使用；未完成返回后仍须保留控制器与已有回执。
    pub(super) fn shutdown_until(
        &self,
        deadline: Instant,
        cleanup: impl FnOnce(&mut S) -> bool + Send + 'static,
    ) -> OwnerShutdownReceipt {
        self.shutdown_until_inner(deadline, cleanup, false)
    }

    /// A cleanup panic poisons normal shutdown. Only an explicit recovery callback
    /// may repair that state; a normal retry never reinvokes the panicked callback.
    ///
    /// # 约束与返回值
    /// 仅已发布 RECEIPT_PANICKED 的回执可获准恢复；锁忙或其他状态返回静态错误。
    /// recovery 作为新的拥有型清理命令提交，仍须等待成功回执和池 join。
    pub(super) fn recover_poisoned_until(
        &self,
        deadline: Instant,
        recovery: impl FnOnce(&mut S) -> bool + Send + 'static,
    ) -> Result<OwnerShutdownReceipt, &'static str> {
        {
            let shutdown = self.shutdown.try_lock().map_err(|_| "owner busy")?;
            if !shutdown
                .as_ref()
                .is_some_and(|receipt| receipt.state.load(Ordering::Acquire) == RECEIPT_PANICKED)
            {
                return Err("owner is not poisoned");
            }
        }
        Ok(self.shutdown_until_inner(deadline, recovery, true))
    }

    // 共享在途回执；仅 Retryable 或显式 Poisoned 恢复会准入新的回调。
    fn shutdown_until_inner(
        &self,
        deadline: Instant,
        cleanup: impl FnOnce(&mut S) -> bool + Send + 'static,
        recover_poisoned: bool,
    ) -> OwnerShutdownReceipt {
        if self.task_is_terminal() {
            // terminal 只表明任务体退出；即使返回 Panicked，也先确认池 worker 真正 join。
            if !self.pool.shutdown_until(deadline) {
                return OwnerShutdownReceipt::Pending;
            }
            self.joined.store(true, Ordering::Release);
            return match self.shutdown.try_lock() {
                Ok(shutdown)
                    if shutdown.as_ref().is_some_and(|receipt| {
                        receipt.state.load(Ordering::Acquire) == RECEIPT_SUCCEEDED
                    }) =>
                {
                    OwnerShutdownReceipt::Joined
                }
                _ => OwnerShutdownReceipt::Panicked,
            };
        }
        let receipt = {
            let mut shutdown = match self.shutdown.try_lock() {
                Ok(shutdown) => shutdown,
                Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
                Err(TryLockError::WouldBlock) => return OwnerShutdownReceipt::Pending,
            };
            let retry = match shutdown
                .as_ref()
                .map(|receipt| receipt.state.load(Ordering::Acquire))
            {
                None | Some(RECEIPT_RETRYABLE) => !recover_poisoned,
                Some(RECEIPT_PANICKED) => recover_poisoned,
                _ => false,
            };
            if retry {
                let receipt = Arc::new(CleanupReceipt::new());
                #[cfg(all(test, windows))]
                receipt_publication::prepare_receipt(
                    &receipt,
                    Arc::as_ptr(&self.identity) as usize,
                    deadline,
                );
                if self
                    .sender
                    .try_send(OwnerCommand::Shutdown(
                        Box::new(cleanup),
                        Arc::clone(&receipt),
                    ))
                    .is_err()
                {
                    // 未准入的清理未执行；保留 Retryable 回执供下次尝试，而本次报告 Pending。
                    receipt.publish(RECEIPT_RETRYABLE);
                    *shutdown = Some(receipt);
                    return OwnerShutdownReceipt::Pending;
                }
                *shutdown = Some(receipt);
                #[cfg(all(test, windows))]
                receipt_publication::admitted(shutdown.as_ref().expect("actual admitted receipt"));
            }
            Arc::clone(shutdown.as_ref().expect("admitted shutdown receipt"))
        };
        match receipt.wait_until(deadline) {
            RECEIPT_RETRYABLE => return OwnerShutdownReceipt::Retryable,
            RECEIPT_PANICKED => return OwnerShutdownReceipt::Poisoned,
            RECEIPT_SUCCEEDED => {
                // S 已经销毁；仍可能阻塞在 worker 的 TLS 析构，必须继续保留控制器。
                return if self.pool.shutdown_until(deadline) {
                    self.joined.store(true, Ordering::Release);
                    OwnerShutdownReceipt::Joined
                } else {
                    OwnerShutdownReceipt::Pending
                };
            }
            _ => return OwnerShutdownReceipt::Pending,
        }
    }

    /// 无等待读取任务体的 terminal 标志；true 不代表池 worker/TLS 已 join。
    pub(super) fn task_is_terminal(&self) -> bool {
        self.terminal.load(Ordering::Acquire)
    }

    // 同线程同身份才拒绝重入；其他 owner 的调用不由此处一概禁止。
    fn is_current_thread(&self) -> bool {
        let identity = Arc::as_ptr(&self.identity) as usize;
        ACTIVE_SESSION_OWNERS.with(|owners| owners.borrow().contains(&identity))
    }

    /// 只观察保留回执，不执行清理或 join；None 表示尚无请求，锁忙按 Pending 处理。
    /// 成功回执只有在 joined 已置位时映射为 Joined；不把 terminal 自动映射为 Panicked。
    pub(super) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        let shutdown = match self.shutdown.try_lock() {
            Ok(shutdown) => shutdown,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Some(OwnerShutdownReceipt::Pending),
        };
        shutdown
            .as_ref()
            .map(|receipt| match receipt.state.load(Ordering::Acquire) {
                RECEIPT_RETRYABLE => OwnerShutdownReceipt::Retryable,
                RECEIPT_PANICKED => OwnerShutdownReceipt::Poisoned,
                RECEIPT_SUCCEEDED if self.joined.load(Ordering::Acquire) => {
                    OwnerShutdownReceipt::Joined
                }
                _ => OwnerShutdownReceipt::Pending,
            })
    }
}

/// 在 owner 任务作用域登记当前线程身份；退出任务时按后进先出顺序撤销。
struct OwnerThreadGuard {
    identity: usize,
}

impl OwnerThreadGuard {
    // identity 的 Arc 由任务捕获保留，整数地址只用于比较，不反向解引用。
    fn enter(identity: &Arc<()>) -> Self {
        let identity = Arc::as_ptr(identity) as usize;
        ACTIVE_SESSION_OWNERS.with(|owners| owners.borrow_mut().push(identity));
        Self { identity }
    }
}

impl Drop for OwnerThreadGuard {
    fn drop(&mut self) {
        ACTIVE_SESSION_OWNERS.with(|owners| {
            let popped = owners.borrow_mut().pop();
            debug_assert_eq!(popped, Some(self.identity));
        });
    }
}

/// 在同一任务栈创建并独占 S；成功清理先销毁状态，再发布成功回执并退出接收循环。
fn run_owner<S>(factory: impl FnOnce() -> S, receiver: Receiver<OwnerCommand<S>>) {
    let mut state = factory();
    while let Ok(command) = receiver.recv() {
        match command {
            OwnerCommand::Action(action) => {
                // The outer owner boundary reports terminal panic after state is
                // destroyed here; no further command may access unwound state.
                // owned 动作的 panic 不在此分支恢复，避免后续命令继续访问展开中的状态。
                action(&mut state);
            }
            OwnerCommand::Scoped(scoped) => {
                // SAFETY: new 配对回调指针和具体 invoke，发送方等待 scoped 的 Drop；
                // 本循环唯一访问 S，命令只消费一次，回调栈存储在本次调用期间仍有效。
                unsafe { (scoped.invoke)(scoped.callback, &mut state) };
            }
            OwnerCommand::Shutdown(cleanup, receipt) => {
                match catch_unwind(AssertUnwindSafe(|| cleanup(&mut state))) {
                    Ok(true) => {
                        // 状态 Drop 本身若 panic，由外层任务边界处理，不发布虚假的成功回执。
                        drop(state);
                        receipt.publish(RECEIPT_SUCCEEDED);
                        return;
                    }
                    Ok(false) => receipt.publish(RECEIPT_RETRYABLE),
                    Err(_) => receipt.publish(RECEIPT_PANICKED),
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "session_owner/tests/cases.rs"]
mod tests;
