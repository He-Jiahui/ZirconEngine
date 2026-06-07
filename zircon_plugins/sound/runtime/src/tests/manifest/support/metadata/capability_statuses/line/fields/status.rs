mod field;
mod state;
mod value;

use super::super::super::state::CapabilityStatusParserState;

pub(super) fn parse_status_field(line: &str, parser: &mut CapabilityStatusParserState) -> bool {
    let Some(value) = field::capability_status_status_value(line) else {
        return false;
    };
    state::set_capability_status(
        parser,
        value::capability_status_status_from_plugin_toml(value),
    );
    true
}
