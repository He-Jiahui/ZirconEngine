use super::super::storage::EventCatalogParserState;

impl EventCatalogParserState {
    pub(in super::super::super) fn set_namespace(&mut self, namespace: String) {
        self.current_namespace = Some(namespace);
    }
}
