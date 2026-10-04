use crate::core::framework::net::RpcPayloadSchema;
use crate::scene::SystemStage;

use super::VmCallbackHandle;

/// Schedule stage supported by VM system dispatchers.
/// 阶段可显式转换为运行时 `SystemStage`；VM 注册表自身按本枚举分组活动贡献。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VmSystemStage {
    /// Fixed-step simulation stage.
    FixedUpdate,
    /// Variable-step gameplay stage.
    Update,
    /// End-of-frame maintenance stage.
    Last,
}

impl VmSystemStage {
    /// All stages that receive a fixed host dispatcher.
    pub const ALL: [Self; 3] = [Self::FixedUpdate, Self::Update, Self::Last];

    /// Maps the VM-facing stage to the runtime scheduler stage.
    pub const fn system_stage(self) -> SystemStage {
        match self {
            Self::FixedUpdate => SystemStage::FixedUpdate,
            Self::Update => SystemStage::Update,
            Self::Last => SystemStage::Last,
        }
    }

    /// Parses the stable script spelling of a system stage.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fixed_update" => Some(Self::FixedUpdate),
            "update" => Some(Self::Update),
            "last" => Some(Self::Last),
            _ => None,
        }
    }

    /// Returns the stable script spelling of this stage.
    pub const fn label(self) -> &'static str {
        match self {
            Self::FixedUpdate => "fixed_update",
            Self::Update => "update",
            Self::Last => "last",
        }
    }
}

/// Active VM system contribution published to the scheduler dispatcher.
/// 记录只描述已发布贡献；实际回调仍通过 `VmCallbackHandle` 解析。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmSystemRegistration {
    /// Package-local stable system identifier.
    pub id: String,
    /// Scheduler stage selected by the package.
    pub stage: VmSystemStage,
    /// Stable callback target invoked by the dispatcher.
    pub callback: VmCallbackHandle,
}

/// VM behavior-node contribution consumed by the AI plugin adapter.
/// 节点适配器使用稳定 id 建目录，并在执行时解析 callback。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmBehaviorNodeRegistration {
    /// Stable catalog identifier.
    pub id: String,
    /// Human-readable editor/catalog label.
    pub display_name: String,
    /// Stable callback target invoked by the node adapter.
    pub callback: VmCallbackHandle,
}

/// VM RPC-handler contribution consumed by the networking plugin adapter.
/// payload schema 与 callback 一起构成网络层可验证的入口描述。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmRpcHandlerRegistration {
    /// Stable RPC identifier.
    pub id: String,
    /// Shared reflection-backed payload schema consumed by the networking layer.
    pub payload_schema: RpcPayloadSchema,
    /// Stable callback target invoked for matching RPC messages.
    pub callback: VmCallbackHandle,
}

/// VM editor-operation contribution consumed by the editor adapter.
/// 记录三段 operation 标识与回调；注册表按标识保存贡献，管理器提供活动快照查询。
// TODO: [CR-VM-HOST-EDITOR-CONSUMER-0001] 当前仅证实注册表快照和管理器查询，未定位编辑器执行消费者；下一步沿编辑器操作入口确认匹配、能力校验及回调分发接入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmEditorOperationRegistration {
    /// Three-segment operation identifier (`Domain.Group.Action`).
    pub operation: String,
    /// Stable callback target invoked by the operation adapter.
    pub callback: VmCallbackHandle,
}
