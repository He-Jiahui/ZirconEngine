//! 以进程级编号发布 owner 槽位，统一会话动作的准入、结果 finalization 与关闭重试。
//!
//! 查得槽位只是取得共享引用；普通动作仍须在 Open 状态登记 guard 后才能访问 owner。
//! owner 命令与调用线程上的 finalizer 分开执行，guard 的范围由各包装器确定。
//! 销毁沿同一个截止时间排空动作和 wake，检查输出 census、join owner 后才删除句柄。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError};
use std::time::{Duration, Instant};

use thiserror::Error;
use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrStatus};

use super::action_guard::SessionActionGuard;
use super::allocation_registry::{
    session_has_outstanding_allocations_until, try_lock_registry_for_destroy,
};
use super::session_owner::runtime::RuntimeSessionOwnerCreateError;
use super::session_slot::SessionSlot;
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::diagnostic_log::write_log;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::status::{invalid_argument, not_found, teardown_incomplete};
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();
/// 一次销毁尝试共享的等待预算，涵盖查找、排空、owner 关闭与最终删除。
const DYNAMIC_SESSION_ACTION_WAKE_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
/// owned 命令进入 owner 队列的等待预算，不限制命令执行或结果接收的时间。
const DYNAMIC_SESSION_OWNED_COMMAND_ADMISSION_TIMEOUT: Duration = Duration::from_secs(5);

/// 句柄的唯一发布位置；移出记录前必须完成槽位关闭，编号不会循环复用。
struct SessionRegistry {
    /// 零表示永久耗尽；成功分配最大编号后下一次请求失败。
    next_handle: u64,
    sessions: HashMap<u64, Arc<SessionSlot>>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// 已链接入口与测试插入入口的构造错误；生产 ABI 创建路径另转换为 `ZrStatus`。
pub(in crate::dynamic_api::session) enum SessionRegistryInsertError {
    /// 不再有可发布的非零编号，不撤回已经消耗的编号。
    #[error("runtime session handle space exhausted")]
    HandleSpaceExhausted,
    /// owner 构造或启动会话失败后返回的拥有所有权的诊断。
    #[error("runtime session owner unavailable: {0}")]
    OwnerUnavailable(String),
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self {
            next_handle: 1,
            sessions: HashMap::new(),
        }
    }
}

impl SessionRegistry {
    /// 消耗一个尚未使用的非零编号，不在此发布槽位。
    ///
    /// # 返回值
    /// 可分配到最大编号；随后将计数置零并永久返回耗尽错误，不回绕到旧句柄。
    ///
    /// # 参考
    /// `tests/store_handle_alloc.rs` 的最大值与零值测试。
    fn try_allocate_handle(&mut self) -> Result<u64, SessionRegistryInsertError> {
        let handle = self.next_handle;
        if handle == 0 {
            return Err(SessionRegistryInsertError::HandleSpaceExhausted);
        }
        debug_assert!(!self.sessions.contains_key(&handle));
        // Zero is invalid and becomes the permanent exhausted state after u64::MAX.
        self.next_handle = handle.checked_add(1).unwrap_or(0);
        Ok(handle)
    }
}

fn registry() -> &'static Mutex<SessionRegistry> {
    SESSION_REGISTRY.get_or_init(|| Mutex::new(SessionRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, SessionRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 在销毁截止时间内尝试取得注册表锁；竞争时让出线程，None 不表示句柄不存在。
fn lock_registry_until(deadline: Instant) -> Option<MutexGuard<'static, SessionRegistry>> {
    loop {
        if Instant::now() >= deadline {
            return None;
        }
        let guard = match registry().try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                std::thread::yield_now();
                continue;
            }
        };
        return (Instant::now() < deadline).then_some(guard);
    }
}

