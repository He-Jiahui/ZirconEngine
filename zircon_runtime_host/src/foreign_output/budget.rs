//! Per-output resource budgets enforced by the host.
//!
//! Host 为每类 Runtime 输出执行资源预算。
//!
//! 字节、条目、JSON 嵌套深度和解码时限共同约束 ABI 外来数据；类型化条目数由调用方依据
//! Interface DTO 计算，profile 响应另在类型化反序列化前检查已知集合。

use std::time::Duration;
use zircon_runtime_interface::{ZrRuntimePayloadLimitV1, ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1};

use super::RuntimeForeignOutputError;

pub const RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH: usize =
    ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// JSON 预检所采用的输出专用路径策略。
pub(super) enum RuntimeForeignOutputPreflight {
    /// 只按整份 JSON 图与类型化条目总数限制。
    None,
    /// 额外按 profile 文件数和已知类型化集合数提前拒绝。
    ProfileFiles,
}

/// 一类外来输出的解码上限。
///
/// ## 使用时机与生命周期
///
/// App 和 Editor 在处理 ABI 返回的输出时选择预算；值本身可复制，不持有 Runtime 资源。
/// `from_interface` 保留共享 ABI 契约的上限，`new` 供专用路径和测试指定各项预算上限。
///
/// ## 约束
///
/// 只有 `allow_empty` 被设置后，空输出才代表合法的空页。Profile 文件预检策略是私有的，
/// 仅由 profile 响应预算启用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeForeignOutputBudget {
    /// 序列化字节数上限；与 Interface `ZrRuntimePayloadLimitV1` 一致。
    pub(super) max_encoded_bytes: usize,
    /// 类型化数据项总数上限。
    pub(super) max_items: usize,
    /// 从预检开始到类型化条目验证完成的时间上限。
    pub(super) max_decode_time: Duration,
    /// 是否允许 ABI 返回规范空结果。
    pub(super) allow_empty: bool,
    /// 是否应用 profile 专属的已知集合预检。
    pub(super) preflight: RuntimeForeignOutputPreflight,
}

impl RuntimeForeignOutputBudget {
    /// 从 Interface 声明的 ABI 输出限制建立预算。
    ///
    /// ## 参数
    ///
    /// `limit` 提供字节、条目、处理时限和空输出约定。
    ///
    /// ## 返回值
    ///
    /// 返回不含 profile 专属预检的预算；该策略由相应的内部常量单独开启。
    pub const fn from_interface(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            max_encoded_bytes: limit.max_encoded_bytes,
            max_items: limit.max_items,
            max_decode_time: Duration::from_micros(limit.max_processing_time_micros),
            allow_empty: limit.allow_empty,
            preflight: RuntimeForeignOutputPreflight::None,
        }
    }

    /// 建立调用方指定的测试或专用路径预算。
    ///
    /// ## 参数
    ///
    /// `max_encoded_bytes` 和 `max_items` 分别给出编码字节与类型化条目上限；`max_decode_time`
    /// 限制从 JSON 预检开始到类型化条目验证结束的总耗时。
    ///
    /// ## 返回值
    ///
    /// 返回带指定上限的预算，默认拒绝空输出，并只执行普通 JSON 图及类型化条目检查。
    pub const fn new(
        max_encoded_bytes: usize,
        max_items: usize,
        max_decode_time: Duration,
    ) -> Self {
        Self {
            max_encoded_bytes,
            max_items,
            max_decode_time,
            allow_empty: false,
            preflight: RuntimeForeignOutputPreflight::None,
        }
    }

    /// 将空 ABI 输出标记为合法结果；解码器以 `Ok(None)` 表示该空页。
    pub const fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// 为 profile 响应启用已知集合的类型化预检。
    ///
    /// 预检依据 Interface DTO 的 map 键或 sequence 字段次序识别文件和集合，忽略未知嵌套
    /// 字段中的同名键；具体映射由 `decode.rs` 的 `JsonItemContext` 维护。
    pub(super) const fn preflight_profile_files(mut self) -> Self {
        self.preflight = RuntimeForeignOutputPreflight::ProfileFiles;
        self
    }

    /// 返回本预算允许的解码时间。
    pub const fn max_decode_time(self) -> Duration {
        self.max_decode_time
    }

    /// 检查从解码开始计量的耗时是否不超过上限。
    ///
    /// `elapsed` 是调用方测得的处理耗时，`operation` 用于构造诊断；边界含等号，超时返回
    /// 协议违规。
    pub fn validate_decode_duration(
        self,
        elapsed: Duration,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if elapsed <= self.max_decode_time {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget: observed {} microseconds; maximum is {} microseconds",
            elapsed.as_micros(),
            self.max_decode_time.as_micros()
        )))
    }

    /// 检查 ABI 返回的编码长度是否在字节上限内；超限时返回协议违规。
    pub(super) fn validate_encoded_len(
        self,
        encoded_len: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if encoded_len <= self.max_encoded_bytes {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {encoded_len} encoded bytes; maximum is {}",
            self.max_encoded_bytes
        )))
    }

    /// 检查调用方按 Interface DTO 结构计算的类型化条目数；超限时返回协议违规。
    pub(super) fn validate_item_count(
        self,
        item_count: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if item_count <= self.max_items {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {item_count} items; maximum is {}",
            self.max_items
        )))
    }
}
