use super::super::super::super::StaticEventCatalog;
use super::super::storage::EventCatalogParserState;

impl EventCatalogParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticEventCatalog> {
        self.push_current_event_catalog();
        self.catalogs
    }
}
