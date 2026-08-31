mod validation;

pub(crate) use validation::component_param_kind;
pub use validation::{collect_asset_binding_report, validate_asset_bindings};
