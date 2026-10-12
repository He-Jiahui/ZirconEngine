use crate::plugin::PluginFeatureDependency;

use super::super::{owner, primary_count};

/// 单次功能清单审查的主依赖计数状态；不可跨功能复用。
#[derive(Default)]
pub(super) struct FeaturePrimaryDependencyRows {
    primary_count: usize,
}

impl FeaturePrimaryDependencyRows {
    pub(super) fn validate(
        &mut self,
        dependency: &PluginFeatureDependency,
        owner_plugin_id: &str,
        diagnostics: &mut Vec<String>,
    ) {
        self.primary_count += owner::validate_runtime_plugin_feature_primary_dependency_owner(
            dependency,
            owner_plugin_id,
            diagnostics,
        );
    }

    /// 消费状态完成集合级检查，避免把未结束的行扫描结果当作最终结论。
    pub(super) fn validate_count(self, diagnostics: &mut Vec<String>) {
        primary_count::validate_runtime_plugin_feature_primary_dependency_count(
            self.primary_count,
            diagnostics,
        );
    }
}
