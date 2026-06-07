use super::super::super::super::super::super::values::string_array_values;

pub(super) fn option_enum_values_from_plugin_toml(value: &str) -> Vec<String> {
    string_array_values(value)
}
