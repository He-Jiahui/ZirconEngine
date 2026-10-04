//! Service and module lifecycle vocabulary.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::contexts::ModuleContext;
use super::error::CoreResult;

/// Immediate 服务参加模块启动计划；Lazy 服务留到按需解析时初始化。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupMode {
    Immediate,
    Lazy,
}

/// 模块初始化层级按声明顺序递增；依赖不能位于更晚层级。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitLevel {
    Kernel,
    Services,
    Scene,
    Editor,
    Post,
}

impl InitLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kernel => "Kernel",
            Self::Services => "Services",
            Self::Scene => "Scene",
            Self::Editor => "Editor",
            Self::Post => "Post",
        }
    }
}

/// 模块运行阶段；停用先进入 Stopping，清理及服务卸载完成后才到 Unloaded。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Registered,
    Initializing,
    Running,
    Stopping,
    Unloaded,
}

/// 服务依赖层级为 Driver、Manager、Plugin，依赖只能指向同层或更早层。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceKind {
    Driver,
    Manager,
    Plugin,
}

impl ServiceKind {
    pub fn from_registry_segment(value: &str) -> Option<Self> {
        Self::from_registry_segment_bytes(value.as_bytes())
    }

    pub(crate) fn from_registry_segment_bytes(value: &[u8]) -> Option<Self> {
        match value {
            b"Driver" => Some(Self::Driver),
            b"Manager" => Some(Self::Manager),
            b"Plugin" => Some(Self::Plugin),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Driver => "Driver",
            Self::Manager => "Manager",
            Self::Plugin => "Plugin",
        }
    }
}

/// 激活依次构建、轮询 ready、finish；失败回滚或停用路径调用 cleanup。
pub trait ModuleLifecycle: Send + Sync {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        Ok(true)
    }

    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    /// 默认实现忽略 deadline 并调用 cleanup；需要限时清理的模块应覆盖此方法。
    fn cleanup_until(&self, context: &ModuleContext, deadline: Instant) -> CoreResult<()> {
        let _ = deadline;
        self.cleanup(context)
    }
}

#[derive(Debug, Default)]
pub struct NoopModuleLifecycle;

impl ModuleLifecycle for NoopModuleLifecycle {}
