mod segments;

/// 对参考源码路径应用仓库相对路径契约；前缀和路径段检查一起防止声明越出指定参考树。
pub(super) fn validate_runtime_plugin_package_bevy_reference_path(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    if !reference.starts_with("dev/bevy/") {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must stay under `dev/bevy`"
        ));
    }
    if contains_non_repository_path_separator(reference) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must be a repository-relative forward-slash path"
        ));
    }
    segments::validate_runtime_plugin_package_bevy_reference_path_segments(
        capability,
        reference,
        diagnostics,
    );
}

fn contains_non_repository_path_separator(reference: &str) -> bool {
    reference.bytes().any(|byte| matches!(byte, b'\\' | b':'))
}

#[cfg(test)]
#[path = "tests/path_optimization_tests.rs"]
mod optimization_tests;
