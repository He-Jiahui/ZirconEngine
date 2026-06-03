use crate::plugin::PluginModuleKind;

use super::NativePluginLoadReport;

impl NativePluginLoadReport {
    pub fn entry_diagnostics(&self) -> Vec<String> {
        sorted_deduped(
            self.loaded
                .iter()
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .flat_map(|report| {
                            report.diagnostics.iter().map(|message| {
                                format!("native plugin {}: {message}", plugin.plugin_id)
                            })
                        })
                })
                .chain(self.loaded.iter().flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .flat_map(|report| {
                            report
                                .behavior_validation
                                .diagnostics
                                .iter()
                                .map(|message| {
                                    format!("native plugin {}: {message}", plugin.plugin_id)
                                })
                        })
                }))
                .collect(),
        )
    }

    pub fn descriptor_diagnostics(&self) -> Vec<String> {
        sorted_deduped(
            self.loaded
                .iter()
                .filter(|plugin| plugin.descriptor.is_none())
                .map(|plugin| {
                    format!(
                        "native plugin {} has no ABI descriptor attached",
                        plugin.plugin_id
                    )
                })
                .collect(),
        )
    }

    pub fn diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(
            plugin_id,
            &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
        )
    }

    pub fn diagnostics_for_runtime_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(plugin_id, &[PluginModuleKind::Runtime])
    }

    pub fn diagnostics_for_editor_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(plugin_id, &[PluginModuleKind::Editor])
    }

    fn diagnostics_for_plugin_with_entry_kinds(
        &self,
        plugin_id: &str,
        module_kinds: &[PluginModuleKind],
    ) -> Vec<String> {
        let mut diagnostics = self
            .diagnostics
            .iter()
            .filter(|message| diagnostic_mentions_plugin(message, plugin_id))
            .cloned()
            .collect::<Vec<_>>();
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id && plugin.descriptor.is_none())
                .map(|plugin| {
                    format!(
                        "native plugin {} has no ABI descriptor attached",
                        plugin.plugin_id
                    )
                }),
        );
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id)
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .filter(|report| module_kinds.contains(&report.module_kind))
                        .flat_map(|report| {
                            report.diagnostics.iter().map(|message| {
                                format!("native plugin {}: {message}", plugin.plugin_id)
                            })
                        })
                }),
        );
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id)
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .filter(|report| module_kinds.contains(&report.module_kind))
                        .flat_map(|report| {
                            report
                                .behavior_validation
                                .diagnostics
                                .iter()
                                .map(|message| {
                                    format!("native plugin {}: {message}", plugin.plugin_id)
                                })
                        })
                }),
        );
        sorted_deduped(diagnostics)
    }
}

fn sorted_deduped(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn diagnostic_mentions_plugin(message: &str, plugin_id: &str) -> bool {
    message.contains(&format!("native plugin {plugin_id} "))
        || message.contains(&format!("native plugin {plugin_id}:"))
}
