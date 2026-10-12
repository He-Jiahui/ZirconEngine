// 表切换是静态测试投影的提交边界，离开本类表后不再吸收同名字段。
use super::storage::EventCatalogParserState;

impl EventCatalogParserState {
    pub(in super::super) fn begin_event_catalog_table(&mut self) {
        self.push_current_event_catalog();
        self.inside_catalog = true;
    }

    pub(in super::super) fn leave_event_catalog_table(&mut self) {
        self.push_current_event_catalog();
        self.inside_catalog = false;
    }

    pub(in super::super) fn is_inside_event_catalog(&self) -> bool {
        self.inside_catalog
    }
}
