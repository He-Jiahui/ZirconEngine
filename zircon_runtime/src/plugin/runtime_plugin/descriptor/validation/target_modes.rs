use super::DescriptorTargetMode;

pub(super) fn validate_runtime_plugin_descriptor_target_modes(
    target_modes: &[DescriptorTargetMode],
    diagnostics: &mut Vec<String>,
) {
    if target_modes.is_empty() {
        diagnostics.push(
            "runtime plugin descriptor target_modes must declare at least one target mode"
                .to_string(),
        );
    }
    let mut seen = Vec::new();
    for target_mode in target_modes.iter().copied() {
        if seen.contains(&target_mode) {
            diagnostics.push(format!(
                "runtime plugin descriptor target mode {target_mode:?} must be unique"
            ));
        } else {
            seen.push(target_mode);
        }
    }
}
