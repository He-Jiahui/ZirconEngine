mod field;
mod state;
mod value;

use super::super::super::state::EventCatalogParserState;

pub(super) fn parse_event_catalog_namespace_field(
    line: &str,
    parser: &mut EventCatalogParserState,
) -> bool {
    let Some(value) = field::event_catalog_namespace_value(line) else {
        return false;
    };
    state::set_event_catalog_namespace(
        parser,
        value::event_catalog_namespace_from_plugin_toml(value),
    );
    true
}
