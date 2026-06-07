use super::super::storage::OptionManifestParserState;

impl OptionManifestParserState {
    pub(in super::super::super) fn finish(
        mut self,
    ) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
        self.pending.push_into(&mut self.options);
        self.options
    }
}
