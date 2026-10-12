#[path = "../../../modules/line/capabilities/field.rs"]
mod field;
#[path = "../../../modules/line/capabilities/state.rs"]
mod state;
#[path = "../../../modules/line/capabilities/values.rs"]
mod values;

pub(super) fn parse_module_capabilities_line(line: &str, capabilities: &mut Vec<String>) {
    let Some(value) = field::module_capabilities_value(line) else {
        return;
    };
    state::set_module_capabilities(
        capabilities,
        values::module_capabilities_from_plugin_toml(value),
    );
}
