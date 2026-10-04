use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;

use super::ProjectPluginSelection;

fn default_runtime_crate_name(plugin_id: &str) -> String {
    const PREFIX: &str = "zircon_plugin_";
    const SUFFIX: &str = "_runtime";

    let mut name = String::with_capacity(PREFIX.len() + plugin_id.len() + SUFFIX.len());
    name.push_str(PREFIX);
    for character in plugin_id.chars() {
        name.push(if character == '-' { '_' } else { character });
    }
    name.push_str(SUFFIX);
    name
}

impl ProjectPluginSelection {
    /// 空目标列表表示不限制运行目标；非空列表只接受其中列出的目标。
    pub fn supports_target(&self, target: RuntimeTargetMode) -> bool {
        self.target_modes.is_empty() || self.target_modes.contains(&target)
    }

    /// 显式 crate 名按 builtin_ 前缀判断；未指定时只有内置 UI 插件 ID 使用运行时内建域。
    pub fn is_runtime_builtin_domain(&self) -> bool {
        match self.runtime_crate.as_deref() {
            Some(crate_name) => crate_name.starts_with("builtin_"),
            None => self.id == RuntimePluginId::Ui.key(),
        }
    }

    /// 返回显式配置的 crate 名；未配置时按插件 ID 生成 zircon_plugin_*_runtime 名称。
    pub fn runtime_crate_name(&self) -> String {
        self.runtime_crate
            .clone()
            .unwrap_or_else(|| default_runtime_crate_name(&self.id))
    }
}

#[cfg(test)]
#[path = "project_plugin_selection_access/tests/single_buffer_runtime_crate_name_tests.rs"]
mod single_buffer_runtime_crate_name_tests;

#[cfg(test)]
#[path = "tests/project_plugin_selection_access_runtime_builtin_domain_tests.rs"]
mod runtime_builtin_domain_tests;
