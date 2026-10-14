//! 为一个 owner 任务维护动作准入和关闭状态；会话状态本身不由此槽位加锁访问。
//!
//! 普通动作只在 Open 获准。关闭失败保留槽位并进入 TeardownRetryPending，
//! 该状态仅重新允许输出释放，供调用方排空 allocation 后显式重试销毁。

use std::any::Any;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use zircon_runtime_interface::ZrStatus;

use super::action_guard::SessionActionGuard;
use super::session_owner::abi_status::OwnedSessionStatus;
use super::session_owner::runtime::RuntimeSessionOwner;
use super::session_owner::runtime::RuntimeSessionOwnerCreateError;
use super::session_owner::{OwnerDispatchError, OwnerShutdownReceipt};
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

/// 会话注册表中的共享槽位，组合 owner 分派、动作计数与异步 wake 的生命周期。
///
/// # 使用时机与生命周期
/// 创建 owner 成功后才由 `session_store` 发布句柄。动作 guard 保留同一槽位，
/// 销毁方须先阻止新动作，再等待已有动作和 wake，最后检查 allocation 并取得 owner 的 Joined 回执。
/// 槽位引用不等于动态库代码的外部保留凭据；ABI 调用方仍须遵守 provider 存活期契约。
///
/// # 参考
/// `session_store.rs::destroy_session_slot_with_timeout`；[`SessionActionGuard`]；`tests/cases.rs`。
pub(in crate::dynamic_api::session) struct SessionSlot {
    /// 唯一访问可变会话状态的任务控制器，关闭回执与 worker join 由它管理。
    owner: RuntimeSessionOwner,
    /// 只保护准入状态与计数，不在执行会话回调时持有。
    lifecycle: Mutex<SessionSlotLifecycle>,
    /// 计数降为零时通知等待销毁的线程。
    actions_drained: Condvar,
    /// frame demand 与 wake 登记独立共享，销毁时关闭新增 wake 并排空回调。
    frame_activity: RuntimeFrameActivity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionSlotPhase {
    /// 普通动作和释放动作均可登记。
    Open,
    /// 当前销毁尝试关闭准入，已有动作继续完成。
    Closing,
    /// 保留尚未完成的关闭状态；拒绝普通动作，允许归还输出及再次销毁。
    TeardownRetryPending,
}

#[derive(Debug)]
struct SessionSlotLifecycle {
    phase: SessionSlotPhase,
    /// 每次获准动作增加一次，由对应 guard 唯一归还；不包含 wake 回调的独立计数。
    active_actions: usize,
}

impl SessionSlot {
    #[cfg(test)]
    /// 将测试已构造的会话交给 owner；在注入 wake 后创建零活动动作的 Open 槽位。
    pub(super) fn new(
        session: RuntimeDynamicSession,
        wake: RuntimeWakeRegistration,
    ) -> Result<Self, String> {
        let session = session.with_runtime_frame_wake(wake.channel_wake());
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = RuntimeSessionOwner::from_session(session, wake, code_owner)?;
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
    }

    /// 在 owner 任务上构造生产会话，启动成功后返回尚未发布的 Open 槽位。
    ///
    /// # 参数
    /// `profile/project` 交给 owner 工厂；`wake` 同时供会话生产者和关闭排空方使用。
    ///
    /// # 返回值
    /// 构造完成的槽位；启动失败已由 owner 处理未完成资源，诊断转换为当前调用线程的 ABI status。
    ///
    /// # 参考
    /// `session_store.rs::try_create_session_with_wake`；`session_owner/runtime.rs::RuntimeSessionOwner::create`。
    pub(super) fn create(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        wake: RuntimeWakeRegistration,
    ) -> Result<Self, ZrStatus> {
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = match RuntimeSessionOwner::create(profile, project, wake, code_owner) {
            Ok(owner) => owner,
            Err(RuntimeSessionOwnerCreateError::Session(error)) => return Err(error.into_abi()),
            Err(RuntimeSessionOwnerCreateError::Owner(error)) => {
                return Err(crate::dynamic_api::session::status::error_status(error));
            }
        };
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
    }

    /// 仅在 Open 状态增加普通动作计数；返回 guard 后不再持有生命周期锁。
    ///
    /// # 返回值
    /// Some 表示本动作已登记，None 表示关闭或待重试；调用方不能仅凭槽位 Arc 跳过准入。
    pub(super) fn begin_action(self: &Arc<Self>) -> Option<SessionActionGuard> {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase != SessionSlotPhase::Open {
            return None;
        }
        lifecycle.active_actions += 1;
        drop(lifecycle);
        Some(SessionActionGuard::new(Arc::clone(self)))
    }

