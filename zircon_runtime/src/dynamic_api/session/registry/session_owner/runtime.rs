//! 将 Runtime 会话构造、状态诊断和关闭顺序接到通用 owner 任务。
//!
//! owner 中保存成功会话或仍有清理责任的启动失败；启动错误诊断先复制，
//! 原 Core、scope、插件代次和进程日志保留到规范关闭完成。
//! 启动握手成功后，注册表才通过 SessionSlot 发布会话句柄；失败返回前须先取得真实 owner join。

use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use zircon_runtime_interface::ZrStatus;

use crate::diagnostic_log::DynamicProcessLogLease;
use crate::dynamic_api::session::construction::RuntimeConstructionFailure;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::registry::RuntimeWakeRegistration;
use crate::dynamic_api::session::status::error_status;
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

use super::abi_status::OwnedSessionStatus;
use super::{OwnerDispatchError, OwnerShutdownReceipt, SessionOwner};

// 启动失败沿用一次固定清理预算；重试和测试调度都不得另起延长的 engine deadline。
const OWNER_STARTUP_SHUTDOWN_WAIT: Duration = Duration::from_secs(5);

#[derive(Debug)]
/// 上层分别转换 owner 控制路径失败与已拥有诊断的会话构造失败。
pub(in crate::dynamic_api::session) enum RuntimeSessionOwnerCreateError {
    /// 池创建或启动握手失败，尚无可发布的成功会话。
    Owner(String),
    /// 原构造失败的状态码和诊断副本，可在调用线程重建 ABI status。
    Session(OwnedSessionStatus),
}

/// 留在 owner 线程的启动失败；诊断副本不取代原错误及其未完成资源所有权。
struct RuntimeSessionStartupFailure {
    /// 可跨线程复制的原诊断；后续清理错误不会覆盖它。
    status: OwnedSessionStatus,
    /// 保留 typed primary 与待关闭的同一 Core、scope 和插件代次。
    construction: RuntimeConstructionFailure,
    /// Core 尚未关闭时保留日志 lease，等规范 Core 关闭后再排空。
    dynamic_process_log: Option<DynamicProcessLogLease>,
}

impl RuntimeSessionStartupFailure {
    // 在工厂所在 owner 线程捕获诊断，同时把完整 failure 移入保留状态。
    fn capture(construction: RuntimeConstructionFailure) -> Self {
        // Capture only the diagnostic; the typed primary and unfinished Core remain owned here.
        // SAFETY: error_status 刚在本线程建立有界 TLS 诊断；capture 在下一次错误调用前同步复制。
        let status = unsafe { OwnedSessionStatus::capture(error_status(construction.primary())) };
        Self {
            status,
            construction,
            dynamic_process_log: None,
        }
    }

    // 未关闭 Core 继续持有日志；已关闭 Core 才直接释放 lease，失败沿用现有 fatal 策略。
    fn with_dynamic_process_log(mut self, mut log: DynamicProcessLogLease) -> Self {
        if self.construction.has_pending_core() {
            self.dynamic_process_log = Some(log);
        } else if !log.shutdown() {
            // Preserve the existing post-lease bootstrap log failure policy.
            eprintln!("fatal runtime session owner bootstrap log teardown incomplete");
            std::process::abort();
        }
        self
    }

    // 先重试原 construction 的关闭，再关闭日志；false 时保留仍未完成的资源。
    fn shutdown_until(&mut self, deadline: Instant) -> bool {
        if !self.construction.shutdown_until(deadline) {
            eprintln!(
                "runtime startup cleanup incomplete: {}; secondary shutdown: {}",
                self.construction.primary(),
                self.construction
                    .shutdown_error()
                    .expect("incomplete Core close diagnostic")
            );
            return false;
        }
        if let Some(log) = self.dynamic_process_log.as_mut() {
            if !log.shutdown_until(deadline) {
                return false;
            }
            self.dynamic_process_log = None;
        }
        Instant::now() <= deadline
    }
}

/// 对成功和失败两种 owner 状态使用各自规范关闭入口，true 才允许通用 owner 销毁状态。
fn shutdown_owner_state_until(
    state: &mut Result<RuntimeDynamicSession, RuntimeSessionStartupFailure>,
    deadline: Instant,
) -> bool {
    match state {
        Ok(session) => session.shutdown_before_library_unload_until(deadline),
        Err(failure) => failure.shutdown_until(deadline),
    }
}

