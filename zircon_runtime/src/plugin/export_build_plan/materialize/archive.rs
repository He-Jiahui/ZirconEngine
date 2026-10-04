use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime};

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_directory, native_dynamic_package_report_template,
    NativeDynamicPackageExportPlan,
};
use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::{
    write_verified_native_file, MaterializedFileStaging, NativeDynamicPackageFileEntry,
};
use super::package_lookup::NativePackageInventory;
use super::paths::validated_materialized_relative_path;

const NATIVE_PACKAGE_REPORT_FILE: &str = "native_dynamic_package.toml";
const AUTHORITY_PATH: &str = "src/zircon_native_authority.json";

pub(super) fn materialize_zip_archive(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
    archive_path: &Path,
) -> Result<ExportMaterializeReport, std::io::Error> {
    let fatal_diagnostics = plan.effective_fatal_diagnostics();
    if !fatal_diagnostics.is_empty() {
        return Ok(blocked_archive_report(
            plan,
            archive_path.to_path_buf(),
            fatal_diagnostics,
        ));
    }

    validate_reserved_authority_entry(plan)?;
    // Admission and authority construction finish before staging starts; the
    // previous archive remains published until every staged entry is verified.
    let native_packages = if plan.native_dynamic_packages.is_empty() {
        None
    } else {
        let inventory = NativePackageInventory::build(plugin_root, &plan.native_dynamic_packages)?;
        validate_native_entry_collisions(plan, &inventory)?;
        let authority = super::native_authority::embedded_native_authority_bytes(plan, &inventory)?;
        Some((inventory, authority))
    };

    if let Some(parent) = archive_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let (file, staging) = MaterializedFileStaging::create(archive_path)?;
    let mut writer = ZipWriter::new(file);
    let mut written_entries = HashSet::with_capacity(archive_entry_capacity(plan));
    let mut report = ExportMaterializeReport {
        archive_file: Some(archive_path.to_path_buf()),
        generated_files: Vec::with_capacity(plan.generated_files.len()),
        copied_packages: Vec::with_capacity(plan.native_dynamic_packages.len()),
        diagnostics: plan.diagnostics.clone(),
        fatal_diagnostics,
    };

    write_generated_entries(plan, &mut writer, &mut written_entries, &mut report)?;
    if let Some((inventory, authority)) = native_packages {
        if !written_entries.insert(AUTHORITY_PATH.to_string()) {
            return Err(std::io::Error::other(
                "generated native authority path is reserved",
            ));
        }
        write_zip_entry(&mut writer, AUTHORITY_PATH, &authority)?;
        report.generated_files.push(PathBuf::from(AUTHORITY_PATH));
        write_native_package_entries(
            plan,
            &inventory,
            &mut writer,
            &mut written_entries,
            &mut report,
        )?;
    }
    let file = writer.finish()?;
    file.sync_all()?;
    drop(file);
    staging.publish()?;

    Ok(report)
}

pub(super) fn preview_zip_archive(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
    archive_path: &Path,
) -> Result<ExportMaterializeReport, std::io::Error> {
    validate_reserved_authority_entry(plan)?;
    let mut report = ExportMaterializeReport {
        archive_file: Some(archive_path.to_path_buf()),
        generated_files: Vec::with_capacity(plan.generated_files.len()),
        copied_packages: Vec::with_capacity(plan.native_dynamic_packages.len()),
        diagnostics: plan.diagnostics.clone(),
        fatal_diagnostics: plan.effective_fatal_diagnostics(),
    };

    for file in &plan.generated_files {
        validated_materialized_relative_path(&file.path)?;
        report
            .generated_files
            .push(PathBuf::from(file.path.as_str()));
    }

    if !plan.native_dynamic_packages.is_empty() {
        let inventory = NativePackageInventory::build(plugin_root, &plan.native_dynamic_packages)?;
        validate_native_entry_collisions(plan, &inventory)?;
        preview_native_package_entries(plan, &inventory, &mut report)?;
        report.generated_files.push(PathBuf::from(AUTHORITY_PATH));
    }

    Ok(report)
}

fn validate_reserved_authority_entry(plan: &ExportBuildPlan) -> Result<(), std::io::Error> {
    if plan.native_dynamic_packages.is_empty() {
        return Ok(());
    }
    for file in &plan.generated_files {
        if validated_materialized_relative_path(&file.path)?.eq_ignore_ascii_case(AUTHORITY_PATH) {
            return Err(std::io::Error::other(
                "generated native authority path is reserved",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_native_entry_collisions(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
) -> std::io::Result<()> {
    let mut selected_directories = HashMap::with_capacity(plan.native_dynamic_packages.len());
    let generated = plan
        .generated_files
        .iter()
        .map(|file| {
            validated_materialized_relative_path(&file.path).map(|path| path.to_ascii_lowercase())
        })
        .collect::<std::io::Result<HashSet<_>>>()?;
    for package_id in &plan.native_dynamic_packages {
        let directory_name = native_dynamic_package_directory(package_id);
        if let Some(previous_id) =
            selected_directories.insert(directory_name.to_ascii_lowercase(), package_id.as_str())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "selected native packages {previous_id} and {package_id} collide at output directory plugins/{directory_name}"
                ),
            ));
        }
        let Some(files) = inventory.file_inventory(package_id) else {
            continue;
        };
        let directory = format!("plugins/{directory_name}");
        for relative_path in files
            .entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .chain(std::iter::once(NATIVE_PACKAGE_REPORT_FILE))
        {
            let entry_path =
                validated_materialized_relative_path(&format!("{directory}/{relative_path}"))?;
            if generated.contains(&entry_path.to_ascii_lowercase()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("generated export entry {entry_path} collides with native package {package_id}"),
                ));
            }
        }
    }
    Ok(())
}

