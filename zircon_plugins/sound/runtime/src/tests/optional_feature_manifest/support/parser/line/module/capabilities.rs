mod field;
mod state;
mod values;

pub(super) fn parse_module_capabilities_line(line: &str, capabilities: &mut Vec<String>) -> bool {
    let Some(value) = field::module_capabilities_value(line) else {
        return false;
    };
    state::set_module_capabilities(
        capabilities,
        values::module_capabilities_from_plugin_toml(value),
    );
    true
}
