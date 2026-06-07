mod field;
mod state;
mod value;

pub(super) fn parse_dependency_plugin_id_line(line: &str, plugin_id: &mut Option<String>) -> bool {
    let Some(value) = field::dependency_plugin_id_value(line) else {
        return false;
    };
    state::set_dependency_plugin_id(
        plugin_id,
        value::dependency_plugin_id_from_plugin_toml(value),
    );
    true
}