#[cfg(test)]
/// 让截止时间测试在另一个线程有意持锁；回调完成前不释放注册表锁。
pub(super) fn with_registry_lock_for_test(action: impl FnOnce()) {
    let _registry = lock_registry();
    action();
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn session_handle_is_published_for_test(
    handle: ZrRuntimeSessionHandle,
) -> bool {
    lock_registry().sessions.contains_key(&handle.raw())
}

#[cfg(test)]
pub(super) fn session_slot_for_test(handle: ZrRuntimeSessionHandle) -> Option<Arc<SessionSlot>> {
    find_session_slot(handle).ok()
}

#[cfg(test)]
/// 仅为锁恢复测试制造 panic；调用者负责捕获，后续取得锁仍沿 poisoned.into_inner 恢复。
pub(in crate::dynamic_api::session) fn poison_registry_lock_for_test() {
    let _registry = lock_registry();
    panic!("poison dynamic API session registry lock");
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn try_insert_session(
    session: RuntimeDynamicSession,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    try_insert_session_with_wake(session, RuntimeWakeRegistration::disabled())
}

#[cfg(test)]
/// 将已构造的测试会话交给 owner 后发布；此测试路径在 owner 构造期间保留注册表锁。
/// 编号耗尽时先释放表锁再关闭尚未发布的会话；无法完成清理则沿既有策略 abort。
pub(in crate::dynamic_api::session) fn try_insert_session_with_wake(
    mut session: RuntimeDynamicSession,
    wake: RuntimeWakeRegistration,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    let mut registry = lock_registry();
    let handle = match registry.try_allocate_handle() {
        Ok(handle) => handle,
        Err(error) => {
            drop(registry);
            if !session.shutdown_before_library_unload() {
                eprintln!(
                    "fatal dynamic runtime session handle allocation teardown failure; aborting before dynamic library unload"
                );
                std::process::abort();
            }
            return Err(error);
        }
    };
    let slot = SessionSlot::new(session, wake)
        .map(Arc::new)
        .map_err(SessionRegistryInsertError::OwnerUnavailable)?;
    registry.sessions.insert(handle, slot);
    Ok(ZrRuntimeSessionHandle::new(handle))
}

/// 为已链接的插件集合创建 owner，会话启动成功之后才发布此前分配的编号。
///
/// # 参数
/// `profile/project` 描述待构造的会话；`registrations` 移入 owner 工厂，外部 wake 在此路径禁用。
///
/// # 返回值
/// 已发布句柄或构造错误；失败消耗的编号不复用，owner/session 诊断由调用入口统一映射。
///
/// # 参考
/// `session/linked_session.rs::create_linked_runtime_session`；[`SessionSlot::create_with_linked_plugins`]。
pub(in crate::dynamic_api::session) fn try_create_linked_session(
    profile: RuntimeDynamicSessionProfile,
    project: Option<RuntimeProjectConfig>,
    registrations: Vec<RuntimePluginRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    let handle = lock_registry().try_allocate_handle()?;
    #[cfg(test)]
    super::session_owner::runtime::startup_observation::record_allocated_handle(handle);
    let slot = SessionSlot::create_with_linked_plugins(profile, project, registrations).map_err(
        |error| match error {
            RuntimeSessionOwnerCreateError::Owner(message) => {
                SessionRegistryInsertError::OwnerUnavailable(message)
            }
            RuntimeSessionOwnerCreateError::Session(status) => {
                SessionRegistryInsertError::OwnerUnavailable(status.into_message())
            }
        },
    )?;
    lock_registry().sessions.insert(handle, Arc::new(slot));
    Ok(ZrRuntimeSessionHandle::new(handle))
}

/// 为动态 ABI 创建 owner 并发布句柄，工厂执行期间不持有全局注册表锁。
///
/// # 参数
/// `profile/project/wake` 由创建入口验证后交给 owner；wake 的回调代码和 token 存活期由 ABI 调用方保证。
///
/// # 返回值
/// 仅启动成功的槽位进入表；编号耗尽返回 LimitExceeded，其他启动错误由槽位转换为 ABI status。
/// 失败不会把已消耗的编号再次分配给下一会话。
///
/// # 参考
/// `session/ffi.rs::create_session`；[`SessionSlot::create`]。
pub(in crate::dynamic_api::session) fn try_create_session_with_wake(
    profile: RuntimeDynamicSessionProfile,
    project: Option<RuntimeProjectConfig>,
    wake: RuntimeWakeRegistration,
) -> Result<ZrRuntimeSessionHandle, ZrStatus> {
    let handle = lock_registry().try_allocate_handle().map_err(|_| {
        crate::dynamic_api::session::status::limit_exceeded(
            b"runtime session handle space exhausted",
        )
    })?;
    #[cfg(test)]
    super::session_owner::runtime::startup_observation::record_allocated_handle(handle);
    let slot = Arc::new(SessionSlot::create(profile, project, wake)?);
    lock_registry().sessions.insert(handle, slot);
    Ok(ZrRuntimeSessionHandle::new(handle))
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn insert_session(
    session: RuntimeDynamicSession,
) -> ZrRuntimeSessionHandle {
    try_insert_session(session).expect("test runtime session handle")
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn insert_session_with_wake(
    session: RuntimeDynamicSession,
    wake: RuntimeWakeRegistration,
) -> ZrRuntimeSessionHandle {
    try_insert_session_with_wake(session, wake).expect("test runtime session handle")
}

/// 在已获准的动作范围内同步访问 owner 上的会话，并将诊断带回当前调用线程。
///
/// # 参数
/// `handle` 须存在且仍接受普通动作；`action` 可借用调用栈，但须能发送到 owner。
///
/// # 返回值
/// 动作的 status 或查找/准入错误；同一 owner 的同步重入转为 InvalidArgument。
///
/// # 参考
/// [`with_session_activity`]；`session/ffi.rs::bind_viewport_surface`。
pub(in crate::dynamic_api::session) fn with_session(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus + Send,
) -> ZrStatus {
    with_session_activity(handle, |session, _activity| action(session))
}

/// 分派拥有全部捕获数据的命令，并持有动作 guard 等到结果被收取。
///
/// # 参数
/// `handle` 按普通动作准入；`action` 必须满足 Send 与静态生命周期，不能借用会提前返回的栈数据。
///
/// # 返回值
/// 成功或动作诊断；同步重入为 InvalidArgument，准入超时为 Error，owner 终止等错误为 NotFound。
///
/// # 使用时机与生命周期
/// 五秒预算仅作用于进入命令队列；结果 recv 没有独立超时，等待期间 guard 仍计入销毁屏障。
/// owner 线程诊断先复制成拥有所有权的消息，再在当前线程生成 ABI status。
///
/// # 参考
/// `session/ffi.rs::handle_event` 先复制 payload，再调用此入口；[`SessionSlot::dispatch_owned`]。
pub(in crate::dynamic_api::session) fn with_session_owned(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus + Send + 'static,
) -> ZrStatus {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return status,
    };
    let Some(action_guard) = slot.begin_action() else {
        return not_found(b"runtime session not found");
    };
    let admission_deadline = Instant::now()
        .checked_add(DYNAMIC_SESSION_OWNED_COMMAND_ADMISSION_TIMEOUT)
        .unwrap_or_else(Instant::now);
    let status = match slot.dispatch_owned(admission_deadline, move |session| {
        let status = action(session);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status)
        }
    }) {
        Ok(result) => match result.recv() {
            Ok(Ok(())) => ZrStatus::ok(),
            Ok(Err(error)) => error.into_abi(),
            Err(_) => not_found(b"runtime session not found"),
        },
        Err(super::session_owner::OwnerDispatchError::Reentrant) => {
            invalid_argument(b"runtime session callback cannot reenter synchronously")
        }
        Err(super::session_owner::OwnerDispatchError::AdmissionIncomplete) => {
            crate::dynamic_api::session::status::error_status(
                "runtime session owner command admission incomplete",
            )
        }
        Err(_) => not_found(b"runtime session not found"),
    };
    drop(action_guard);
    status
}

/// 同步分派会话动作，并借用同一槽位的 frame activity；status 转换与准入规则同 [`with_session`]。
pub(in crate::dynamic_api::session) fn with_session_activity(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> ZrStatus + Send,
) -> ZrStatus {
    match with_session_activity_result(handle, |session, activity| {
        let status = action(session, activity);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status)
        }
    }) {
        Ok(()) => ZrStatus::ok(),
        Err(status) => status,
    }
}

