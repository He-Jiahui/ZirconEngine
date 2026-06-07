mod field;
mod state;
mod value;

use super::super::super::state::CapabilityStatusParserState;

pub(super) fn parse_capability_field(line: &str, parser: &mut CapabilityStatusParserState) -> bool {
    let Some(value) = field::capability_status_capability_value(line) else {
        return false;
    };
    state::set_capability_status_capability(
        parser,
        value::capability_status_capability_from_plugin_toml(value),
    );
    true
}
