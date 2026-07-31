use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_directory, NativeDynamicPackageExportPlan,
};
use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::copy_native_dynamic_package_files;
use super::package_lookup::NativePackageInventory;
use super::report::write_native_dynamic_package_report;

pub(super) fn materialize_native_dynamic_packages(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::new();
    let package_exports = native_dynamic_package_export_index(plan);

    for package_id in &plan.native_dynamic_packages {
        let Some(file_inventory) = inventory.file_inventory(package_id) else {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} was selected but no plugin.toml was found under {}",
                inventory.plugin_root().display()
            ));
            continue;
        };
        let package_directory = native_dynamic_package_directory(package_id);
        if !copied_package_directories.insert(package_directory.clone()) {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} resolves to duplicate output directory plugins/{package_directory}"
            ));
            continue;
        }
        let destination = output_root.join("plugins").join(package_directory);
        report
            .diagnostics
            .extend(file_inventory.diagnostics.iter().cloned());
        copy_native_dynamic_package_files(&file_inventory.entries, &destination)?;
        let fallback_export;
        let package_export = if let Some(package_export) = package_exports.get(package_id.as_str())
        {
            *package_export
        } else {
            fallback_export = NativeDynamicPackageExportPlan::for_package_id(package_id.as_str());
            &fallback_export
        };
        write_native_dynamic_package_report(&destination, package_export)?;
        report.copied_packages.push(destination);
    }

    Ok(())
}

pub(super) fn preview_native_dynamic_packages(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::new();

    for package_id in &plan.native_dynamic_packages {
        let Some(file_inventory) = inventory.file_inventory(package_id) else {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} was selected but no plugin.toml was found under {}",
                inventory.plugin_root().display()
            ));
            continue;
        };
        let package_directory = native_dynamic_package_directory(package_id);
        if !copied_package_directories.insert(package_directory.clone()) {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} resolves to duplicate output directory plugins/{package_directory}"
            ));
            continue;
        }
        let destination = output_root.join("plugins").join(package_directory);
        report
            .diagnostics
            .extend(file_inventory.diagnostics.iter().cloned());
        report.copied_packages.push(destination);
    }

    Ok(())
}

fn native_dynamic_package_export_index<'a>(
    plan: &'a ExportBuildPlan,
) -> HashMap<&'a str, &'a NativeDynamicPackageExportPlan> {
    let mut exports = HashMap::with_capacity(plan.native_dynamic_package_exports.len());
    for package in &plan.native_dynamic_package_exports {
        exports
            .entry(package.package_id.as_str())
            .or_insert(package);
    }
    exports
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_materialization_indexes_package_export_rows_once() {
        let source = include_str!("native.rs");
        let linear_lookup = [".find(|package| package.", "package_id == package_id)"].concat();
        let cloned_lookup = [".copied()", "\n            .cloned()"].concat();

        assert!(source.contains("native_dynamic_package_export_index(plan)"));
        assert!(source.contains("copy_native_dynamic_package_files"));
        assert!(!source.contains("fs::copy"));
        assert!(!source.contains(&linear_lookup));
        assert!(!source.contains(&cloned_lookup));
    }
}