/// 在 owner 上产生结果，再在调用线程执行 finalizer，两个阶段共用一次动作 guard。
///
/// # 参数
/// `action` 的结果 T 须能跨线程传回；`finalize` 无 Send 要求，可使用仍有效的调用方输出位置。
///
/// # 返回值
/// finalizer 的 U 或任一阶段的错误；动作失败时不执行 finalizer。
///
/// # 使用时机与生命周期
/// guard 保留到 finalizer 返回；ABI 地址的可写性和借用期限仍由具体 FFI 契约建立。
///
/// # 参考
/// `session/ffi.rs::capture_frame`、`capture_accessibility_tree` 将登记与输出写入放在 finalizer 内；
/// `session/operation.rs::submit_operation`、`poll_operation` 在同一范围内发布句柄或状态。
pub(in crate::dynamic_api::session) fn with_session_result_finalized<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    with_session_activity_result_finalized(handle, |session, _activity| action(session), finalize)
}

/// 在一次动作 guard 内完成准备、finalization 与提交；finalization 失败时请求回滚准备状态。
///
/// # 参数
/// `action` 在 owner 上准备 T；`finalize` 在调用线程将其登记或发布为 U；
/// `commit/rollback` 重新分派到同一 owner，具体可撤销状态由这些回调定义。
///
/// # 返回值
/// 成功提交后返回 U。动作错误直接退出；finalizer 错误先分派 rollback，
/// rollback 分派错误会取代原错误；commit 分派失败不会自动回滚或回收已登记的 allocation。
///
/// # 使用时机与生命周期
/// guard 覆盖 action、finalize 和 commit/rollback，在返回前释放，不包含调用者随后执行的输出写入。
/// `ffi.rs` 的 host request、plugin event 和 world invalidation drain 在返回后发布描述符；
/// 非空输出仍由 allocation census 阻止会话销毁，规范空输出没有这一 allocation 前提。
/// `operation.rs::harvest_operation` 则在 finalizer 内写入输出；不能把两种发布位置视为同一范围。
///
/// # 参考
/// `tests/cases.rs::committed_session_result_rolls_back_in_flight_state_when_finalization_fails`；
/// `session/ffi.rs::drain_host_requests`、`drain_plugin_events`、`drain_world_invalidations`。
pub(in crate::dynamic_api::session) fn with_session_result_committed<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
    commit: impl FnOnce(&mut RuntimeDynamicSession) + Send,
    rollback: impl FnOnce(&mut RuntimeDynamicSession) + Send,
) -> Result<U, ZrStatus> {
    let slot = find_session_slot(handle)?;
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let value = dispatch_session_action(&slot, action)?;
    let finalized = match finalize(handle, value) {
        Ok(finalized) => finalized,
        Err(status) => {
            dispatch_session_action(&slot, move |session| {
                rollback(session);
                Ok(())
            })?;
            return Err(status);
        }
    };
    dispatch_session_action(&slot, move |session| {
        commit(session);
        Ok(())
    })?;
    drop(action_guard);
    Ok(finalized)
}

