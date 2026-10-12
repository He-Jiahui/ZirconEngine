#[path = "../../../support/parser/line/capability/field.rs"]
mod field;
#[path = "../../../support/parser/line/capability/state.rs"]
mod state;
#[path = "../../../support/parser/line/capability/value.rs"]
mod value;

pub(super) fn parse_dependency_capability_line(
    line: &str,
    capability: &mut Option<String>,
) -> bool {
    let Some(value) = field::dependency_capability_value(line) else {
        return false;
    };
    state::set_dependency_capability(
        capability,
        value::dependency_capability_from_plugin_toml(value),
    );
    true
}
