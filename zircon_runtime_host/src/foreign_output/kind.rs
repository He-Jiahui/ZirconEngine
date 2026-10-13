//! Stable metric categories for host-consumed runtime output.
//!
//! 本模块定义 Host 消费的 Runtime 输出类别与稳定指标索引。
//!
//! `RuntimeForeignOutputState` 用同一枚举选择预算、熔断计数和诊断字段；数组顺序必须与
//! `ALL`、`index` 和诊断输出保持一致，调用方包括 App runtime session 与 Editor gateway。

pub(super) const RUNTIME_FOREIGN_OUTPUT_KIND_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Runtime ABI 输出在 Host 侧的协议类别。
pub enum RuntimeForeignOutputKind {
    /// 会话握手或帧级协议结果。
    SessionProtocol,
    /// Runtime 推送给 Host 的请求队列。
    HostRequests,
    /// Profile 控制与诊断响应。
    ProfileResponse,
    /// 操作提交或查询返回的结果。
    OperationResult,
    /// 插件事件投递批次。
    PluginEvents,
    /// World 查询返回结果。
    WorldQuery,
    /// World 失效事实批次。
    WorldInvalidations,
}

impl RuntimeForeignOutputKind {
    /// 按稳定诊断顺序列出全部类别。
    pub(super) const ALL: [Self; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT] = [
        Self::SessionProtocol,
        Self::HostRequests,
        Self::ProfileResponse,
        Self::OperationResult,
        Self::PluginEvents,
        Self::WorldQuery,
        Self::WorldInvalidations,
    ];

    /// 返回该类别在共享计数数组中的索引。
    pub(super) const fn index(self) -> usize {
        match self {
            Self::SessionProtocol => 0,
            Self::HostRequests => 1,
            Self::ProfileResponse => 2,
            Self::OperationResult => 3,
            Self::PluginEvents => 4,
            Self::WorldQuery => 5,
            Self::WorldInvalidations => 6,
        }
    }

    /// 返回诊断行和错误文本使用的稳定标签。
    pub const fn label(self) -> &'static str {
        match self {
            Self::SessionProtocol => "session_protocol",
            Self::HostRequests => "host_requests",
            Self::ProfileResponse => "profile_response",
            Self::OperationResult => "operation_result",
            Self::PluginEvents => "plugin_events",
            Self::WorldQuery => "world_query",
            Self::WorldInvalidations => "world_invalidations",
        }
    }
}
