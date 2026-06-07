use super::super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super::super) fn finish(
        mut self,
    ) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
        self.push_current_status();
        self.statuses
    }
}
