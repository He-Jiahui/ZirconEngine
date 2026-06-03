use std::collections::HashSet;

use super::native_plugin_load_manifest_template::native_dynamic_package_directory;

pub(super) struct NativeDynamicPackagePlan {
    pub(super) packages: Vec<String>,
    pub(super) diagnostics: Vec<String>,
}

#[derive(Default)]
pub(super) struct NativeDynamicPackageAccumulator {
    package_ids: HashSet<String>,
    package_directories: HashSet<String>,
    packages: Vec<String>,
    diagnostics: Vec<String>,
}

impl NativeDynamicPackageAccumulator {
    pub(super) fn push(&mut self, package_id: &str) {
        if !self.package_ids.insert(package_id.to_string()) {
            return;
        }
        let package_directory = native_dynamic_package_directory(package_id);
        if !self.package_directories.insert(package_directory.clone()) {
            self.diagnostics.push(format!(
                "plugin {package_id} uses NativeDynamic packaging but resolves to duplicate output directory plugins/{package_directory}"
            ));
            return;
        }
        self.packages.push(package_id.to_string());
    }

    pub(super) fn finish(self) -> NativeDynamicPackagePlan {
        NativeDynamicPackagePlan {
            packages: self.packages,
            diagnostics: self.diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NativeDynamicPackageAccumulator;

    #[test]
    fn accumulator_deduplicates_native_dynamic_package_ids() {
        let mut accumulator = NativeDynamicPackageAccumulator::default();

        accumulator.push("sound");
        accumulator.push("sound");
        let plan = accumulator.finish();

        assert_eq!(plan.packages, vec!["sound".to_string()]);
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    fn accumulator_reports_sanitized_output_directory_collisions() {
        let mut accumulator = NativeDynamicPackageAccumulator::default();

        accumulator.push("sound/escape");
        accumulator.push("sound_escape");
        let plan = accumulator.finish();

        assert_eq!(plan.packages, vec!["sound/escape".to_string()]);
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.contains(
            "plugin sound_escape uses NativeDynamic packaging but resolves to duplicate output directory plugins/sound_escape"
        )));
    }
}
