// 当前固定清单用数组表界定记录；切换到其他表时完成上一条，避免相同字段跨表串入。
use super::super::state::EventCatalogParserState;

pub(super) fn event_catalog_table_transition_consumed(
    line: &str,
    parser: &mut EventCatalogParserState,
) -> bool {
    if line == "[[event_catalogs]]" {
        parser.begin_event_catalog_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_event_catalog_table();
    }
    !parser.is_inside_event_catalog()
}