/// 会话状态的线程所有者与 wake 关闭控制器；外层 SessionSlot 另外管理动作准入和计数。
///
/// # 使用时机与生命周期
/// 生产创建在 owner 任务内完成，启动握手之后才能发布槽位。
/// 关闭先停止新增 wake 并排空已有回调，再关闭 owner 状态；未完成必须保留控制器。
///
/// # 参考
/// `registry/session_slot.rs::SessionSlot`；[`SessionOwner`]；[`OwnedSessionStatus`]。
pub(in crate::dynamic_api::session) struct RuntimeSessionOwner {
    /// 两种状态均留在一个长期任务内，失败分支也持有真实清理责任。
    owner: SessionOwner<Result<RuntimeDynamicSession, RuntimeSessionStartupFailure>>,
    /// 与会话生产者共享登记，关闭在清理状态前排空外部回调。
    wake: RuntimeWakeRegistration,
    #[cfg(test)]
    startup_observation: Option<startup_observation::ObservationSink>,
}

impl RuntimeSessionOwner {
    #[cfg(test)]
    /// 将测试已构造的会话移动到 owner；此入口不覆盖生产工厂的创建线程契约。
    pub(in crate::dynamic_api::session) fn from_session(
        session: RuntimeDynamicSession,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, String> {
        let owner = SessionOwner::create(move || Ok(session), code_owner)?;
        Ok(Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation: None,
        })
    }

