//! 组件与宿主之间交换的事件信封、绑定目标、数据源描述符、适配结果和错误。
//!
//! 这些类型拥有传递的数据；目标域解析、源数据写入和投影应用属于宿主消费端。
//! 信封反序列化通过临时线格式检查事件种类与载荷，其他描述符构建器只记录配置。

#[path = "adapter_error.rs"]
mod adapter_error;
#[path = "adapter_result.rs"]
mod adapter_result;
#[path = "binding_target.rs"]
mod binding_target;
#[path = "data_source.rs"]
mod data_source;
#[path = "event_envelope.rs"]
mod event_envelope;
#[path = "projection_patch.rs"]
mod projection_patch;

pub use adapter_error::UiComponentAdapterError;
pub use adapter_result::UiComponentAdapterResult;
pub use binding_target::UiComponentBindingTarget;
pub use data_source::{
    UiComponentDataSourceDescriptor, UiComponentDataSourceFieldDescriptor,
    UiComponentDataSourceFieldOption, UiComponentDataSourceKind,
};
pub use event_envelope::UiComponentEventEnvelope;
pub use projection_patch::UiComponentProjectionPatch;
