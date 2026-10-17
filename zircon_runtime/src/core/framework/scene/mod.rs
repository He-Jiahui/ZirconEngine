use std::fmt;
use std::sync::Arc;
use web_time::Instant;

#[path = "component_type_descriptor/mod.rs"]
mod component_type_descriptor;
#[path = "entity_path.rs"]
mod entity_path;
#[path = "level_manager_error.rs"]
mod level_manager_error;
#[path = "level_summary.rs"]
mod level_summary;
#[path = "mobility.rs"]
mod mobility;
#[path = "module_identity.rs"]
mod module_identity;
#[path = "physics/mod.rs"]
pub mod physics;
#[path = "property_value.rs"]
mod property_value;
#[path = "resource.rs"]
mod resource;
#[path = "system_stage.rs"]
mod system_stage;
#[path = "world_handle.rs"]
mod world_handle;

/// World 内使用的数值实体键，别名本身不绑定 World 身份或实体寿命。
pub type EntityId = u64;
/// 层级节点使用同一实体键类型，不引入独立的编号或所有权。
pub type NodeId = EntityId;

/// 场景资产异步保存的终态；同一路径的新代请求可取代尚未开始的旧请求。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactTerminal {
    /// 保存工作函数执行成功，该值沿用提供方的成功判定。
    Succeeded,
    /// 保存工作失败，`code` 是提供方返回的静态错误代码。
    Failed { code: &'static str },
    /// 底层 I/O 工作在启动前达到执行期限。
    DeadlineBeforeStart,
    /// 底层 I/O 工作在启动前被取消，观察票据自身不提供取消操作。
    CancelledBeforeStart,
    /// 未开始的旧请求被替代，`successor` 是替代请求的代次。
    Superseded { successor: u64 },
    /// 底层 I/O 队列报告关闭终态。
    Shutdown,
}

/// 一次有期限观察的结果，观察超时与保存任务终态分别表示。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactWaitResult {
    /// 已经观察到保存请求的终态。
    Terminal(SceneArtifactTerminal),
    /// 本次等待到期，调用者仍可继续用同一票据观察任务。
    ObserverTimedOut,
}

/// 保存请求的只读观察句柄。调用者可按代次识别被取代的任务，观察超时不会取消写入。
pub trait SceneArtifactTicket: Send + Sync + fmt::Debug + 'static {
    /// 返回当前请求的代次，可与被替代终态记录的后继代次对照。
    fn generation(&self) -> u64;
    /// 读取已发布的终态，尚未完成时返回 `None`，不等待工作结束。
    fn terminal(&self) -> Option<SceneArtifactTerminal>;
    /// 等待终态或单调时钟的绝对 `deadline`，返回本次观察结果。
    /// 观察超时不取消工作；票据接口的实现参见 `BoundedKeyedIoTicket` 的场景适配。
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
    /// 创建并登记默认关卡，成功返回句柄键；创建或句柄空间错误以领域错误返回。
    /// 句柄不拥有 World，实际登记寿命由实现管理，默认实现见 `DefaultLevelManager`。
    fn create_default_level_handle(&self) -> Result<WorldHandle, LevelManagerError>;
    /// 按 `handle` 检查当前登记状态，结果只描述本次查询，不建立后续访问的存活保证。
    fn level_exists(&self, handle: WorldHandle) -> bool;
    /// 按 `handle` 读取拥有型概览，未登记时返回 `None`。
    /// 返回值不借用管理器，字段语义见 `LevelSummary`，后续查询仍需使用有效句柄。
    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary>;
    /// 从项目内的资源定位符加载并登记关卡，返回句柄或加载链错误。
    ///
    /// # 参数与约束
    /// `project_root` 指定预期活动项目根，`uri` 为资源定位符，参数只在调用期借用。
    /// 默认实现先取得活动项目快照并核对根路径，随后解析定位符、加载 World 和登记关卡。
    fn load_level_asset(
        &self,
        project_root: &str,
        uri: &str,
    ) -> Result<WorldHandle, LevelManagerError>;
    /// 为指定关卡接纳保存请求，成功返回可共享的观察票据。
    ///
    /// # 参数、返回值与生命周期
    /// `handle` 指向已登记关卡，`project_root` 与 `uri` 指定项目和保存目标。
    /// 默认实现捕获 World、项目及定位符快照供后台工作使用，票据不借用这些字符串参数。
    /// 同步错误只描述接纳链；序列化与写入错误随后由终态报告，等待超时也不会撤销工作。
    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<Arc<dyn SceneArtifactTicket>, LevelManagerError>;
}
