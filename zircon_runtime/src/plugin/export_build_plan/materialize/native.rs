use std::collections::HashSet;
use std::path::Path;

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_directory, NativeDynamicPackageExportPlan,
};
use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::{copy_native_dynamic_package, preview_native_dynamic_package_copy};
use super::package_lookup::find_native_package_dir;
use super::report::write_native_dynamic_package_report;

pub(super) fn materialize_native_dynamic_packages(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::new();

    for package_id in &plan.native_dynamic_packages {
        let Some(source) = find_native_package_dir(plugin_root, package_id)? else {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} was selected but no plugin.toml was found under {}",
                plugin_root.display()
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
        report.diagnostics.extend(copy_native_dynamic_package(
            &source,
            &destination,
            package_id,
        )?);
        let package_export = native_dynamic_package_export(plan, package_id)
            .cloned()
            .unwrap_or_else(|| NativeDynamicPackageExportPlan::for_package_id(package_id.as_str()));
        write_native_dynamic_package_report(&destination, &package_export)?;
        report.copied_packages.push(destination);
    }

    Ok(())
}

pub(super) fn preview_native_dynamic_packages(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::new();

    for package_id in &plan.native_dynamic_packages {
        let Some(source) = find_native_package_dir(plugin_root, package_id)? else {
            report.diagnostics.push(format!(
                "native dynamic package {package_id} was selected but no plugin.toml was found under {}",
                plugin_root.display()
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
        let preview = preview_native_dynamic_package_copy(&source, package_id)?;
        report.diagnostics.extend(preview.diagnostics);
        report.copied_packages.push(destination);
    }

    Ok(())
}

fn native_dynamic_package_export<'a>(
    plan: &'a ExportBuildPlan,
    package_id: &str,
) -> Option<&'a NativeDynamicPackageExportPlan> {
    plan.native_dynamic_package_exports
        .iter()
        .find(|package| package.package_id == package_id)
}
