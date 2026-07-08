use std::fs;
use std::path::Path;

pub(super) fn assert_owner_files(
    owner_root: &Path,
    expected_modules: &[&str],
    max_lines: usize,
    owner_label: &str,
) {
    for module in expected_modules {
        let path = owner_root.join(module);
        assert!(
            path.exists(),
            "{owner_label} module `{module}` is missing; update input_stack_boundary before changing the input owner set"
        );
        let line_count = line_count(&path);
        assert!(
            line_count <= max_lines,
            "{owner_label} module `{module}` has {line_count} lines, exceeding the {max_lines}-line owner budget"
        );
    }
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}
