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
