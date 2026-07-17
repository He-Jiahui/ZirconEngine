mod count;

use self::count::validate_runtime_plugin_package_semver_segment_count;
use super::component::validate_runtime_plugin_package_semver_component;

pub(super) fn validate_runtime_plugin_package_semver_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segments = value.split('.');
    let Some(major) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 0, diagnostics);
        return;
    };
    let Some(minor) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 1, diagnostics);
        return;
    };
    let Some(patch) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 2, diagnostics);
        return;
    };
    if segments.next().is_some() {
        validate_runtime_plugin_package_semver_segment_count(
            field_name,
            value,
            4 + segments.count(),
            diagnostics,
        );
        return;
    }
    for (component_name, segment) in ["major", "minor", "patch"]
        .into_iter()
        .zip([major, minor, patch])
    {
        validate_runtime_plugin_package_semver_component(
            field_name,
            value,
            component_name,
            segment,
            diagnostics,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_semver_validation_does_not_collect_segments() {
        let source = include_str!("segments.rs");
        let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&allocating_shape));
    }

    #[test]
    fn package_semver_validation_preserves_shape_and_component_diagnostics() {
        let mut diagnostics = Vec::new();
        super::validate_runtime_plugin_package_semver_segments(
            "version",
            "1.2.3",
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());

        super::validate_runtime_plugin_package_semver_segments("version", "1.2", &mut diagnostics);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("must use MAJOR.MINOR.PATCH form"));

        diagnostics.clear();
        super::validate_runtime_plugin_package_semver_segments("version", "1..3", &mut diagnostics);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("minor component `` must contain ASCII digits"));
    }
}
