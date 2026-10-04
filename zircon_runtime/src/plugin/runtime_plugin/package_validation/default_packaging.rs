//! 描述符和包清单共用此约束，保证导出策略至少有一个候选且没有重复声明；实际导出仍按项目选择决定。
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
