mod presence;
mod strategies;

use crate::core::framework::project::ExportPackagingStrategy;

use self::{
    presence::validate_runtime_plugin_default_packaging_presence,
    strategies::validate_runtime_plugin_default_packaging_strategies,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_default_packaging(
    owner: &str,
    default_packaging: &[ExportPackagingStrategy],
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_default_packaging_presence(owner, default_packaging, diagnostics);
    validate_runtime_plugin_default_packaging_strategies(owner, default_packaging, diagnostics);
}