    /// 在 owner 任务内创建生产会话，并通过借用命令握手确认构造结果。
    ///
    /// # 参数
    /// profile/project 由工厂消费；wake 同时交给生产者与关闭方；code_owner 留到控制器释放。
    ///
    /// # 返回值与生命周期
    /// 成功返回供未发布的 SessionSlot 持有的控制器。失败先处理原状态并等待 owner join，
    /// 再返回分类错误；握手阶段不会把失败状态发布为可访问的会话。
    pub(in crate::dynamic_api::session) fn create(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let owner_wake = wake.clone();
        #[cfg(test)]
        let startup_observation = startup_observation::current();
        #[cfg(test)]
        let factory_observation = startup_observation.clone();
        let owner = SessionOwner::create(
            move || {
                let log =
                    crate::diagnostic_log::acquire_dynamic_unity_process_log("runtime-dynamic");
                let session = RuntimeDynamicSession::new(profile, project);
                crate::diagnostic_log::write_log(
                    "runtime_session",
                    "runtime_session_owner_factory_returned",
                );
                match session {
                    Ok(session) => Ok(session
                        .with_runtime_frame_wake(owner_wake.channel_wake())
                        .with_dynamic_process_log_lease(log)),
                    Err(error) => {
                        #[cfg(test)]
                        startup_observation::record_failure(&factory_observation, &error);
                        Err(RuntimeSessionStartupFailure::capture(error)
                            .with_dynamic_process_log(log))
                    }
                }
            },
            code_owner,
        )
        .map_err(RuntimeSessionOwnerCreateError::Owner)?;
        crate::diagnostic_log::write_log("runtime_session", "runtime_session_owner_task_spawned");
        let session_owner = Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation,
        };
        let startup = session_owner.owner.dispatch_scoped(|state| match state {
            Ok(_) => Ok(()),
            Err(error) => Err(error.status.clone()),
        });
        crate::diagnostic_log::write_log(
            "runtime_session",
            "runtime_session_owner_bootstrap_returned",
        );
        match startup {
            Ok(Ok(())) => Ok(session_owner),
            Ok(Err(error)) => {
                // 诊断已经独立复制，真正的 failure 仍在 owner 中，必须先关闭并 join。
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Session(error))
            }
            Err(error) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Owner(format!(
                    "runtime session owner terminated during startup: {error:?}"
                )))
            }
        }
    }

    /// 准入拥有全部捕获数据的会话动作；deadline 只限制入队，结果接收另由调用方等待。
    ///
    /// # 约束与返回值
    /// action 的错误诊断须来自该 owner 线程刚完成的 Runtime 调用，并有效到本次同步复制结束。
    /// 接收端外层是动作结果，错误已复制为 OwnedSessionStatus；此处不登记槽位动作计数。
    pub(in crate::dynamic_api::session) fn dispatch<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send + 'static,
    ) -> Result<Receiver<Result<R, OwnedSessionStatus>>, OwnerDispatchError> {
        self.owner.dispatch(deadline, move |state| {
            let session = state.as_mut().map_err(|error| error.status.clone())?;
            // SAFETY: 内部会话动作返回同线程 Runtime/static 诊断；在下一次 Runtime 调用前复制，
            // 结果通道只传拥有字节的 OwnedSessionStatus，不传原诊断指针。
            action(session).map_err(|status| unsafe { OwnedSessionStatus::capture(status) })
        })
    }

    /// 使用调用方移交的插件登记报告在 owner 上构造会话，启动失败仍先关闭原状态并 join。
    /// profile/project/wake/code_owner 的责任与 create 相同；registrations 由此工厂消费。
    /// 与普通工厂不同，此分支没有获取 dynamic process log lease。
    pub(in crate::dynamic_api::session) fn create_with_linked_plugins(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        registrations: Vec<RuntimePluginRegistrationReport>,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let owner_wake = wake.clone();
        #[cfg(test)]
        let startup_observation = startup_observation::current();
        #[cfg(test)]
        let factory_observation = startup_observation.clone();
        let owner = SessionOwner::create(
            move || {
                RuntimeDynamicSession::new_with_linked_plugins(profile, project, registrations)
                    .map(|session| session.with_runtime_frame_wake(owner_wake.channel_wake()))
                    .map_err(|error| {
                        #[cfg(test)]
                        startup_observation::record_failure(&factory_observation, &error);
                        RuntimeSessionStartupFailure::capture(error)
                    })
            },
            code_owner,
        )
        .map_err(RuntimeSessionOwnerCreateError::Owner)?;
        let session_owner = Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation,
        };
        match session_owner.owner.dispatch_scoped(|state| match state {
            Ok(_) => Ok(()),
            Err(error) => Err(error.status.clone()),
        }) {
            Ok(Ok(())) => Ok(session_owner),
            Ok(Err(error)) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Session(error))
            }
            Err(error) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Owner(format!(
                    "runtime session owner terminated during startup: {error:?}"
                )))
            }
        }
    }

    /// 在 owner 执行可借用调用栈的会话动作，完成信号之后才返回或传播回调 panic。
    /// 返回外层分派错误或内层会话结果；错误诊断在离开 owner 线程前复制。
    /// action 返回 status 的可读性前提与 dispatch 相同，上层仍须保留槽位动作 guard。
    pub(in crate::dynamic_api::session) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
    ) -> Result<Result<R, OwnedSessionStatus>, OwnerDispatchError> {
        self.owner.dispatch_scoped(move |state| {
            let session = state.as_mut().map_err(|error| error.status.clone())?;
            // SAFETY: SessionSlot 的内部动作同步返回 owner 线程的 Runtime/static 诊断，
            // 此处立即复制，不在中间调用会覆盖 TLS 的 Runtime 错误入口。
            action(session).map_err(|status| unsafe { OwnedSessionStatus::capture(status) })
        })
    }

    /// 先禁止并排空 wake 回调，再用同一 deadline 请求状态清理与 owner join。
    /// Pending 保留原控制器；已有清理回执未完成时不会再次调用清理。
    pub(in crate::dynamic_api::session) fn shutdown_until(
        &self,
        deadline: Instant,
    ) -> OwnerShutdownReceipt {
        self.wake.disable_new_entries();
        if !self
            .wake
            .wait_for_callbacks(deadline.saturating_duration_since(Instant::now()))
        {
            return OwnerShutdownReceipt::Pending;
        }
        self.owner.shutdown_until(deadline, move |state| {
            shutdown_owner_state_until(state, deadline)
        })
    }

    /// 无等待地观察现有回执，不重新关闭 wake、执行清理或 join。
    pub(in crate::dynamic_api::session) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        self.owner.shutdown_receipt()
    }

    // 失败控制器尚未发布；返回原错误前要结束其真实任务，不能把未完成责任直接 Drop。
    fn join_failed_startup(&self) {
        let deadline = Instant::now()
            .checked_add(OWNER_STARTUP_SHUTDOWN_WAIT)
            .unwrap_or_else(Instant::now);
        #[cfg(all(test, windows))]
        super::receipt_publication::startup_deadline(deadline);
        #[cfg(test)]
        let startup_observation = self.startup_observation.clone();
        let receipt = self.owner.shutdown_until(deadline, move |state| {
            #[cfg(all(test, windows))]
            super::receipt_publication::callback_entered(deadline);
            let completed = shutdown_owner_state_until(state, deadline);
            #[cfg(test)]
            startup_observation::record_cleanup(&startup_observation, state, completed);
            #[cfg(all(test, windows))]
            super::receipt_publication::callback_completed(deadline, completed);
            completed
        });
        #[cfg(test)]
        startup_observation::record_owner_receipt(&self.startup_observation, receipt);
        // BUG: [CR-OWNER-0001] 发布者持有真实回执 wait 锁但尚未 store 成功时，
        // wait_until 的 WouldBlock 会提前返回 Pending；即使本次 deadline 尚未耗尽，
        // 此分支仍 abort，使原 late Play 加载错误无法返回调用方。
        // 证据：CleanupReceipt::publish/wait_until → SessionOwner::shutdown_until_inner；
        // construction/tests/receipt_publication.rs 的真实公开回归安排此竞态，
        // 要求原 ReadPlayScene 错误返回且同 owner 完成真实 join；本批仅静态核对，未执行测试。
        if !matches!(
            receipt,
            OwnerShutdownReceipt::Joined | OwnerShutdownReceipt::Panicked
        ) {
            eprintln!("fatal runtime session owner startup teardown incomplete");
            std::process::abort();
        }
    }
}

#[cfg(test)]
#[path = "../../tests/runtime_startup_observation.rs"]
pub(in crate::dynamic_api::session) mod startup_observation;

#[cfg(test)]
#[path = "tests/runtime.rs"]
mod tests;
