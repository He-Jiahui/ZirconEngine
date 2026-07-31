use super::NativePluginLoadReport;

impl NativePluginLoadReport {
    pub fn entry_diagnostics(&self) -> Vec<String> {
        self.projection().entry_diagnostics().to_vec()
    }

    pub fn descriptor_diagnostics(&self) -> Vec<String> {
        self.projection().descriptor_diagnostics().to_vec()
    }

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
mod tests {
    use super::diagnostic_mentions_plugin;

    #[test]
    fn plugin_diagnostic_matching_preserves_boundaries_and_embedded_prefixes() {
        assert!(diagnostic_mentions_plugin(
            "load failed: native plugin physics: invalid ABI",
            "physics"
        ));
        assert!(diagnostic_mentions_plugin(
            "native plugin physics skipped",
            "physics"
        ));
        assert!(!diagnostic_mentions_plugin(
            "native plugin physics2 skipped",
            "physics"
        ));
    }

    #[test]
    fn plugin_diagnostic_matching_does_not_format_needles_per_message() {
        let source = include_str!("diagnostics.rs");
        let function = source
            .split_once("fn diagnostic_mentions_plugin")
            .expect("diagnostic matcher should exist")
            .1
            .split_once("#[cfg(test)]")
            .expect("tests should follow the matcher")
            .0;

        assert!(!function.contains("format!"));
    }
}
