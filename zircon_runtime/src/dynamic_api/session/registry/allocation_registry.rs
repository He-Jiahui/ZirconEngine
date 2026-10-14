//! 保存 Runtime 返回给 host 的字节所有权，并以会话绑定的编号回收一次分配。
//!
//! 非空 `Vec` 移入注册表后地址保持不变，ABI 描述符只借用其字节，须经同一会话释放。
//! census 将尚未归还的输出纳入销毁条件；空输出没有 allocation，也不会建立 census。
//! 本模块不从 host 提交的裸地址重建容器，不以 allocation 编号充当内存地址。

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock, TryLockError};
use std::time::Instant;

use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeSessionHandle, ZrStatus,
};

use super::session_store::{begin_session_action, begin_session_release_action};
use crate::dynamic_api::session::status::{invalid_argument, not_found};

static RUNTIME_ALLOCATIONS: OnceLock<Mutex<RuntimeAllocationRegistry>> = OnceLock::new();

/// 标识产生字节的输出通道；类别保存在记录中，释放不按类别分派。
///
/// # 参考
/// `session/ffi.rs` 的各结果 finalizer 与 `session/operation.rs::harvest_operation`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::dynamic_api::session) enum RuntimeAllocationKind {
    /// 帧的 RGBA 字节，不包含帧描述符自身。
    Frame,
    /// 编码后的可访问性树。
    Accessibility,
    /// profile control 的编码响应。
    Profile,
    /// 准备提交的 host request 批次。
    HostRequests,
    /// world query 或 invalidation 批次的编码结果。
    WorldSync,
    /// 指定订阅的 plugin event 批次。
    PluginEvents,
    /// operation harvest 的编码结果。
    Operation,
}

/// 在 allocation 锁内取得的会话统计快照；它自身不保留任何输出的存活期。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimeAllocationCensus {
    /// 当前尚未成功归还的非空分配数。
    pub(super) outstanding_allocations: u64,
    /// 上述分配的逻辑字节长度之和，不计容器容量。
    pub(super) outstanding_bytes: u64,
    /// 本会话 census 自创建以来的分配数峰值，归还输出不清零。
    pub(super) high_water_allocations: u64,
    /// 本会话 census 自创建以来的逻辑字节峰值，成功销毁会话才移除统计项。
    pub(super) high_water_bytes: u64,
}

/// 一个编号对应的唯一所有权；描述符中的指针借用此处的 `bytes`。
struct RuntimeAllocationRecord {
    /// 释放时用于拒绝其他会话的编号，避免修改真正所有者的统计。
    session: ZrRuntimeSessionHandle,
    /// 记录来源，但不参与释放身份校验。
    _kind: RuntimeAllocationKind,
    bytes: Vec<u8>,
}

/// 进程级登记簿，记录与 census 由同一把锁更新。
struct RuntimeAllocationRegistry {
    /// 从一递增且不复用；达到最大值时拒绝继续登记。
    next_id: u64,
    allocations: HashMap<u64, RuntimeAllocationRecord>,
    census: HashMap<u64, RuntimeAllocationCensus>,
}

impl Default for RuntimeAllocationRegistry {
    fn default() -> Self {
        Self {
            next_id: 1,
            allocations: HashMap::new(),
            census: HashMap::new(),
        }
    }
}

