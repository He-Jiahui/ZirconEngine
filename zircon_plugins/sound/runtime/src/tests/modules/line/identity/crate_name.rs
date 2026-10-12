#[path = "../../../support/modules/line/crate_name/field.rs"]
mod field;
#[path = "../../../support/modules/line/crate_name/state.rs"]
mod state;
#[path = "../../../support/modules/line/crate_name/value.rs"]
mod value;

pub(super) fn parse_module_crate_name_line(line: &str, crate_name: &mut Option<String>) -> bool {
    let Some(value) = field::module_crate_name_value(line) else {
        return false;
    };
    state::set_module_crate_name(crate_name, value::module_crate_name_from_plugin_toml(value));
    true
}
