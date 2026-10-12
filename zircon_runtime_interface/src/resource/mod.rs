//! 资源接口共有的身份、定位符、句柄、记录和事件 DTO，供 Runtime、Editor 与资源基础 crate 交换同一套契约。
//!
//! # 入口与职责
//!
//! [`ResourceLocator`] 建立逻辑地址，[`AssetReference`] 保留资产身份对，[`ResourceHandle`] 与 [`UntypedResourceHandle`] 传递资源身份，记录和事件描述管理器发布的数据。
//!
//! # 所有权与边界
//!
//! 本模块定义可共享的数据类型，不执行注册、加载或状态提交。资源基础 crate 与 Runtime 的资源门面导出这些类型；读取和提交端负责注册关系、载荷类型、修订和事件日志的检查。
//!
//! # 参考
//!
//! [`ResourceRecord`]、[`ResourceState`]、[`ResourceEvent`]。
//! Shared resource identity and status DTOs for editor/runtime contracts.

#[path = "asset_reference.rs"]
mod asset_reference;
#[path = "asset_uuid.rs"]
mod asset_uuid;
#[path = "diagnostic.rs"]
mod diagnostic;
#[path = "locator.rs"]
mod locator;
#[path = "marker.rs"]
mod marker;
#[path = "resource_event.rs"]
mod resource_event;
#[path = "resource_event_kind.rs"]
mod resource_event_kind;
#[path = "resource_handle.rs"]
mod resource_handle;
#[path = "resource_id.rs"]
mod resource_id;
#[path = "resource_record.rs"]
mod resource_record;
#[path = "stable_uuid.rs"]
mod stable_uuid;
#[path = "state.rs"]
mod state;
#[path = "untyped_handle.rs"]
mod untyped_handle;

pub use asset_reference::AssetReference;
pub use asset_uuid::AssetUuid;
pub use diagnostic::{ResourceDiagnostic, ResourceDiagnosticSeverity};
pub use locator::{ResourceLocator, ResourceLocatorError, ResourceScheme};
pub use marker::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    MeshMarker, ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker,
    PrefabMarker, ResourceKind, ResourceMarker, SceneMarker, ShaderMarker, SoundMarker,
    TerrainLayerStackMarker, TerrainMarker, TextureMarker, TileMapMarker, TileSetMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
pub use resource_event::ResourceEvent;
pub use resource_event_kind::ResourceEventKind;
pub use resource_handle::ResourceHandle;
pub use resource_id::ResourceId;
pub use resource_record::ResourceRecord;
pub use stable_uuid::STABLE_UUID_ALGORITHM_VERSION;
pub use state::ResourceState;
pub use untyped_handle::UntypedResourceHandle;

pub(crate) use stable_uuid::stable_uuid_from_components;