fn registry() -> &'static Mutex<RuntimeAllocationRegistry> {
    RUNTIME_ALLOCATIONS.get_or_init(|| Mutex::new(RuntimeAllocationRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, RuntimeAllocationRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 尝试取得销毁所需的统计锁；竞争时让出线程，截止时间到达则放弃本次等待。
fn lock_registry_until(
    deadline: Instant,
) -> Option<MutexGuard<'static, RuntimeAllocationRegistry>> {
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

/// 在最终删除会话时保留 allocation 锁，使检查零分配与移除 census 处于同一临界区。
pub(super) struct RuntimeAllocationDestroyGuard(MutexGuard<'static, RuntimeAllocationRegistry>);

impl RuntimeAllocationDestroyGuard {
    pub(super) fn has_census(&self, session: ZrRuntimeSessionHandle) -> bool {
        self.0.census.contains_key(&session.raw())
    }

    pub(super) fn has_outstanding_allocations(&self, session: ZrRuntimeSessionHandle) -> bool {
        self.0
            .census
            .get(&session.raw())
            .is_some_and(|census| census.outstanding_allocations != 0)
    }

    pub(super) fn forget_empty_census(&mut self, session: ZrRuntimeSessionHandle) {
        // 调用方已检查零分配；debug 断言只校验这个前提，不是 release 配置下的拒绝分支。
        debug_assert!(!self.has_outstanding_allocations(session));
        self.0.census.remove(&session.raw());
    }
}

/// 不等待 allocation 锁；最终删除方可先归还 session 锁后重试，避免持前锁阻塞释放路径。
pub(super) fn try_lock_registry_for_destroy() -> Option<RuntimeAllocationDestroyGuard> {
    match registry().try_lock() {
        Ok(guard) => Some(RuntimeAllocationDestroyGuard(guard)),
        Err(TryLockError::Poisoned(poisoned)) => {
            Some(RuntimeAllocationDestroyGuard(poisoned.into_inner()))
        }
        Err(TryLockError::WouldBlock) => None,
    }
}

#[cfg(test)]
pub(super) fn with_registry_lock_for_test(action: impl FnOnce()) {
    let _registry = lock_registry();
    action();
}

/// 先取得普通动作 guard，再登记一个输出；此包装器的 guard 在登记返回时结束。
///
/// # 参数
/// `session` 须仍接受普通动作；`kind` 记录输出来源，`bytes` 的所有权在成功时交给注册表。
///
/// # 返回值
/// 可跨 ABI 传递的描述符或准入/登记错误；空字节返回无 allocation 的规范空值。
///
/// # 参考
/// [`register_runtime_allocation_in_action`]；`tests/cases.rs` 的归属、重复释放与并发释放测试。
pub(in crate::dynamic_api::session) fn register_runtime_allocation(
    session: ZrRuntimeSessionHandle,
    kind: RuntimeAllocationKind,
    bytes: Vec<u8>,
) -> Result<ZrOwnedResultV2, ZrStatus> {
    let _action = begin_session_action(session)?;
    register_runtime_allocation_in_action(session, kind, bytes)
}

// The caller must hold the session action lease through registration and ABI output publication.
/// 在已有动作范围内取得跨 ABI 输出的所有权编号，不再次进行会话准入。
///
/// # 参数
/// `session` 是调用链已获准的会话；`kind` 仅标记来源；非空 `bytes` 移入注册表，不再借用调用栈。
///
/// # 使用时机与生命周期
/// 调用方必须持有该会话的动作 guard，至少覆盖本次登记；函数签名不验证此前提。
/// 非空描述符的字节直到同会话成功调用 [`release_runtime_allocation`] 前均由此模块拥有。
/// 本函数不写 ABI 输出；仅登记后返回的 committed 调用链在 guard 外发布结果，
/// 具体边界见 `session_store.rs::with_session_result_committed`，非空分配继续约束销毁。
///
/// # 返回值
/// 空输入不消耗编号或统计项；其余错误拒绝登记，已经消耗的递增编号不会复用。
/// 成功结果的 `data/len/allocation` 描述同一个记录，不授予 host 自行释放 `Vec` 的权利。
///
/// # 参考
/// `session/ffi.rs::capture_frame`、`drain_host_requests`；`tests/cases.rs` 的 finalizer 销毁屏障测试。
pub(in crate::dynamic_api::session) fn register_runtime_allocation_in_action(
    session: ZrRuntimeSessionHandle,
    kind: RuntimeAllocationKind,
    bytes: Vec<u8>,
) -> Result<ZrOwnedResultV2, ZrStatus> {
    if bytes.is_empty() {
        return Ok(ZrOwnedResultV2::empty());
    }
    let len = u64::try_from(bytes.len())
        .map_err(|_| invalid_argument(b"runtime allocation exceeds ABI length range"))?;
    let data = bytes.as_ptr();

    let mut registry = lock_registry();
    if registry.next_id == u64::MAX {
        return Err(invalid_argument(b"runtime allocation ID space exhausted"));
    }
    let raw_id = registry.next_id;
    registry.next_id += 1;

    let previous = registry
        .census
        .get(&session.raw())
        .copied()
        .unwrap_or_default();
    let outstanding_allocations = previous
        .outstanding_allocations
        .checked_add(1)
        .ok_or_else(|| invalid_argument(b"runtime allocation census overflow"))?;
    let outstanding_bytes = previous
        .outstanding_bytes
        .checked_add(len)
        .ok_or_else(|| invalid_argument(b"runtime allocation byte census overflow"))?;
    registry.census.insert(
        session.raw(),
        RuntimeAllocationCensus {
            outstanding_allocations,
            outstanding_bytes,
            high_water_allocations: previous.high_water_allocations.max(outstanding_allocations),
            high_water_bytes: previous.high_water_bytes.max(outstanding_bytes),
        },
    );
    registry.allocations.insert(
        raw_id,
        RuntimeAllocationRecord {
            session,
            _kind: kind,
            bytes,
        },
    );

    Ok(ZrOwnedResultV2 {
        data,
        len,
        allocation: ZrRuntimeAllocationId::new(raw_id),
    })
}

/// 按会话与编号归还一次非空输出，移出记录后在 allocation 锁外销毁字节容器。
///
/// # 参数
/// `session` 必须匹配登记时的会话；`allocation` 是返回描述符中的有效编号，不能用裸地址代替。
///
/// # 返回值
/// 唯一成功移出记录的调用返回 Ok；无效、未知、重复或其他会话的编号返回 NotFound。
/// 关闭阶段拒绝准入；销毁失败后的重试状态仍允许归还，使剩余输出可以排空。
///
/// # 使用时机与生命周期
/// 成功后描述符的字节立即失效。释放动作 guard 保留到容器销毁结束，防止将回收过程漏出动作计数。
///
/// # 参考
/// `session/ffi.rs::release_allocation`；`tests/cases.rs` 的 exactly-once 与 release-and-retry 测试。
pub(in crate::dynamic_api::session) fn release_runtime_allocation(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    if !allocation.is_valid() {
        return not_found(b"runtime allocation not found");
    }
    let _action = match begin_session_release_action(session) {
        Ok(action) => action,
        Err(status) => return status,
    };
    let record = {
        let mut registry = lock_registry();
        let Some(record) = registry.allocations.get(&allocation.raw()) else {
            return not_found(b"runtime allocation not found");
        };
        if record.session != session {
            return not_found(b"runtime allocation not found");
        }
        let record = registry
            .allocations
            .remove(&allocation.raw())
            .expect("validated runtime allocation must remain present while registry is locked");
        let byte_len = record.bytes.len() as u64;
        if let Some(census) = registry.census.get_mut(&record.session.raw()) {
            census.outstanding_allocations = census.outstanding_allocations.saturating_sub(1);
            census.outstanding_bytes = census.outstanding_bytes.saturating_sub(byte_len);
        }
        record
    };
    drop(record);
    ZrStatus::ok()
}

/// 复制当前统计；无项时返回全零，峰值只有成功删除 census 后才消失。
pub(super) fn allocation_census(session: ZrRuntimeSessionHandle) -> RuntimeAllocationCensus {
    lock_registry()
        .census
        .get(&session.raw())
        .copied()
        .unwrap_or_default()
}

/// 在销毁截止时间内读取未归还分配数；None 表示锁未及时取得，不能解释为零分配。
pub(super) fn session_has_outstanding_allocations_until(
    session: ZrRuntimeSessionHandle,
    deadline: Instant,
) -> Option<bool> {
    let registry = lock_registry_until(deadline)?;
    Some(
        registry
            .census
            .get(&session.raw())
            .is_some_and(|census| census.outstanding_allocations != 0),
    )
}
