//! 同步成形请求的字节阈值分类与标量工作回执。
//!
//! 会话在缓存未命中、调用后端前记账；并行批处理对去重后加入待处理队列的请求记账。
//! 阈值只用于分类，本模块不执行调度，也不授权切分语义范围、延期或内存准入。
//! 回执可包含后端失败或延期的尝试，不证明产生了成功产物；会话负责在帧开始时重置。
//! 两种值均可复制且不持有请求借用、字体或后端资源，报告副本不会延长这些资源的生命周期。

const DEFAULT_MAX_INLINE_INPUT_BYTES: usize = 64 * 1024;

/// Scheduling threshold for one synchronous shaping request.
///
/// This budget classifies work; it is never a source-line, script-run, or cluster boundary.
/// Classification never authorizes slicing a request. Until a typed deferred outcome exists,
/// the synchronous fallback must retain the complete semantic context.
///
/// 以完整请求的 UTF-8 字节数比较调度分类阈值。
///
/// # 约束与传递
/// 构造器仅接纳非零阈值；默认值为 64 KiB，恰等于阈值仍归入阈值内请求。
/// 值按副本传递，只做分类，不执行请求，也不限制调用方的内存或输出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextShapingWorkBudget {
    max_inline_input_bytes: usize,
}

/// Aggregate receipt for complete requests that actually reached a shaping backend.
///
/// Oversized requests remain synchronous until a typed deferred work-unit owner exists. The
/// counters therefore describe observed work without authorizing a semantic split or defer.
///
/// 累计调用方登记的完整同步请求，不持有输入内容。
///
/// # 字段与生命周期
/// 每次登记只归入其中一类请求计数，累计字节数采用饱和加法；最大字节数保留单次请求的最大值。
/// 重置由持有者以默认值替换；最大输入字节数不表示峰值驻留内存。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingWorkReport {
    pub(crate) inline_request_count: usize,
    pub(crate) oversized_synchronous_request_count: usize,
    pub(crate) synchronous_input_bytes: usize,
    pub(crate) max_synchronous_input_bytes: usize,
}

impl TextShapingWorkBudget {
    /// 为调用方选择非零的输入字节阈值。
    ///
    /// # 参数与返回
    /// `max_inline_input_bytes` 是单次完整请求的 UTF-8 字节数阈值；为零时返回 `None`，否则返回分类值。
    pub(crate) const fn new(max_inline_input_bytes: usize) -> Option<Self> {
        if max_inline_input_bytes == 0 {
            return None;
        }
        Some(Self {
            max_inline_input_bytes,
        })
    }

    pub(crate) const fn max_inline_input_bytes(self) -> usize {
        self.max_inline_input_bytes
    }

    /// 判断完整输入的 UTF-8 字节数是否严格超过阈值；相等时返回 `false`。
    /// 接收预算副本，不修改调用方持有的其他副本。
    pub(crate) const fn exceeds_inline_threshold(self, input_bytes: usize) -> bool {
        input_bytes > self.max_inline_input_bytes
    }
}

/// 使用 64 KiB 阈值，不预留存储或启动成形任务。
impl Default for TextShapingWorkBudget {
    fn default() -> Self {
        Self {
            max_inline_input_bytes: DEFAULT_MAX_INLINE_INPUT_BYTES,
        }
    }
}

impl TextShapingWorkReport {
    /// 由调用方在同步路径登记一次完整请求的工作量。
    ///
    /// # 参数与后效
    /// `budget` 按值给出分类阈值；`input_bytes` 是完整输入的 UTF-8 字节数。
    /// `self` 原地更新：一类请求计数与总字节数饱和增加，单次最大值取较大者。
    /// 先于后端调用登记时，后续失败或延期也保留本次工作量；本方法不观察结果。
    pub(crate) fn record_synchronous_request(
        &mut self,
        budget: TextShapingWorkBudget,
        input_bytes: usize,
    ) {
        if budget.exceeds_inline_threshold(input_bytes) {
            self.oversized_synchronous_request_count =
                self.oversized_synchronous_request_count.saturating_add(1);
        } else {
            self.inline_request_count = self.inline_request_count.saturating_add(1);
        }
        self.synchronous_input_bytes = self.synchronous_input_bytes.saturating_add(input_bytes);
        self.max_synchronous_input_bytes = self.max_synchronous_input_bytes.max(input_bytes);
    }

    /// 将另一个报告副本并入当前累计值。
    ///
    /// # 参数与后效
    /// `other` 按值提供四项标量；三个累计值饱和相加，单次最大值取两者较大者。
    /// 方法只修改 `self`；报告可复制，调用方持有的其他副本不会被清空。
    pub(crate) fn merge(&mut self, other: Self) {
        self.inline_request_count = self
            .inline_request_count
            .saturating_add(other.inline_request_count);
        self.oversized_synchronous_request_count = self
            .oversized_synchronous_request_count
            .saturating_add(other.oversized_synchronous_request_count);
        self.synchronous_input_bytes = self
            .synchronous_input_bytes
            .saturating_add(other.synchronous_input_bytes);
        self.max_synchronous_input_bytes = self
            .max_synchronous_input_bytes
            .max(other.max_synchronous_input_bytes);
    }
}

#[cfg(test)]
#[path = "tests/work_budget.rs"]
mod tests;