fn with_session_activity_result<T: Send>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus> + Send,
) -> Result<T, ZrStatus> {
    with_session_activity_result_finalized(handle, action, |_active_handle, value| Ok(value))
}

/// 与 [`with_session_result_finalized`] 共用准入与 guard 范围，并向 owner 回调提供槽位的 frame activity。
/// `session/ffi.rs::tick_frame` 在 owner 上更新 demand，再由调用线程 finalizer 写出 ABI demand。
pub(in crate::dynamic_api::session) fn with_session_activity_result_finalized<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return Err(status),
    };
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let activity = slot.frame_activity();
    let value = dispatch_session_action(&slot, move |session| action(session, activity))?;
    let finalized = finalize(handle, value);
    drop(action_guard);
    finalized
}

/// 供 allocation 包装器登记普通动作；查找成功仍须通过槽位的 Open 准入检查。
pub(super) fn begin_session_action(
    handle: ZrRuntimeSessionHandle,
) -> Result<SessionActionGuard, ZrStatus> {
    let slot = find_session_slot(handle)?;
    slot.begin_action()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

/// 供输出释放登记动作；销毁失败后的待重试阶段也可准入，正在 Closing 时则返回 NotFound。
pub(super) fn begin_session_release_action(
    handle: ZrRuntimeSessionHandle,
) -> Result<SessionActionGuard, ZrStatus> {
    let slot = find_session_slot(handle)?;
    slot.begin_release_action()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

/// 按默认共享等待预算尝试关闭会话，成功后才删除句柄与空 census。
///
/// # 参数
/// `handle` 是同 provider 的已发布会话；调用方须先完成输出使用与释放，并在未完成回执期间保留 provider。
///
/// # 返回值
/// Ok 表示本次正常清理和删除已完成；Error 不是句柄已经移除的证明，应排空保留资源后重试。
/// 无效零句柄为 InvalidArgument，未知句柄为 NotFound；当前 wake 回调的同步销毁会被拒绝。
///
/// # 参考
/// [`destroy_session_slot_with_timeout`]；`tests/cases.rs` 的 quiescence、deadline 与 explicit retry 测试。
pub(in crate::dynamic_api::session) fn destroy_session_slot(
    handle: ZrRuntimeSessionHandle,
) -> ZrStatus {
    destroy_session_slot_with_timeout(handle, DYNAMIC_SESSION_ACTION_WAKE_DRAIN_TIMEOUT)
}

/// 以一次尝试的截止时间顺序查找、关闭准入、排空动作/wake、检查 census、关闭 owner、删除记录。
///
/// # 返回值
/// 已进入关闭后的失败保留待重试状态；初次查找超时尚未关闭槽位，不能一律推断 phase 已改变。
/// 只有 owner 的 Joined 回执允许最终删除，其他回执不代表可以卸载仍被使用的代码。
///
/// # 参考
/// `session/tests/destroy_registry_deadline.rs`；[`SessionSlot::begin_close`]。
pub(super) fn destroy_session_slot_with_timeout(
    handle: ZrRuntimeSessionHandle,
    timeout: Duration,
) -> ZrStatus {
    let destroy_started_at = Instant::now();
    let Some(destroy_deadline) = destroy_started_at.checked_add(timeout) else {
        return teardown_incomplete();
    };
    write_log("runtime_session", "destroy_find_slot_start");
    let slot = match find_session_slot_until(handle, destroy_deadline) {
        Ok(slot) => slot,
        Err(status) => return status,
    };
    write_log("runtime_session", "destroy_find_slot_done");
    if slot
        .frame_activity()
        .wake_callback_active_on_current_thread()
    {
        return invalid_argument(b"runtime wake callback cannot destroy its session synchronously");
    }
    write_log("runtime_session", "destroy_begin_close_start");
    if !slot.begin_close() {
        return not_found(b"runtime session not found");
    }
    write_log("runtime_session", "destroy_begin_close_done");

    slot.frame_activity().disable_wake_entries();
    write_log("runtime_session", "destroy_wait_actions_start");
    if !slot.wait_for_actions(destroy_deadline.saturating_duration_since(Instant::now())) {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_wait_actions_done");
    write_log("runtime_session", "destroy_wait_wake_callbacks_start");
    if !slot
        .frame_activity()
        .wait_for_wake_callbacks(destroy_deadline.saturating_duration_since(Instant::now()))
    {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_wait_wake_callbacks_done");
    write_log("runtime_session", "destroy_allocation_check_start");
    let Some(has_outstanding_allocations) =
        session_has_outstanding_allocations_until(handle, destroy_deadline)
    else {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    };
    if has_outstanding_allocations {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_allocation_check_done");
    write_log("runtime_session", "destroy_shutdown_start");
    let session_shutdown = matches!(
        slot.shutdown_until(destroy_deadline),
        super::session_owner::OwnerShutdownReceipt::Joined
    );
    write_log("runtime_session", "destroy_shutdown_done");

    if !session_shutdown {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }

    write_log("runtime_session", "destroy_registry_remove_start");
    let removed = remove_session_and_census_until(handle, &slot, destroy_deadline);
    if !removed {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_registry_remove_done");
    ZrStatus::ok()
}

/// 在 session 与 allocation 两把锁下删除同一个已关闭槽位及其零分配 census。
///
/// # 参数
/// `slot` 是已排空动作/wake 且 owner 已 Joined 的原槽位，`destroy_deadline` 沿用本次销毁预算。
///
/// # 返回值
/// true 表示本方删除成功，或并发方已同时移除两个记录；锁超时、槽位身份不符或仍有分配则为 false。
/// 单凭编号相同不能替代 Arc 身份校验；本函数不执行 owner 关闭。
pub(super) fn remove_session_and_census_until(
    handle: ZrRuntimeSessionHandle,
    slot: &Arc<SessionSlot>,
    destroy_deadline: Instant,
) -> bool {
    let removed = loop {
        let Some(mut registry) = lock_registry_until(destroy_deadline) else {
            break false;
        };
        // allocation 锁只尝试一次；竞争时先放下 session 锁，避免阻塞先查会话再回收输出的调用链。
        let Some(mut allocations) = try_lock_registry_for_destroy() else {
            drop(registry);
            if Instant::now() >= destroy_deadline {
                break false;
            }
            std::thread::yield_now();
            continue;
        };
        if Instant::now() >= destroy_deadline {
            break false;
        }
        match registry.sessions.get(&handle.raw()) {
            Some(registered) if Arc::ptr_eq(registered, &slot) => {}
            None if !allocations.has_census(handle) => break true,
            _ => break false,
        }
        // TODO: [CR-SESSION-0001] 并发 destroy 接受 Closing；另一失败方转 Retry 后可重新准入 release。
        // 先前 wait_actions 已归零，release 仍可能在 census 减零但 guard 尚存时被另一销毁方观察。
        // ABI 要求先归还输出，App 的 &mut 包装排除同对象双销毁；外部并发集合及在途生产者 F 的合法输入仍待核实。
        // 后续核对销毁并发前提，并用屏障验证 A/B/F/release 交错；尚无合规 ABI 复现。
        if allocations.has_outstanding_allocations(handle) || Instant::now() >= destroy_deadline {
            break false;
        }
        registry.sessions.remove(&handle.raw());
        allocations.forget_empty_census(handle);
        break true;
    };
    removed
}

/// 在 owner 返回前复制的诊断转为当前线程 status；线程重入单独映射为 InvalidArgument。
/// 本函数只分派，调用者仍负责持有槽位动作 guard。
fn dispatch_session_action<R: Send>(
    slot: &SessionSlot,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
) -> Result<R, ZrStatus> {
    let admission_deadline = Instant::now()
        .checked_add(DYNAMIC_SESSION_OWNED_COMMAND_ADMISSION_TIMEOUT)
        .unwrap_or_else(Instant::now);
    match slot.dispatch_scoped(admission_deadline, action) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.into_abi()),
        Err(super::session_owner::OwnerDispatchError::Reentrant) => Err(invalid_argument(
            b"runtime session callback cannot reenter synchronously",
        )),
        Err(super::session_owner::OwnerDispatchError::AdmissionIncomplete) => {
            Err(crate::dynamic_api::session::status::error_status(
                b"runtime session command admission timed out",
            ))
        }
        Err(_) => Err(not_found(b"runtime session not found")),
    }
}

/// 克隆已发布槽位，不登记动作；零句柄与未知非零句柄分别返回 InvalidArgument 和 NotFound。
fn find_session_slot(handle: ZrRuntimeSessionHandle) -> Result<Arc<SessionSlot>, ZrStatus> {
    if !handle.is_valid() {
        return Err(invalid_argument(b"invalid runtime session handle"));
    }
    let registry = lock_registry();
    registry
        .sessions
        .get(&handle.raw())
        .cloned()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

/// 销毁专用查找；无法在截止时间内取得表锁返回未完成，而不是伪装成 NotFound。
fn find_session_slot_until(
    handle: ZrRuntimeSessionHandle,
    deadline: Instant,
) -> Result<Arc<SessionSlot>, ZrStatus> {
    if !handle.is_valid() {
        return Err(invalid_argument(b"invalid runtime session handle"));
    }
    let registry = lock_registry_until(deadline).ok_or_else(teardown_incomplete)?;
    registry
        .sessions
        .get(&handle.raw())
        .cloned()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn session_is_closing(handle: ZrRuntimeSessionHandle) -> bool {
    find_session_slot(handle).is_ok_and(|slot| slot.is_closing())
}

#[cfg(test)]
pub(super) fn session_active_actions(handle: ZrRuntimeSessionHandle) -> Option<usize> {
    find_session_slot(handle)
        .ok()
        .map(|slot| slot.active_actions())
}

#[cfg(test)]
#[path = "tests/store_handle_alloc.rs"]
mod handle_allocation_tests;
