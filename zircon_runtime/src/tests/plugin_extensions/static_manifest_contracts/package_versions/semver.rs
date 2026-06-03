use std::path::Path;

pub(super) fn assert_semver_core(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should not have leading or trailing whitespace"
    );

    let mut segments = value.split('.');
    for component_name in ["major", "minor", "patch"] {
        let segment = segments.next().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should use MAJOR.MINOR.PATCH form"
            )
        });
        assert_semver_segment(
            relative_path,
            context,
            field_name,
            value,
            component_name,
            segment,
        );
    }

    assert!(
        segments.next().is_none(),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should use MAJOR.MINOR.PATCH form"
    );
}

fn assert_semver_segment(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
) {
    assert!(
        !segment.is_empty(),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` has an empty {component_name} component"
    );
    assert!(
        segment.chars().all(|character| character.is_ascii_digit()),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should contain only ASCII digits"
    );
    assert!(
        segment == "0" || !segment.starts_with('0'),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should not use leading zeroes"
    );

    segment.parse::<u32>().unwrap_or_else(|error| {
        panic!(
            "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should fit in u32: {error}"
        )
    });
}
