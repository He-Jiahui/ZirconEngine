#[path = "../../../../support/line/fields/version/field.rs"]
mod field;
#[path = "../../../../support/line/fields/version/state.rs"]
mod state;
#[path = "../../../../support/line/fields/version/value.rs"]
mod value;

use super::super::super::state::EventCatalogParserState;

pub(super) fn parse_event_catalog_version_field(line: &str, parser: &mut EventCatalogParserState) {
    let Some(value) = field::event_catalog_version_value(line) else {
        return;
    };
    state::set_event_catalog_version(parser, value::event_catalog_version_from_plugin_toml(value));
}
