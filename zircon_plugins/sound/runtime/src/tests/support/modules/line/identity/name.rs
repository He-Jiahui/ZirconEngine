#[path = "../name/field.rs"]
mod field;
#[path = "../name/state.rs"]
mod state;
#[path = "../name/value.rs"]
mod value;

pub(super) fn parse_module_name_line(line: &str, name: &mut Option<String>) -> bool {
    let Some(value) = field::module_name_value(line) else {
        return false;
    };
    state::set_module_name(name, value::module_name_from_plugin_toml(value));
    true
}
