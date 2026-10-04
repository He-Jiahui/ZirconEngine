//! 在原生库存及 authority 预检之后复制选定包；报告与预览沿用计划顺序和同一库存视图。
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_directory, NativeDynamicPackageExportPlan,
};
use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::copy_native_dynamic_package_files;
use super::package_lookup::NativePackageInventory;
use super::report::write_native_dynamic_package_report;

/// 调用方先完成选包库存、碰撞和 authority 预检；此阶段复制已入库文件并生成每包报告。
pub(super) fn materialize_native_dynamic_packages(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::with_capacity(plan.native_dynamic_packages.len());
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

/// 使用同一库存估计包输出，不再次搜索磁盘或修改导出根目录。
pub(super) fn preview_native_dynamic_packages(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut copied_package_directories = HashSet::with_capacity(plan.native_dynamic_packages.len());

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
#[path = "tests/native.rs"]
mod tests;
