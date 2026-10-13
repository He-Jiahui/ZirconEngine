//! Lock-free counters and snapshots for foreign-output activity.
//!
//! 本模块管理外来输出活动的无锁计数器及其快照。
//!
//! 计数器属于一个共享 `RuntimeForeignOutputState`，使用 Relaxed 原子操作记录接受、拒绝、
//! 调用失败、熔断阻断和解码耗时；诊断读取通过快照避免持有状态锁。

use std::array;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::kind::{RuntimeForeignOutputKind, RUNTIME_FOREIGN_OUTPUT_KIND_COUNT};

#[derive(Default)]
/// 单一输出类别的原子累计计数。
pub(super) struct RuntimeForeignOutputCounters {
    /// 成功完成验证、解码和释放的 payload 数。
    accepted_payloads: AtomicU64,
    /// 已接受 payload 的编码字节累计。
    accepted_bytes: AtomicU64,
    /// 被预算、解码或清理路径拒绝的 payload 数。
    rejected_payloads: AtomicU64,
    /// 被拒绝 payload 的编码字节累计。
    rejected_bytes: AtomicU64,
    /// Runtime ABI 返回失败状态的调用数。
    call_failures: AtomicU64,
    /// 会话已经熔断后被阻断的类别调用数。
    blocked_calls: AtomicU64,
    /// 已记录解码时长的纳秒累计。
    total_decode_nanoseconds: AtomicU64,
    /// 单次解码的最大纳秒数。
    max_decode_nanoseconds: AtomicU64,
}

impl RuntimeForeignOutputCounters {
    /// 读取当前类别的近似一致快照。
    pub(super) fn snapshot(&self) -> RuntimeForeignOutputMetrics {
        RuntimeForeignOutputMetrics {
            accepted_payloads: self.accepted_payloads.load(Ordering::Relaxed),
            accepted_bytes: self.accepted_bytes.load(Ordering::Relaxed),
            rejected_payloads: self.rejected_payloads.load(Ordering::Relaxed),
            rejected_bytes: self.rejected_bytes.load(Ordering::Relaxed),
            call_failures: self.call_failures.load(Ordering::Relaxed),
            blocked_calls: self.blocked_calls.load(Ordering::Relaxed),
            total_decode_nanoseconds: self.total_decode_nanoseconds.load(Ordering::Relaxed),
            max_decode_nanoseconds: self.max_decode_nanoseconds.load(Ordering::Relaxed),
        }
    }

    /// 记录一次完成验证、解码和释放的接受结果。
    pub(super) fn record_accepted(&self, encoded_len: usize, decode_time: Duration) {
        self.accepted_payloads.fetch_add(1, Ordering::Relaxed);
        self.accepted_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    /// 记录一次拒绝结果及其处理耗时。
    pub(super) fn record_rejected(&self, encoded_len: usize, decode_time: Duration) {
        self.rejected_payloads.fetch_add(1, Ordering::Relaxed);
        self.rejected_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    /// 记录 Runtime ABI 返回失败状态。
    pub(super) fn record_call_failure(&self) {
        self.call_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录因会话熔断而未进入 Runtime 的调用。
    pub(super) fn record_blocked_call(&self) {
        self.blocked_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// 将非零解码时长折算为纳秒并更新总量、最大值。
    fn record_decode_time(&self, decode_time: Duration) {
        if decode_time.is_zero() {
            return;
        }
        let decode_nanoseconds = duration_to_u64_nanoseconds(decode_time).max(1);
        self.total_decode_nanoseconds
            .fetch_add(decode_nanoseconds, Ordering::Relaxed);
        self.max_decode_nanoseconds
            .fetch_max(decode_nanoseconds, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 单个输出类别的只读指标。
pub struct RuntimeForeignOutputMetrics {
    /// 已接受 payload 数。
    pub accepted_payloads: u64,
    /// 已接受编码字节数。
    pub accepted_bytes: u64,
    /// 已拒绝 payload 数。
    pub rejected_payloads: u64,
    /// 已拒绝编码字节数。
    pub rejected_bytes: u64,
    /// Runtime 调用失败数。
    pub call_failures: u64,
    /// 熔断后被阻断的类别调用数。
    pub blocked_calls: u64,
    /// 解码总耗时（纳秒）。
    pub total_decode_nanoseconds: u64,
    /// 最大单次解码耗时（纳秒）。
    pub max_decode_nanoseconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 一个会话的协议熔断与全部输出类别指标快照。
///
/// 各字段由独立的 relaxed atomic loads 组成；该快照适合诊断观察，不保证不同字段来自同一瞬间。
pub struct RuntimeForeignOutputMetricsSnapshot {
    /// 是否已经发生协议违规并熔断。
    pub protocol_failed: bool,
    /// 首次熔断计数；正常会话为零。
    pub protocol_failures: u64,
    /// 被熔断阻断的会话级调用数。
    pub blocked_session_calls: u64,
    /// 按 [`RuntimeForeignOutputKind`] 稳定索引排列的指标。
    pub(super) by_kind: [RuntimeForeignOutputMetrics; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT],
}

impl RuntimeForeignOutputMetricsSnapshot {
    /// 取指定类别的指标。
    pub const fn for_kind(self, kind: RuntimeForeignOutputKind) -> RuntimeForeignOutputMetrics {
        self.by_kind[kind.index()]
    }

    /// 判断诊断行是否至少有一项可观察活动。
    pub(super) fn has_activity(self) -> bool {
        self.protocol_failures > 0
            || self.blocked_session_calls > 0
            || self.by_kind.iter().any(|metrics| {
                metrics.accepted_payloads > 0
                    || metrics.rejected_payloads > 0
                    || metrics.call_failures > 0
                    || metrics.blocked_calls > 0
            })
    }
}

/// 创建全部类别的零初始化计数器。
pub(super) fn empty_counters() -> [RuntimeForeignOutputCounters; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT]
{
    array::from_fn(|_| RuntimeForeignOutputCounters::default())
}

/// 将平台字节数饱和转换为诊断使用的 u64。
fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// 将 Duration 纳秒数饱和转换为诊断使用的 u64。
fn duration_to_u64_nanoseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
