use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime};

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_directory, native_dynamic_package_report_template,
    NativeDynamicPackageExportPlan,
};
use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::{native_dynamic_package_file_entries, preview_native_dynamic_package_copy};
use super::package_lookup::find_native_package_dir;
use super::paths::validated_materialized_relative_path;

const NATIVE_PACKAGE_REPORT_FILE: &str = "native_dynamic_package.toml";

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

    if let Some(parent) = archive_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(archive_path)?;
    let mut writer = ZipWriter::new(file);
    let mut written_entries = HashSet::new();
    let mut report = ExportMaterializeReport {
        archive_file: Some(archive_path.to_path_buf()),
        generated_files: Vec::new(),
        copied_packages: Vec::new(),
        diagnostics: plan.diagnostics.clone(),
        fatal_diagnostics,
    };

    write_generated_entries(plan, &mut writer, &mut written_entries, &mut report)?;
    write_native_package_entries(
        plan,
        plugin_root,
        &mut writer,
        &mut written_entries,
        &mut report,
    )?;
    writer.finish()?;

    Ok(report)
}

pub(super) fn preview_zip_archive(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
    archive_path: &Path,
) -> Result<ExportMaterializeReport, std::io::Error> {
    let mut report = ExportMaterializeReport {
        archive_file: Some(archive_path.to_path_buf()),
        generated_files: Vec::new(),
        copied_packages: Vec::new(),
        diagnostics: plan.diagnostics.clone(),
        fatal_diagnostics: plan.effective_fatal_diagnostics(),
    };

    for file in &plan.generated_files {
        validated_materialized_relative_path(&file.path)?;
        report
            .generated_files
            .push(PathBuf::from(file.path.as_str()));
    }

    preview_native_package_entries(plan, plugin_root, &mut report)?;

    Ok(report)
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
    plugin_root: &Path,
    writer: &mut ZipWriter<W>,
    written_entries: &mut HashSet<String>,
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

        let archive_directory = format!("plugins/{package_directory}");
        let preview = preview_native_dynamic_package_copy(&source, package_id)?;
        report.diagnostics.extend(preview.diagnostics);

        let entries = native_dynamic_package_file_entries(&source)?;
        for entry in entries {
            let archive_path = validated_materialized_relative_path(&format!(
                "{archive_directory}/{}",
                entry.relative_path
            ))?;
            if !written_entries.insert(archive_path.clone()) {
                report.diagnostics.push(format!(
                    "export archive skipped duplicate entry {archive_path}"
                ));
                continue;
            }
            write_zip_file_entry(writer, &archive_path, &entry.source_path)?;
        }

        let package_export = native_dynamic_package_export(plan, package_id)
            .cloned()
            .unwrap_or_else(|| NativeDynamicPackageExportPlan::for_package_id(package_id.as_str()));
        let report_path = validated_materialized_relative_path(&format!(
            "{archive_directory}/{NATIVE_PACKAGE_REPORT_FILE}"
        ))?;
        if written_entries.insert(report_path.clone()) {
            let report_contents = native_dynamic_package_report_template(&package_export);
            write_zip_entry(writer, &report_path, report_contents.as_bytes())?;
        }
        report
            .copied_packages
            .push(PathBuf::from(archive_directory));
    }

    Ok(())
}

fn preview_native_package_entries(
    plan: &ExportBuildPlan,
    plugin_root: &Path,
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
        let preview = preview_native_dynamic_package_copy(&source, package_id)?;
        report.diagnostics.extend(preview.diagnostics);
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
    source_path: &Path,
) -> Result<(), std::io::Error> {
    let options = zip_file_options();
    writer.start_file(entry_name, options)?;
    let mut source = File::open(source_path)?;
    std::io::copy(&mut source, writer)?;
    Ok(())
}

fn zip_file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644)
}

fn native_dynamic_package_export<'a>(
    plan: &'a ExportBuildPlan,
    package_id: &str,
) -> Option<&'a NativeDynamicPackageExportPlan> {
    plan.native_dynamic_package_exports
        .iter()
        .find(|package| package.package_id == package_id)
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
mod tests {
    #[test]
    fn native_package_archive_entries_are_streamed_from_disk() {
        let source = include_str!("archive.rs");
        let whole_file_read = ["fs::", "read(&entry.source_path)"].concat();
        assert!(
            !source.contains(&whole_file_read),
            "native package files should stream into ZipWriter without a full-file Vec"
        );
    }
}
