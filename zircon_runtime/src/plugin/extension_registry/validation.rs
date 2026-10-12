//! 注册边界的清单验证集中于此，保证进入可撤销表的标识与描述符满足各自约束。
#[path = "validation/component.rs"]
mod component;
#[path = "validation/plugin_event_catalog.rs"]
mod plugin_event_catalog;
#[path = "validation/plugin_option.rs"]
mod plugin_option;
#[path = "validation/runtime_core.rs"]
mod runtime_core;
#[path = "validation/token.rs"]
mod token;

pub(super) use component::validate_component_type_descriptor;
#[cfg(feature = "ui")]
pub(super) use component::validate_ui_component_descriptor;
pub(super) use plugin_event_catalog::validate_plugin_event_catalog_manifest;
pub(super) use plugin_option::validate_plugin_option_manifest;
pub(super) use runtime_core::{validate_manager_plugin_id, validate_module_descriptor};
use token::{is_lowercase_plugin_package_id, is_lowercase_plugin_token};
