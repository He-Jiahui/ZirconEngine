//! 将原始与加载入口诊断按插件和模块种类投影，供 live host、编辑器状态和导出报告消费。

use super::NativePluginLoadReport;

impl NativePluginLoadReport {
    pub fn entry_diagnostics(&self) -> Vec<String> {
        self.projection().entry_diagnostics().to_vec()
    }

    pub fn descriptor_diagnostics(&self) -> Vec<String> {
        self.projection().descriptor_diagnostics().to_vec()
    }

    /// 返回该插件的原始、描述符与入口诊断；不包含运行时注册时才读取 shader 文件的错误。
    pub fn diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.projection().diagnostics_for_plugin(plugin_id)
    }

    pub fn diagnostics_for_runtime_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.projection().runtime_diagnostics_for_plugin(plugin_id)
    }

    pub fn diagnostics_for_editor_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.projection().editor_diagnostics_for_plugin(plugin_id)
    }
}

fn diagnostic_mentions_plugin(message: &str, plugin_id: &str) -> bool {
    mentioned_plugin_ids(message).any(|mentioned| mentioned == plugin_id)
}

// TODO: [CR-PLUGIN-NATIVE-0502] 确认一条原始诊断同时含多个插件名前缀时应归属谁；当前文本扫描会把整条消息投给每个命中的 ID，缺少结构化归属字段；下一步检查错误消息来源并增加多 ID 投影契约测试。
/// 从加载器既有诊断文本中恢复插件 ID，供原始错误归属索引使用；它依赖消息包含固定前缀与分隔符。
pub(super) fn mentioned_plugin_ids(message: &str) -> impl Iterator<Item = &str> {
    const NATIVE_PLUGIN_PREFIX: &str = "native plugin ";

    message
        .match_indices(NATIVE_PLUGIN_PREFIX)
        .filter_map(|(offset, _)| {
            let suffix = &message[offset + NATIVE_PLUGIN_PREFIX.len()..];
            let boundary = suffix
                .bytes()
                .position(|byte| matches!(byte, b' ' | b':'))?;
            (boundary > 0).then_some(&suffix[..boundary])
        })
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