    /// 使用已链接的插件登记报告创建 owner；禁用外部 wake，启动完成后交给注册表发布。
    /// `registrations` 由 owner 工厂消费，构造错误仍保留 owner/session 两种分类供上层转换。
    pub(super) fn create_with_linked_plugins(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        registrations: Vec<RuntimePluginRegistrationReport>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let wake = RuntimeWakeRegistration::disabled();
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = RuntimeSessionOwner::create_with_linked_plugins(
            profile,
            project,
            registrations,
            wake,
            code_owner,
        )?;
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
    }

    /// 允许 Open 或待重试状态归还输出，但拒绝当前 Closing 阶段的新释放动作。
    /// 返回的 guard 与普通动作共用计数，使销毁重试可观察正在进行的回收。
    pub(super) fn begin_release_action(self: &Arc<Self>) -> Option<SessionActionGuard> {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase == SessionSlotPhase::Closing {
            return None;
        }
        lifecycle.active_actions += 1;
        drop(lifecycle);
        Some(SessionActionGuard::new(Arc::clone(self)))
    }

    /// 归还一次已登记动作；只能由对应 guard 调用，零计数唤醒全部等待方。
    pub(super) fn finish_action(&self) {
        let mut lifecycle = self.lock_lifecycle();
        lifecycle.active_actions -= 1;
        if lifecycle.active_actions == 0 {
            self.actions_drained.notify_all();
        }
    }

    /// 进入或保持 Closing，不删除槽位，也不在此等待已有动作。
    /// 并发销毁方可共同进入，后续共享 owner 已保留的关闭回执，各自仍使用自己的截止时间。
    pub(super) fn begin_close(&self) -> bool {
        let mut lifecycle = self.lock_lifecycle();
        match lifecycle.phase {
            SessionSlotPhase::Open | SessionSlotPhase::TeardownRetryPending => {
                lifecycle.phase = SessionSlotPhase::Closing;
                true
            }
            // Concurrent destroy callers share the owner's retained receipt.
            SessionSlotPhase::Closing => true,
        }
    }

    /// 将非 Open 状态保留为待重试；不会恢复普通动作或重新启用 wake。
    pub(super) fn preserve_failed_teardown_for_retry(&self) {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase != SessionSlotPhase::Open {
            lifecycle.phase = SessionSlotPhase::TeardownRetryPending;
        }
    }

    /// 等待计数为零，超时仍保留计数与阶段；调用方应先通过 [`Self::begin_close`] 阻止普通新动作。
    /// 返回 true 只表示本次观察已排空，不代表 owner 已 Joined 或 allocation 已全部归还。
    pub(super) fn wait_for_actions(&self, timeout: Duration) -> bool {
        let lifecycle = self.lock_lifecycle();
        let (lifecycle, _) = self
            .actions_drained
            .wait_timeout_while(lifecycle, timeout, |state| state.active_actions != 0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        lifecycle.active_actions == 0
    }

    /// 同步在 owner 上执行可借用调用栈的 Send 回调，等待回调结束或被终止的 owner 丢弃后返回。
    /// 外层 Result 表示 owner 准入/终止错误，内层错误是离开 owner 线程前已复制的诊断。
    /// 本方法不登记动作；`session_store` 须另外保留动作 guard，且同一 owner 的同步重入会被拒绝。
    pub(super) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
    ) -> Result<Result<R, OwnedSessionStatus>, OwnerDispatchError> {
        self.owner.dispatch_scoped(action)
    }

    /// 将拥有全部捕获数据的命令送到 owner，`deadline` 只限制命令准入等待。
    /// 成功返回独立的结果接收端；执行与收取结果的等待不由此截止时间限制。
    /// `action/R` 的 Send 与静态生命周期允许跨线程传递，本方法不替代动作准入。
    pub(super) fn dispatch_owned<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send + 'static,
    ) -> Result<Receiver<Result<R, OwnedSessionStatus>>, OwnerDispatchError> {
        self.owner.dispatch(deadline, action)
    }

    /// 沿用销毁方的截止时间取得 owner 关闭回执；仅 Joined 表示正常清理和任务 join 完成。
    pub(super) fn shutdown_until(&self, deadline: Instant) -> OwnerShutdownReceipt {
        self.owner.shutdown_until(deadline)
    }

    /// 观察已保留的关闭回执，不重新执行清理，也不等待；None 表示尚无关闭请求。
    pub(super) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        self.owner.shutdown_receipt()
    }

    pub(super) fn frame_activity(&self) -> &RuntimeFrameActivity {
        &self.frame_activity
    }

    #[cfg(test)]
    /// 测试将 Closing 与待重试均视为已关闭普通动作准入。
    pub(super) fn is_closing(&self) -> bool {
        self.lock_lifecycle().phase != SessionSlotPhase::Open
    }

    #[cfg(test)]
    pub(super) fn active_actions(&self) -> usize {
        self.lock_lifecycle().active_actions
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, SessionSlotLifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
