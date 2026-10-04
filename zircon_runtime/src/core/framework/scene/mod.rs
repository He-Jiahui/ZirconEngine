use std::fmt;
use std::sync::Arc;
use std::time::Instant;

mod component_type_descriptor;
mod entity_path;
mod level_manager_error;
mod level_summary;
mod mobility;
mod module_identity;
pub mod physics;
mod property_value;
mod resource;
mod system_stage;
mod world_handle;

pub type EntityId = u64;
pub type NodeId = EntityId;

/// 场景资产异步保存的终态；同一路径的新代请求可取代尚未开始的旧请求。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactTerminal {
    Succeeded,
    Failed { code: &'static str },
    DeadlineBeforeStart,
    CancelledBeforeStart,
    Superseded { successor: u64 },
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactWaitResult {
    Terminal(SceneArtifactTerminal),
    ObserverTimedOut,
}

/// 保存请求的只读观察句柄。调用者可按代次识别被取代的任务，观察超时不会取消写入。
pub trait SceneArtifactTicket: Send + Sync + fmt::Debug + 'static {
    fn generation(&self) -> u64;
    fn terminal(&self) -> Option<SceneArtifactTerminal>;
    fn wait_until(&self, deadline: Instant) -> SceneArtifactWaitResult;
}

pub use component_type_descriptor::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
pub use entity_path::{ComponentPropertyPath, EntityPath, PathParseError};
pub use level_manager_error::LevelManagerError;
pub use level_summary::LevelSummary;
pub use mobility::Mobility;
pub use module_identity::SCENE_MODULE_NAME;
pub(crate) use property_value::ScenePropertyEntry;
pub use property_value::ScenePropertyValue;
pub use resource::SceneResource;
pub use system_stage::SystemStage;
pub use world_handle::WorldHandle;

/// 向宿主暴露关卡句柄与资产 I/O；实现负责绑定当前项目代次并保持场景世界所有权。
/// 保存返回票据后仍须观察终态，不能把请求被接纳视为文件已落盘。
pub trait LevelManager: Send + Sync {
    fn create_default_level_handle(&self) -> Result<WorldHandle, LevelManagerError>;
    fn level_exists(&self, handle: WorldHandle) -> bool;
    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary>;
    fn load_level_asset(
        &self,
        project_root: &str,
        uri: &str,
    ) -> Result<WorldHandle, LevelManagerError>;
    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<Arc<dyn SceneArtifactTicket>, LevelManagerError>;
}
