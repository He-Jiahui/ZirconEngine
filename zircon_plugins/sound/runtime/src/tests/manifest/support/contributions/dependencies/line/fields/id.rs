mod field;
mod state;
mod value;

use super::super::super::state::DependencyParserState;

pub(super) fn parse_dependency_id_field(line: &str, parser: &mut DependencyParserState) -> bool {
    let Some(value) = field::dependency_id_value(line) else {
        return false;
    };
    state::set_dependency_id(parser, value::dependency_id_from_plugin_toml(value));
    true
}
