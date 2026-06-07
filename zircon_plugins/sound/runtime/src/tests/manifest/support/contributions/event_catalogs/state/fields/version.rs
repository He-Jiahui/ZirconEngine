use super::super::storage::EventCatalogParserState;

impl EventCatalogParserState {
    pub(in super::super::super) fn set_version(&mut self, version: u32) {
        self.current_version = Some(version);
    }
}
