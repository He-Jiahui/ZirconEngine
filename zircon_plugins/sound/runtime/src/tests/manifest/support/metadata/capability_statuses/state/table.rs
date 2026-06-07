use super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super) fn begin_status_table(&mut self) {
        self.push_current_status();
        self.inside_status = true;
    }

    pub(in super::super) fn leave_status_table(&mut self) {
        self.push_current_status();
        self.inside_status = false;
    }

    pub(in super::super) fn is_inside_status(&self) -> bool {
        self.inside_status
    }
}
