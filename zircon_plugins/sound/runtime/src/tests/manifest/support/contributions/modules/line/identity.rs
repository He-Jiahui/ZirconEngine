mod crate_name;
mod name;

pub(super) fn parse_module_identity_line(
    line: &str,
    name: &mut Option<String>,
    crate_name: &mut Option<String>,
) -> bool {
    name::parse_module_name_line(line, name)
        || crate_name::parse_module_crate_name_line(line, crate_name)
}
