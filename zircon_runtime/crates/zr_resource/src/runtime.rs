use serde::{Deserialize, Serialize};

use crate::{ResourceId, ResourceKind, ResourceLocator};

/// 统一的运行时资源身份与状态读取契约，不暴露具体载荷；调用端需要内容时转向管理器的类型化读取。
pub trait Resource: Send + Sync {
    fn id(&self) -> ResourceId;
    fn kind(&self) -> ResourceKind;
    fn primary_locator(&self) -> &ResourceLocator;
    fn revision(&self) -> u64;
    fn runtime_state(&self) -> RuntimeResourceState;
}

/// 描述运行时载荷的驻留/加载状态；目录 Ready 只表示内容可加载，可能仍处于 Unloaded。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeResourceState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Error,
    Reloading,
}

/// 某次查询返回的运行时信息值；它不持有管理器锁或租约，之后的发布不会自动更新此值。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRuntimeInfo {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub primary_locator: ResourceLocator,
    pub revision: u64,
    pub runtime_state: RuntimeResourceState,
}

impl Resource for ResourceRuntimeInfo {
    fn id(&self) -> ResourceId {
        self.id
    }

    fn kind(&self) -> ResourceKind {
        self.kind
    }

    fn primary_locator(&self) -> &ResourceLocator {
        &self.primary_locator
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn runtime_state(&self) -> RuntimeResourceState {
        self.runtime_state
    }
}
