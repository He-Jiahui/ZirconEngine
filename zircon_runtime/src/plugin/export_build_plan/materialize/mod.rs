mod archive;
mod copy;
mod generated;
mod native;
mod native_authority;
mod package_lookup;
mod paths;
mod report;

use std::path::{Path, PathBuf};

use crate::core::framework::project::ExportPackagingStrategy;

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

        self.require_linked_feature_sources_current()?;
        self.require_native_plugin_root()?;
        generated::write_generated_files(self, root.as_ref())
    }

    pub fn materialize(
        &self,
        output_root: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
            self.require_native_plugin_root()?;
        }
        self.materialize_generated_files_after_admission(output_root.as_ref())
    }

    fn materialize_generated_files_after_admission(
        &self,
        output_root: &Path,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let fatal_diagnostics = self.effective_fatal_diagnostics();
        if !fatal_diagnostics.is_empty() {
            return Ok(self.blocked_materialize_report(fatal_diagnostics));
        }

        let generated_files = generated::write_generated_files(self, output_root)?;
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
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
        }
        // Complete role and artifact admission before writing any generated project files.
        let native_packages =
            if self.has_fatal_diagnostics() || self.native_dynamic_packages.is_empty() {
                None
            } else {
                self.ensure_native_authority_path_is_reserved()?;
                let inventory =
                    NativePackageInventory::build(plugin_root, &self.native_dynamic_packages)?;
                archive::validate_native_entry_collisions(self, &inventory)?;
                let authority_bytes =
                    native_authority::embedded_native_authority_bytes(self, &inventory)?;
                Some((inventory, authority_bytes))
            };
        let mut report = self.materialize_generated_files_after_admission(output_root)?;

        if !report.fatal_diagnostics.is_empty() {
            return Ok(report);
        }

        if let Some((inventory, authority_bytes)) = native_packages {
            let copied_package_capacity = self.native_dynamic_packages.len();
            report.copied_packages.reserve(copied_package_capacity);
            native::materialize_native_dynamic_packages(
                self,
                &inventory,
                output_root,
                &mut report,
            )?;
            native_authority::write_embedded_native_authority(
                &authority_bytes,
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
        self.require_valid_admitted_plan_proof()?;
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
            self.require_native_plugin_root()?;
        }
        self.preview_generated_files_after_admission(output_root.as_ref())
    }

    fn preview_generated_files_after_admission(
        &self,
        output_root: &Path,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        let generated_files = generated::preview_generated_files(self, output_root)?;
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
        self.require_valid_admitted_plan_proof()?;
        let plugin_root = plugin_root.as_ref();
        let output_root = output_root.as_ref();
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
        }
        if !self.has_fatal_diagnostics() && !self.native_dynamic_packages.is_empty() {
            self.ensure_native_authority_path_is_reserved()?;
        }
        let mut report = self.preview_generated_files_after_admission(output_root)?;

        if !self.native_dynamic_packages.is_empty() {
            let copied_package_capacity = self.native_dynamic_packages.len();
            report.copied_packages.reserve(copied_package_capacity);
            let inventory =
                NativePackageInventory::build(plugin_root, &self.native_dynamic_packages)?;
            archive::validate_native_entry_collisions(self, &inventory)?;
            native::preview_native_dynamic_packages(self, &inventory, output_root, &mut report)?;
            report
                .generated_files
                .push(output_root.join("src/zircon_native_authority.json"));
        }

        Ok(report)
    }

    fn require_native_plugin_root(&self) -> Result<(), std::io::Error> {
        if self.native_dynamic_packages.is_empty() {
            return Ok(());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "native dynamic packages require a plugin root and native artifact authority admission",
        ))
    }

    fn require_valid_admitted_plan_proof(&self) -> Result<(), std::io::Error> {
        if !self.has_valid_admitted_plan_proof() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "canonical export plan admission is missing or changed; replan from the project manifest",
            ));
        }
        Ok(())
    }

    fn require_linked_feature_sources_current(&self) -> Result<(), std::io::Error> {
        self.require_valid_admitted_plan_proof()?;
        let feature_source = self
            .generated_files
            .iter()
            .find(|file| file.path == "src/zircon_plugins.rs");
        let generated_feature_count = feature_source.map_or(0, |file| {
            file.contents
                .matches("ExportRuntimePluginFeatureRegistrationProvider::new(")
                .count()
        });
        let compile_links = self
            .library_embed_compile_host
            .as_ref()
            .map(|plan| {
                plan.linked_runtime_crates
                    .iter()
                    .filter(|link| {
                        link.registration_kind
                            == super::LibraryEmbedCompileHostTarget::RuntimeFeature
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let expected = generated_feature_count.max(compile_links.len());
        if self.linked_feature_source_count != expected
            || self.linked_feature_sources.len() != expected
            || (feature_source.is_some() && generated_feature_count != expected)
            || (expected > 0
                && self
                    .profile
                    .uses_strategy(ExportPackagingStrategy::SourceTemplate)
                && feature_source.is_none())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "linked feature source receipts missing: expected {}, found {}",
                    expected,
                    self.linked_feature_sources.len()
                ),
            ));
        }
        if expected == 0 {
            return Ok(());
        }
        let cargo = feature_source
            .map(|_| {
                let cargo_source = self
                    .generated_files
                    .iter()
                    .find(|file| file.path == "Cargo.toml")
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "linked feature Cargo manifest is missing",
                        )
                    })?;
                toml::from_str::<toml::Value>(&cargo_source.contents).map_err(|error| {
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("linked feature Cargo manifest is invalid: {error}"),
                    )
                })
            })
            .transpose()?;
        let mut seen = std::collections::HashSet::with_capacity(expected);
        for receipt in &self.linked_feature_sources {
            let expected_provider_call = receipt.generated_provider_call();
            let expected_cargo_path = receipt.generated_cargo_path();
            if !seen.insert(receipt.crate_name())
                || !self
                    .linked_runtime_crates
                    .iter()
                    .any(|name| name.as_str() == receipt.crate_name())
                || feature_source.is_some_and(|source| {
                    source
                        .contents
                        .matches(expected_provider_call.as_str())
                        .count()
                        != 1
                })
                || (!compile_links.is_empty()
                    && !compile_links
                        .iter()
                        .any(|link| receipt.matches_library_embed_link(link)))
                || cargo.as_ref().is_some_and(|cargo| {
                    cargo
                        .get("dependencies")
                        .and_then(|deps| deps.get(receipt.crate_name()))
                        .and_then(|dependency| dependency.get("path"))
                        .and_then(toml::Value::as_str)
                        != Some(expected_cargo_path.as_str())
                })
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "linked feature source receipts do not match generated provider and Cargo bindings",
                ));
            }
            receipt.verify_current()?;
        }
        Ok(())
    }

    fn ensure_native_authority_path_is_reserved(&self) -> Result<(), std::io::Error> {
        const AUTHORITY_PATH: &str = "src/zircon_native_authority.json";
        for file in &self.generated_files {
            if paths::validated_materialized_relative_path(&file.path)?
                .eq_ignore_ascii_case(AUTHORITY_PATH)
            {
                return Err(std::io::Error::other(
                    "generated native authority path is reserved",
                ));
            }
        }
        Ok(())
    }

    pub fn materialize_zip_archive(
        &self,
        plugin_root: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
        }
        archive::materialize_zip_archive(self, plugin_root.as_ref(), archive_path.as_ref())
    }

    pub fn preview_zip_archive(
        &self,
        plugin_root: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
    ) -> Result<ExportMaterializeReport, std::io::Error> {
        self.require_valid_admitted_plan_proof()?;
        if !self.has_fatal_diagnostics() {
            self.require_linked_feature_sources_current()?;
        }
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

#[cfg(test)]
#[path = "tests/mod_native_preflight_tests.rs"]
mod native_preflight_tests;
