mod field;
mod state;
mod value;

use super::super::super::state::DependencyParserState;

pub(super) fn parse_dependency_required_field(
    line: &str,
    parser: &mut DependencyParserState,
) -> bool {
    let Some(value) = field::dependency_required_value(line) else {
        return false;
    };
    state::set_dependency_required(parser, value::dependency_required_from_plugin_toml(value));
    true
}
