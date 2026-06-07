mod field;
mod state;
mod value;

use super::super::super::state::DependencyParserState;

pub(super) fn parse_dependency_capability_field(line: &str, parser: &mut DependencyParserState) {
    let Some(value) = field::dependency_capability_value(line) else {
        return;
    };
    state::set_dependency_capability(parser, value::dependency_capability_from_plugin_toml(value));
}
