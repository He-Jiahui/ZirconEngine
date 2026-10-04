// 有身份字段的静态行才纳入比较；必填字段缺失时应使测试失败，防止不完整声明被接受。
use super::super::storage::EventCatalogParserState;
use super::required;

impl EventCatalogParserState {
    pub(in super::super::super) fn push_current_event_catalog(&mut self) {
        let Some(namespace) = self.current_namespace.take() else {
            return;
        };
        self.catalogs.push((
            namespace,
            required::take_required_event_catalog_version(&mut self.current_version),
        ));
    }
}
