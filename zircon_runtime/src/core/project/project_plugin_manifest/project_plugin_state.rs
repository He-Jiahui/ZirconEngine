use crate::core::framework::platform::RuntimeTargetMode;

use super::{ProjectPluginManifest, ProjectPluginSelection};

impl ProjectPluginManifest {
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    /// 按插件 id 原位替换或追加完整声明，保留清单顺序供编辑器和导出计划稳定显示。
    pub fn set_enabled(&mut self, selection: ProjectPluginSelection) {
        if let Some(existing) = self
            .selections
            .iter_mut()
            .find(|existing| existing.id == selection.id)
        {
            *existing = selection;
        } else {
            self.selections.push(selection);
        }
    }

    pub fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) {
        if let Some(existing) = self
            .selections
            .iter_mut()
            .find(|existing| existing.id == plugin_id)
        {
            existing.enabled = enabled;
            return;
        }
        self.selections.push(ProjectPluginSelection {
            id: plugin_id.to_string(),
            enabled,
            required: false,
            target_modes: Vec::new(),
            packaging: super::super::ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        });
    }

    /// 提供当前目标的可参与项；最终是否能注册仍交给插件目录和解析器裁决。
    pub fn enabled_for_target(
        &self,
        target: RuntimeTargetMode,
    ) -> impl Iterator<Item = &ProjectPluginSelection> {
        self.selections
            .iter()
            .filter(move |selection| selection.enabled && selection.supports_target(target))
    }
}
