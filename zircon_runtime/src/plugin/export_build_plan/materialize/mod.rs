mod archive;
mod copy;
mod generated;
mod native;
mod package_lookup;
mod paths;
mod report;

use std::path::{Path, PathBuf};

use super::{ExportBuildPlan, ExportMaterializeReport};
use package_lookup::NativePackageInventory;

impl ExportBuildPlan {
    pub fn write_generated_files(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        if self.has_fatal_diagnostics() {
            return Ok(Vec::new());
        }

        generated::write_generated_files(self, root.as_ref())
    }

    pub fn materialize(
        &self,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let fatal_diagnostics = self.effective_fatal_diagnostics();
        if !fatal_diagnostics.is_empty() {
            return Ok(self.blocked_materialize_report(fatal_diagnostics));
        }

        let generated_files = generated::write_generated_files(self, output_root.as_ref())?;
        Ok(ExportMaterializeReport {
            archive_file: None,
            generated_files,
            copied_packages: Vec::new(),
            diagnostics: self.diagnostics.clone(),
            fatal_diagnostics,
        })
    }

    pub fn materialize_with_native_packages(
        &self,
        plugin_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let plugin_root = plugin_root.as_ref();
        let output_root = output_root.as_ref();
        let mut report = self.materialize(output_root)?;

        if !report.fatal_diagnostics.is_empty() {
            return Ok(report);
        }

        if !self.native_dynamic_packages.is_empty() {
            let copied_package_capacity = self.native_dynamic_packages.len();
            report.copied_packages.reserve(copied_package_capacity);
            let inventory =
                NativePackageInventory::build(plugin_root, &self.native_dynamic_packages)?;
            native::materialize_native_dynamic_packages(
                self,
                &inventory,
                output_root,
                &mut report,
            )?;
        }

        Ok(report)
    }

    pub fn preview_materialize(
        &self,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let generated_files = generated::preview_generated_files(self, output_root.as_ref())?;
        Ok(ExportMaterializeReport {
            archive_file: None,
            generated_files,
            copied_packages: Vec::new(),
            diagnostics: self.diagnostics.clone(),
            fatal_diagnostics: self.effective_fatal_diagnostics(),
        })
    }

    pub fn preview_materialize_with_native_packages(
        &self,
        plugin_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let plugin_root = plugin_root.as_ref();
        let output_root = output_root.as_ref();
        let mut report = self.preview_materialize(output_root)?;

        if !self.native_dynamic_packages.is_empty() {
            let copied_package_capacity = self.native_dynamic_packages.len();
            report.copied_packages.reserve(copied_package_capacity);
            let inventory =
                NativePackageInventory::build(plugin_root, &self.native_dynamic_packages)?;
            native::preview_native_dynamic_packages(self, &inventory, output_root, &mut report)?;
        }

        Ok(report)
    }

    pub fn materialize_zip_archive(
        &self,
        plugin_root: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        archive::materialize_zip_archive(self, plugin_root.as_ref(), archive_path.as_ref())
    }

    pub fn preview_zip_archive(
        &self,
        plugin_root: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        archive::preview_zip_archive(self, plugin_root.as_ref(), archive_path.as_ref())
    }

    fn blocked_materialize_report(
        &self,
        fatal_diagnostics: Vec<String>,
    ) -> ExportMaterializeReport {
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.push(format!(
            "export materialization blocked for profile {}: fatal diagnostics must be resolved before writing export files",
            self.profile.name
        ));

        ExportMaterializeReport {
            archive_file: None,
            generated_files: Vec::new(),
            copied_packages: Vec::new(),
            diagnostics,
            fatal_diagnostics,
        }
    }
}
