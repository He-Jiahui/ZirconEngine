use std::path::Path;

use super::super::non_empty_string_value;

pub(super) fn visit_option_rows(
    table: &toml::Table,
    relative_path: &Path,
    require_non_empty: bool,
    visit: &mut impl FnMut(&toml::Table, &str, &str),
) {
    let Some(options) = table.get("options") else {
        return;
    };
    let options = options
        .as_array()
        .unwrap_or_else(|| panic!("plugin manifest {relative_path:?} options should be an array"));
    if require_non_empty {
        assert!(
            !options.is_empty(),
            "plugin manifest {relative_path:?} options should not be empty when declared"
        );
    }

    for option in options {
        let option = option.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} option should be a table")
        });
        let key = non_empty_string_value(option, relative_path, "plugin option", "key");
        let option_context = format!("plugin option `{key}`");
        visit(option, key, &option_context);
    }
}
