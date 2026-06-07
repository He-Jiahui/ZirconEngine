use super::super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super::super) fn set_status(
        &mut self,
        status: zircon_runtime::plugin::CapabilityStatus,
    ) {
        self.current_status = Some(status);
    }
}
