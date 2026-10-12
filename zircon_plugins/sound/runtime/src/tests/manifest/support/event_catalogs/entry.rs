// 只扫描 sound/plugin.toml 中对应的数组表；输出供静态与运行时包清单对照，输入不应当是任意 TOML。
use super::{line, state};

pub(super) fn event_catalogs_from_plugin_toml(
    manifest: &str,
) -> Vec<super::super::StaticEventCatalog> {
    let mut parser = state::EventCatalogParserState::default();

    for line in manifest.lines().map(str::trim) {
        line::parse_event_catalog_line(line, &mut parser);
    }
    parser.finish()
}
