// 仅保存当前静态表行及已完成结果；表边界负责提交，最终结果交给包清单对照用例。
use super::super::super::StaticEventCatalog;

// Keeps event-catalog row finalization tied to the static TOML table scanner.
#[derive(Default)]
pub(in super::super) struct EventCatalogParserState {
    pub(in super::super) catalogs: Vec<StaticEventCatalog>,
    pub(in super::super) current_namespace: Option<String>,
    pub(in super::super) current_version: Option<u32>,
    pub(in super::super) inside_catalog: bool,
}
