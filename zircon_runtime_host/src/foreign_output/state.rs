//! Shared per-session protocol fuse and output accounting.
//!
//! 一个 Runtime session 的 App 与 Editor consumer 应共享同一状态：输出调用先经过类别预算，
//! 解码和释放后才竞争接受门；第一个协议违规会熔断会话，阻止并发完成的 payload 被继续接受。

use std::slice;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::de::DeserializeOwned;

use zircon_runtime_interface::{ZrOwnedResultV2, ZrStatus};

use super::decode::decode_bounded_json;
use super::kind::{RuntimeForeignOutputKind, RUNTIME_FOREIGN_OUTPUT_KIND_COUNT};
use super::metrics::{empty_counters, RuntimeForeignOutputCounters};
use super::{
    release_owned_result, validate_owned_result, RuntimeForeignOutputBudget,
    RuntimeForeignOutputError, RuntimeForeignOutputMetricsSnapshot, RuntimeOwnedOutputReleaser,
};

#[path = "state/diagnostic.rs"]
mod diagnostic;

use diagnostic::render_diagnostic_line;

/// 单个 Runtime session 共用的外来输出接受状态和诊断计数。
///
/// ## 使用时机与生命周期
///
/// App 在创建 runtime session 时初始化它；Editor gateway 可接收同一个 `Arc`，使两个 consumer
/// 共享熔断与指标。状态应至少活到该 session 的输出调用全部结束。
///
/// ## 约束
///
/// 所有输出接受/协议拒绝通过 `acceptance_gate` 串行提交；该锁不覆盖解码或 ABI release。接受值
/// 只有在 release 成功且此会话尚未熔断时才能返回给调用方。
pub struct RuntimeForeignOutputState {
    /// 串行化协议熔断与 payload 最终接受/拒绝的提交顺序。
    acceptance_gate: Mutex<()>,
    /// 一旦协议违规即保持 true 的会话 fuse。
    protocol_failed: AtomicBool,
    /// 由首次成功 fuse 递增的协议失败总数。
    protocol_failures: AtomicU64,
    /// session 级调用被 fuse 阻断的数量。
    blocked_session_calls: AtomicU64,
    /// 按 `RuntimeForeignOutputKind::index` 排列的输出类别计数器。
    counters: [RuntimeForeignOutputCounters; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT],
}

impl Default for RuntimeForeignOutputState {
    /// 创建尚未熔断、各类指标归零的新 session 状态。
    fn default() -> Self {
        Self {
            acceptance_gate: Mutex::new(()),
            protocol_failed: AtomicBool::new(false),
            protocol_failures: AtomicU64::new(0),
            blocked_session_calls: AtomicU64::new(0),
            counters: empty_counters(),
        }
    }
}

