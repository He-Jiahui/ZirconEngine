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
    for (index, target_mode) in target_modes.iter().copied().enumerate() {
        if target_modes[..index].contains(&target_mode) {
            diagnostics.push(format!(
                "runtime plugin descriptor target mode {target_mode:?} must be unique"
            ));
        }
    }
}

#[cfg(test)]
#[path = "target_modes/allocation_tests.rs"]
mod allocation_tests;
