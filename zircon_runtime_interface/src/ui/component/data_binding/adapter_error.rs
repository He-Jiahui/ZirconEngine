//! 宿主组件适配器返回的可序列化错误值；域、字段路径和诊断文字均由错误值自行持有。
//! 此模块定义错误分类与显示格式，实际目标解析、输入校验及数据写入由消费端完成。

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::component::{UiComponentEventKind, UiValueKind};

/// 组件事件在目标路由、输入检查或宿主写入阶段产生的错误。
///
/// # 使用与生命周期
/// 适配器选择对应变体并填入上下文，调用者可显示或序列化该独立值；它不借用宿主状态。
/// 字符串字段只保存诊断信息，不解析路径，也不执行回滚；错误显示由派生实现使用各变体格式生成。
#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentAdapterError {
    /// 目标域未被当前消费端支持。
    #[error("unsupported Runtime UI component target domain {domain}")]
    UnsupportedTargetDomain { domain: String },
    /// 域内字段路径不受当前消费端支持。
    #[error("unsupported Runtime UI component target path {path} in domain {domain}")]
    UnsupportedTargetPath { domain: String, path: String },
    /// 目标缺少适配器所需的具名数据来源。
    #[error("Runtime UI component target {domain}:{path} is missing source {source_name}")]
    MissingSource {
        domain: String,
        path: String,
        source_name: String,
    },
    /// 目标不接收给定的类型化事件种类。
    #[error("component target {domain}:{path} does not support event {event_kind:?}")]
    UnsupportedEvent {
        domain: String,
        path: String,
        event_kind: UiComponentEventKind,
    },
    /// 载荷值种类与目标要求不符，保留预期值和实际值供诊断。
    #[error(
        "invalid value kind {actual:?} for component target {domain}:{path}; expected {expected:?}"
    )]
    InvalidValueKind {
        domain: String,
        path: String,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    /// 输入被目标规则拒绝，原因由适配器提供。
    #[error("component target {domain}:{path} rejected input: {reason}")]
    RejectedInput {
        domain: String,
        path: String,
        reason: String,
    },
    /// 宿主处理失败，保留目标和原因；错误也可能来自访问或校验阶段，不表示已开始写入或已回滚。
    #[error("component target {domain}:{path} mutation failed: {reason}")]
    HostMutation {
        domain: String,
        path: String,
        reason: String,
    },
}