impl RuntimeForeignOutputState {
    /// 若会话尚未熔断则允许调用方继续处理指定类别，否则累计类别阻断数。
    ///
    /// `kind` 选择返回诊断文本及对应的类别计数。
    pub fn ensure_available(
        &self,
        kind: RuntimeForeignOutputKind,
    ) -> Result<(), RuntimeForeignOutputError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.counters[kind.index()].record_blocked_call();
        Err(fused_session_error(kind))
    }

    /// 读取 session 协议 fuse；Acquire 与首次熔断的 AcqRel swap 配对。
    pub fn is_protocol_failed(&self) -> bool {
        self.protocol_failed.load(Ordering::Acquire)
    }

    /// 在进入 Runtime session 级操作前检查 fuse 并累计被阻断调用。
    ///
    /// `operation` 仅用于错误诊断；该入口的阻断量记录在 session 级计数器。
    pub fn ensure_session_available(
        &self,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.blocked_session_calls.fetch_add(1, Ordering::Relaxed);
        Err(fused_session_call_error(operation))
    }

    /// 将已识别的协议错误提交为拒绝并熔断 session。
    ///
    /// `kind` 选择类别计数；此入口没有实际输出或解码耗时，因此按零字节、零耗时记录。
    pub fn reject_protocol<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        error: impl std::fmt::Display,
    ) -> Result<T, RuntimeForeignOutputError> {
        self.reject(
            kind,
            0,
            Duration::ZERO,
            RuntimeForeignOutputError::protocol_violation(error.to_string()),
        )
    }

    /// Consumes a foreign status and releases any output returned by a failed call.
    ///
    /// `status` 成功时将输出交给后续解码；失败时先验证并释放失败调用仍交回的输出。普通
    /// `RuntimeCall` 错误且输出所有权、清理均正常时不熔断；所有权或清理本身异常时按协议违规熔断。
    ///
    /// # Safety
    ///
    /// `status` and `output` must come from the same live provider/session as `releaser`.
    /// Failure diagnostics and non-empty output bytes must remain synchronously readable, and the
    /// caller must transfer unique release authority for `output` to this method.
    pub unsafe fn ensure_call_succeeded(
        &self,
        status: ZrStatus,
        output: ZrOwnedResultV2,
        releaser: RuntimeOwnedOutputReleaser,
        kind: RuntimeForeignOutputKind,
        operation: &'static str,
        release_operation: &'static str,
    ) -> Result<ZrOwnedResultV2, RuntimeForeignOutputError> {
        // SAFETY: 本方法调用者须保证 status 来自与 releaser 相同且仍存活的 provider/session，
        // 其失败诊断在同步读取期间有效；该约束与下方 output 的释放权约束由本方法的 Safety 契约共同给出。
        let Some(call_error) =
            (unsafe { RuntimeForeignOutputError::from_status(status, operation) })
        else {
            return Ok(output);
        };
        self.counters[kind.index()].record_call_failure();
        let encoded_len = reported_len(&output);
        let ownership_error = validate_owned_result(&output, release_operation).err();
        // SAFETY: 本方法的 Safety 契约要求 output 来自绑定的存活 provider/session 且转移唯一释放权；
        // release_owned_result 按该来源调用回调，失败诊断由同一存活 provider 在同步读取期提供。
        let release_error =
            unsafe { release_owned_result(output, releaser, release_operation) }.err();
        match (ownership_error, release_error) {
            // BUG: [ZCA-HST-001] 若失败 status 带有可读但超过 diagnostics 上限的字节，from_status
            // 返回 ProtocolViolation；对规范 empty output，validate 与 release 均成功，因此此臂只返回错误，
            // 不调用 reject，protocol_failed/rejected/protocol_failures 保持不变，后续 availability 仍通过。
            // 可用无并发 fuse 的新 state、同一存活 provider/session、4097 个存活初始化诊断字节、规范 empty
            // output 和匹配 releaser 静态复现；checked_slice(4096) 在构造切片前拒绝，无需无效指针。
            // App/Editor 的调用方仅传播/映射错误，不补做 reject；此处不声称 Runtime 当前 producer 会生成该输入。
            (None, None) => Err(call_error),
            (ownership_error, release_error) => {
                let mut error = RuntimeForeignOutputError::protocol_violation(
                    match ownership_error {
                        Some(ownership_error) => format!(
                            "{call_error}; foreign call returned invalid output ownership: {ownership_error}"
                        ),
                        None => call_error.to_string(),
                    },
                );
                if let Some(release_error) = release_error {
                    error = error.with_cleanup_failure(&release_error);
                }
                self.reject(kind, encoded_len, Duration::ZERO, error)
            }
        }
    }

    /// Decodes and releases a runtime-owned JSON result.
    ///
    /// ```compile_fail
    /// use zircon_runtime_host::foreign_output::{
    ///     RuntimeForeignOutputBudget, RuntimeForeignOutputKind, RuntimeForeignOutputState,
    ///     RuntimeOwnedOutputReleaser,
    /// };
    /// use zircon_runtime_interface::ZrOwnedResultV2;
    ///
    /// fn decode(
    ///     state: &RuntimeForeignOutputState,
    ///     output: ZrOwnedResultV2,
    ///     releaser: RuntimeOwnedOutputReleaser,
    ///     budget: RuntimeForeignOutputBudget,
    /// ) {
    ///     let _ = state.decode_json::<serde_json::Value, &str>(
    ///         output,
    ///         releaser,
    ///         RuntimeForeignOutputKind::ProfileResponse,
    ///         budget,
    ///         "decode",
    ///         "release",
    ///         |_| Ok(0),
    ///     );
    /// }
    /// ```
    ///
    /// 空输出按 `allow_empty` 策略接受为 `None`；非空输出依序检查所有权元数据、编码字节上限、
    /// 有界 JSON 预检和类型化 DTO 条目数。失败路径先尝试释放 allocation，再在接受门内记录拒绝
    /// 并设置 session fuse；成功解析也要先释放，再在同一接受门内复核 fuse。
    ///
    /// # Safety
    ///
    /// `output` must come from the same live provider/session as `releaser`. For a non-empty
    /// result, its pointer must remain valid, immutable, and readable for its reported length until
    /// this method releases it. The caller must transfer unique release authority to this method.
    pub unsafe fn decode_json<T, E>(
        &self,
        output: ZrOwnedResultV2,
        releaser: RuntimeOwnedOutputReleaser,
        kind: RuntimeForeignOutputKind,
        budget: RuntimeForeignOutputBudget,
        operation: &'static str,
        release_operation: &'static str,
        validate: impl FnOnce(&T) -> Result<usize, E>,
    ) -> Result<Option<T>, RuntimeForeignOutputError>
    where
        T: DeserializeOwned,
        E: std::fmt::Display,
    {
        let encoded_len = reported_len(&output);
        if self.is_protocol_failed() {
            return self.reject_and_release(
                output,
                releaser,
                kind,
                encoded_len,
                Duration::ZERO,
                fused_session_error(kind),
                release_operation,
            );
        }
        let encoded_len = match validate_owned_result(&output, operation) {
            Ok(encoded_len) => encoded_len,
            Err(error) => {
                return self.reject_and_release(
                    output,
                    releaser,
                    kind,
                    encoded_len,
                    Duration::ZERO,
                    error,
                    release_operation,
                )
            }
        };
        if let Err(error) = budget.validate_encoded_len(encoded_len, operation) {
            return self.reject_and_release(
                output,
                releaser,
                kind,
                encoded_len,
                Duration::ZERO,
                error,
                release_operation,
            );
        }
        if encoded_len == 0 {
            if !budget.allow_empty {
                return self.reject_and_release(
                    output,
                    releaser,
                    kind,
                    0,
                    Duration::ZERO,
                    RuntimeForeignOutputError::protocol_violation(format!(
                        "{operation} returned an empty payload"
                    )),
                    release_operation,
                );
            }
            return self.finish_acceptance(
                output,
                releaser,
                kind,
                0,
                Duration::ZERO,
                operation,
                release_operation,
                None,
            );
        }

        // SAFETY: 前面的 validate_owned_result 已确认非空数据指针非空、长度不超过 isize::MAX，
        // 本方法 Safety 契约进一步要求整段字节仍属于该存活 provider/session 的可读不可变 allocation，
        // 且唯一释放权交给本次调用；此借用在 release_owned_result 之前结束。
        let bytes = unsafe { slice::from_raw_parts(output.data, encoded_len) };
        let (decoded, decode_time) = decode_bounded_json(bytes, budget, operation, validate);
        match decoded {
            Ok(decoded) => self.finish_acceptance(
                output,
                releaser,
                kind,
                encoded_len,
                decode_time,
                operation,
                release_operation,
                Some(decoded),
            ),
            Err(error) => self.reject_and_release(
                output,
                releaser,
                kind,
                encoded_len,
                decode_time,
                error,
                release_operation,
            ),
        }
    }

    /// 快照读取 fuse、协议计数以及全部类别的原子指标。
    pub fn metrics(&self) -> RuntimeForeignOutputMetricsSnapshot {
        RuntimeForeignOutputMetricsSnapshot {
            protocol_failed: self.is_protocol_failed(),
            protocol_failures: self.protocol_failures.load(Ordering::Relaxed),
            blocked_session_calls: self.blocked_session_calls.load(Ordering::Relaxed),
            by_kind: std::array::from_fn(|index| self.counters[index].snapshot()),
        }
    }

    /// 有活动时返回稳定的单行诊断；全零状态返回 `None`。
    pub fn diagnostic_line(&self) -> Option<String> {
        render_diagnostic_line(self.metrics())
    }

    /// 先释放被拒绝输出，再把错误和实际处理量提交到共享 fuse。
    fn reject_and_release<T>(
        &self,
        output: ZrOwnedResultV2,
        releaser: RuntimeOwnedOutputReleaser,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
        release_operation: &'static str,
    ) -> Result<T, RuntimeForeignOutputError> {
        // SAFETY: 调用者按 decode_json 的 Safety 契约交付相同存活 provider/session 的 allocation，
        // 并转移唯一释放权；该辅助函数在释放完成后才返回错误。
        let error = match unsafe { release_owned_result(output, releaser, release_operation) } {
            Ok(()) => error,
            Err(release_error) => error.with_cleanup_failure(&release_error),
        };
        self.reject(kind, encoded_len, decode_time, error)
    }

    /// 释放成功解码的输出，然后在接受门内提交接受或因并发 fuse 转为拒绝。
    fn finish_acceptance<T>(
        &self,
        output: ZrOwnedResultV2,
        releaser: RuntimeOwnedOutputReleaser,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        operation: &'static str,
        release_operation: &'static str,
        value: Option<T>,
    ) -> Result<Option<T>, RuntimeForeignOutputError> {
        // ABI callback 在获取 acceptance gate 之前运行；不让外部代码持有本地状态锁。
        // SAFETY: 调用者按 decode_json 的 Safety 契约交付该 allocation 的唯一释放权；回调在获取
        // acceptance_gate 之前执行，provider/session 必须在此同步释放期间保持存活。
        let release_error =
            unsafe { release_owned_result(output, releaser, release_operation) }.err();
        let _acceptance = self
            .acceptance_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(release_error) = release_error {
            // 清理失败证明输出协议无法完成，须在同一接受门内先记录拒绝再熔断。
            return self.reject_locked(
                kind,
                encoded_len,
                decode_time,
                RuntimeForeignOutputError::protocol_violation(format!(
                    "{operation} cleanup failed: {release_error}"
                )),
            );
        }
        // 解码可能与另一线程的 reject 并发；在此重查才能阻止迟到的成功值越过已触发的 fuse。
        if self.is_protocol_failed() {
            self.counters[kind.index()].record_rejected(encoded_len, decode_time);
            return Err(fused_session_error(kind));
        }
        self.counters[kind.index()].record_accepted(encoded_len, decode_time);
        Ok(value)
    }

    /// 获取接受门并提交一项拒绝，保证 fuse 与接受结果有统一先后顺序。
    fn reject<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
    ) -> Result<T, RuntimeForeignOutputError> {
        let _acceptance = self
            .acceptance_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.reject_locked(kind, encoded_len, decode_time, error)
    }

    /// 调用方必须已持有 acceptance gate；累计拒绝并只让首个违规递增 fuse 次数。
    fn reject_locked<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
    ) -> Result<T, RuntimeForeignOutputError> {
        self.counters[kind.index()].record_rejected(encoded_len, decode_time);
        if !self.protocol_failed.swap(true, Ordering::AcqRel) {
            self.protocol_failures.fetch_add(1, Ordering::Relaxed);
        }
        Err(error)
    }
}

/// 构造类别级 fuse 错误，不改变状态；计数由调用点负责。
fn fused_session_error(kind: RuntimeForeignOutputKind) -> RuntimeForeignOutputError {
    RuntimeForeignOutputError::protocol_violation(format!(
        "runtime session rejected {} because a prior foreign-output protocol violation fused the session",
        kind.label()
    ))
}

/// 构造 session 级 fuse 错误，不改变状态；计数由调用点负责。
fn fused_session_call_error(operation: &'static str) -> RuntimeForeignOutputError {
    RuntimeForeignOutputError::protocol_violation(format!(
        "runtime session rejected {operation} because a prior foreign-output protocol violation fused the session"
    ))
}

/// 将外部长度用于拒绝计数；超出地址空间时饱和为最大 usize。
fn reported_len(output: &ZrOwnedResultV2) -> usize {
    usize::try_from(output.len).unwrap_or(usize::MAX)
}
