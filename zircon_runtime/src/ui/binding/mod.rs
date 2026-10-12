//! 绑定执行前的类型与版本校验，以及执行后的来源/目标报告；真实模型访问和表面变更由各自 owner 完成。

#[path = "conversion_registry.rs"]
mod conversion_registry;
#[path = "model_schema_registry.rs"]
mod model_schema_registry;
#[path = "update_report.rs"]
mod update_report;

pub use conversion_registry::{
    UiBindingConversionFunction, UiBindingConversionRegistry, UiBindingConversionRegistryError,
};
pub use model_schema_registry::{UiModelSchemaRegistrationError, UiModelSchemaRegistry};
pub(crate) use update_report::reflected_property_source_kind;
pub use update_report::{
    binding_update_report, component_state_value_update,
    component_state_value_update_with_source_kind, reflected_property_update,
    reflected_property_update_with_source_kind, rejected_widget_alias_update,
    retained_attribute_update, runtime_state_update_with_source_kind,
};
