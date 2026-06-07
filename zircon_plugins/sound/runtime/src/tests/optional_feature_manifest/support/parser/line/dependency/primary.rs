mod field;
mod state;
mod value;

pub(super) fn parse_dependency_primary_line(line: &str, primary: &mut Option<bool>) -> bool {
    let Some(value) = field::dependency_primary_value(line) else {
        return false;
    };
    state::set_dependency_primary(primary, value::dependency_primary_from_plugin_toml(value));
    true
}
