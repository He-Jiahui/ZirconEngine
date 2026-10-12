#[path = "../../../../modules/line/identity/crate_name.rs"]
mod crate_name;
#[path = "../../../../support/modules/line/identity/name.rs"]
mod name;

pub(super) fn parse_module_identity_line(
    line: &str,
    name: &mut Option<String>,
    crate_name: &mut Option<String>,
) -> bool {
    name::parse_module_name_line(line, name)
        || crate_name::parse_module_crate_name_line(line, crate_name)
}