fn write_generated_entries<W: Write + std::io::Seek>(
    plan: &ExportBuildPlan,
    writer: &mut ZipWriter<W>,
    written_entries: &mut HashSet<String>,
    report: &mut ExportMaterializeReport,
) -> Result<(), std::io::Error> {
    let mut generated_files = plan.generated_files.iter().collect::<Vec<_>>();
    generated_files.sort_by(|left, right| left.path.cmp(&right.path));

    for file in generated_files {
        let entry_name = validated_materialized_relative_path(&file.path)?;
        if !written_entries.insert(entry_name.clone()) {
            report.diagnostics.push(format!(
                "export archive skipped duplicate entry {entry_name}"
            ));
            continue;
        }
        write_zip_entry(writer, &entry_name, file.contents.as_bytes())?;
        report.generated_files.push(PathBuf::from(entry_name));
    }

    Ok(())
}

fn write_native_package_entries<W: Write + std::io::Seek>(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
    writer: &mut ZipWriter<W>,
    written_entries: &mut HashSet<String>,
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

        let archive_directory = format!("plugins/{package_directory}");
        report
            .diagnostics
            .extend(file_inventory.diagnostics.iter().cloned());

        for entry in &file_inventory.entries {
            let archive_path = validated_materialized_relative_path(&format!(
                "{archive_directory}/{}",
                entry.relative_path
            ))?;
            if !written_entries.insert(archive_path.clone()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("export archive native entry {archive_path} collides with an existing entry"),
                ));
            }
            write_zip_file_entry(writer, &archive_path, entry)?;
        }

        let fallback_export;
        let package_export = if let Some(package_export) = package_exports.get(package_id.as_str())
        {
            *package_export
        } else {
            fallback_export = NativeDynamicPackageExportPlan::for_package_id(package_id.as_str());
            &fallback_export
        };
        let report_path = validated_materialized_relative_path(&format!(
            "{archive_directory}/{NATIVE_PACKAGE_REPORT_FILE}"
        ))?;
        if !written_entries.insert(report_path.clone()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "export archive native report {report_path} collides with an existing entry"
                ),
            ));
        }
        let report_contents = native_dynamic_package_report_template(package_export);
        write_zip_entry(writer, &report_path, report_contents.as_bytes())?;
        report
            .copied_packages
            .push(PathBuf::from(archive_directory));
    }

    Ok(())
}

fn preview_native_package_entries(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
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
        report
            .diagnostics
            .extend(file_inventory.diagnostics.iter().cloned());
        report
            .copied_packages
            .push(PathBuf::from(format!("plugins/{package_directory}")));
    }

    Ok(())
}

fn write_zip_entry<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    entry_name: &str,
    contents: &[u8],
) -> Result<(), std::io::Error> {
    let options = zip_file_options();
    writer.start_file(entry_name, options)?;
    writer.write_all(contents)?;
    Ok(())
}

fn write_zip_file_entry<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    entry_name: &str,
    source: &NativeDynamicPackageFileEntry,
) -> Result<(), std::io::Error> {
    let options = zip_file_options();
    writer.start_file(entry_name, options)?;
    write_verified_native_file(source, writer)?;
    Ok(())
}

fn zip_file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644)
}

fn archive_entry_capacity(plan: &ExportBuildPlan) -> usize {
    plan.generated_files
        .len()
        .saturating_add(plan.native_dynamic_packages.len())
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

fn blocked_archive_report(
    plan: &ExportBuildPlan,
    archive_path: PathBuf,
    fatal_diagnostics: Vec<String>,
) -> ExportMaterializeReport {
    let mut diagnostics = plan.diagnostics.clone();
    diagnostics.push(format!(
        "export archive materialization blocked for profile {}: fatal diagnostics must be resolved before writing {}",
        plan.profile.name,
        archive_path.display()
    ));

    ExportMaterializeReport {
        archive_file: Some(archive_path),
        generated_files: Vec::new(),
        copied_packages: Vec::new(),
        diagnostics,
        fatal_diagnostics,
    }
}

#[cfg(test)]
#[path = "tests/archive.rs"]
mod tests;
