// 文件结束时补交最后一行，保证末尾没有下一张表时仍纳入静态对照。
use super::super::super::super::StaticEventCatalog;
use super::super::storage::EventCatalogParserState;

impl EventCatalogParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticEventCatalog> {
        self.push_current_event_catalog();
        self.catalogs
    }
}
