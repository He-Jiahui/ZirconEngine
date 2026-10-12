use std::path::Path;

use libloading::Library;

use super::candidate_from_manifest::native_library_paths_for_candidate;
use super::compatibility::native_distribution_compatibility_diagnostic;
use super::native_plugin_abi::{
    call_native_plugin_entry, probe_native_plugin_descriptor, NativePluginDescriptor,
    NativePluginEntryReport,
};
use super::plugin_load_error::{
    PluginLoadError, PluginLoadResult, PluginLoadStage, ABI_CONTRACT_HINT,
};
use super::{
    LoadedNativePlugin, NativePluginArtifactAuthority, NativePluginCandidate,
    NativePluginLoadReport, NativePluginLoader,
};
use crate::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

#[derive(Clone, Copy)]
struct RequestedModuleKinds(u8);

impl RequestedModuleKinds {
    fn from_slice(module_kinds: &[PluginModuleKind]) -> Self {
        let mut bits = 0;
        for module_kind in module_kinds {
            bits |= Self::bit(*module_kind);
        }
        Self(bits)
    }

    fn contains(self, module_kind: PluginModuleKind) -> bool {
        self.0 & Self::bit(module_kind) != 0
    }

    const fn bit(module_kind: PluginModuleKind) -> u8 {
        match module_kind {
            PluginModuleKind::Runtime => 0b0001,
            PluginModuleKind::Editor => 0b0010,
            PluginModuleKind::Native => 0b0100,
            PluginModuleKind::Vm => 0b1000,
        }
    }
}

impl NativePluginLoader {
    /// Checks discovered package metadata and expected native artifact paths without opening a
    /// candidate DLL. Export assembly uses this before product launch; in-process execution
    /// remains owned by the authority-aware load methods below.
    pub fn validate_discovered_runtime(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.validate_candidates_for_module_kinds(report, &[PluginModuleKind::Runtime])
    }

    /// Checks discovered editor package metadata and expected native artifact paths without
    /// admitting or executing a candidate DLL.
    pub fn validate_discovered_editor(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.validate_candidates_for_module_kinds(report, &[PluginModuleKind::Editor])
    }

    pub fn load_discovered_all(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        self.load_discovered_all_with_authority(root, &NativePluginArtifactAuthority::deny_all())
    }

    pub fn load_discovered_all_with_authority(
        &self,
        root: impl AsRef<Path>,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_all_candidates(report, authority)
    }

    pub fn load_discovered_runtime(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        self.load_discovered_runtime_with_authority(
            root,
            &NativePluginArtifactAuthority::deny_all(),
        )
    }

    pub fn load_discovered_runtime_with_authority(
        &self,
        root: impl AsRef<Path>,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_candidates_for_module_kinds(report, &[PluginModuleKind::Runtime], authority)
    }

    pub fn load_discovered_editor(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        self.load_discovered_editor_with_authority(root, &NativePluginArtifactAuthority::deny_all())
    }

    pub fn load_discovered_editor_with_authority(
        &self,
        root: impl AsRef<Path>,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_candidates_for_module_kinds(report, &[PluginModuleKind::Editor], authority)
    }

    pub(super) fn load_all_candidates(
        &self,
        report: NativePluginLoadReport,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginLoadReport {
        self.load_candidates_for_module_kinds(
            report,
            &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
            authority,
        )
    }

    pub(super) fn load_candidates_for_module_kinds(
        &self,
        mut report: NativePluginLoadReport,
        module_kinds: &[PluginModuleKind],
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginLoadReport {
        let discovered = report.take_discovered();
        let requested_module_kinds = RequestedModuleKinds::from_slice(module_kinds);
        for candidate in &discovered {
            if !package_matches_module_kinds(&candidate.package_manifest, requested_module_kinds) {
                continue;
            }
            if let Some(diagnostic) = native_distribution_compatibility_diagnostic(
                &candidate.plugin_id,
                &candidate.package_manifest,
            ) {
                report.push_diagnostic(diagnostic);
                continue;
            }
            for (library_path, library_module_kinds) in
                native_library_paths_for_candidate(&candidate, module_kinds)
            {
                load_candidate_library(
                    &mut report,
                    candidate,
                    library_path,
                    &library_module_kinds,
                    authority,
                );
            }
        }
        report.restore_discovered(discovered);
        report
    }

    pub(super) fn validate_candidates_for_module_kinds(
        &self,
        mut report: NativePluginLoadReport,
        module_kinds: &[PluginModuleKind],
    ) -> NativePluginLoadReport {
        let discovered = report.take_discovered();
        let requested_module_kinds = RequestedModuleKinds::from_slice(module_kinds);
        for candidate in &discovered {
            if !package_matches_module_kinds(&candidate.package_manifest, requested_module_kinds) {
                continue;
            }
            if let Some(diagnostic) = native_distribution_compatibility_diagnostic(
                &candidate.plugin_id,
                &candidate.package_manifest,
            ) {
                report.push_diagnostic(diagnostic);
                continue;
            }
            for (library_path, _) in native_library_paths_for_candidate(candidate, module_kinds) {
                if !library_path.exists() {
                    report.push_diagnostic(
                        PluginLoadError::missing_artifact(
                            &candidate.plugin_id,
                            &library_path,
                            "native dist library",
                        )
                        .to_string(),
                    );
                }
            }
        }
        report.restore_discovered(discovered);
        report
    }
}

fn load_candidate_library(
    report: &mut NativePluginLoadReport,
    candidate: &NativePluginCandidate,
    library_path: std::path::PathBuf,
    module_kinds: &[PluginModuleKind],
    authority: &NativePluginArtifactAuthority,
) {
    let plugin_id = &candidate.plugin_id;
    if !library_path.exists() {
        report.push_diagnostic(
            PluginLoadError::missing_artifact(plugin_id, &library_path, "native dist library")
                .to_string(),
        );
        return;
    }
    let admission_receipt = match authority.admit(candidate, &library_path, module_kinds) {
        Ok(receipt) => receipt,
        Err(error) => {
            report.push_diagnostic(error.to_string());
            return;
        }
    };
    match unsafe { load_admitted_library(&admission_receipt) } {
        Ok(library) => {
            let descriptor =
                match unsafe { probe_native_plugin_descriptor(&library, &library_path, plugin_id) }
                {
                    Ok(descriptor) => descriptor,
                    Err(error) => {
                        report.push_diagnostic(error.to_string());
                        return;
                    }
                };
            let runtime_entry_report = retain_requested_entry(
                report,
                load_requested_entry(
                    &library,
                    &library_path,
                    plugin_id,
                    module_kinds,
                    PluginModuleKind::Runtime,
                    &descriptor,
                ),
            );
            let editor_entry_report = retain_requested_entry(
                report,
                load_requested_entry(
                    &library,
                    &library_path,
                    plugin_id,
                    module_kinds,
                    PluginModuleKind::Editor,
                    &descriptor,
                ),
            );
            report.push_loaded(LoadedNativePlugin {
                plugin_id: plugin_id.to_string(),
                library_path,
                descriptor: Some(descriptor),
                runtime_entry_report,
                editor_entry_report,
                library: LoadedNativePlugin::admitted_library(library, admission_receipt),
            });
        }
        Err(error) => report.push_diagnostic(
            PluginLoadError::library_open(plugin_id, &library_path, error).to_string(),
        ),
    }
}

unsafe fn load_admitted_library(
    receipt: &super::NativePluginArtifactAdmissionReceipt,
) -> Result<Library, libloading::Error> {
    #[cfg(windows)]
    {
        use libloading::os::windows::Library as WindowsLibrary;
        const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;
        const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x0000_0800;
        let library = WindowsLibrary::load_with_flags(
            receipt.admitted_library_path(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )?;
        return Ok(library.into());
    }
    #[cfg(not(windows))]
    {
        Library::new(receipt.admitted_library_path())
    }
}

fn retain_requested_entry<T>(
    report: &mut NativePluginLoadReport,
    entry: PluginLoadResult<Option<T>>,
) -> Option<T> {
    match entry {
        Ok(entry) => entry,
        Err(error) => {
            report.push_diagnostic(error.to_string());
            None
        }
    }
}

fn load_requested_entry(
    library: &Library,
    library_path: &Path,
    plugin_id: &str,
    module_kinds: &[PluginModuleKind],
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> PluginLoadResult<Option<NativePluginEntryReport>> {
    if !module_kinds.contains(&module_kind) {
        return Ok(None);
    }
    let entry_name = match module_kind {
        PluginModuleKind::Runtime => descriptor.runtime_entry_name.as_deref(),
        PluginModuleKind::Editor => descriptor.editor_entry_name.as_deref(),
        PluginModuleKind::Native | PluginModuleKind::Vm => return Ok(None),
    }
    .ok_or_else(|| {
        PluginLoadError::contract_mismatch(
            plugin_id,
            PluginLoadStage::from(module_kind),
            "descriptor.entry_symbol",
            "entry symbol name",
            "missing",
            library_path,
            ABI_CONTRACT_HINT,
        )
    })?;
    unsafe {
        call_native_plugin_entry(
            library,
            library_path,
            entry_name,
            plugin_id,
            module_kind,
            descriptor,
        )
    }
    .map(Some)
}

fn package_matches_module_kinds(
    package_manifest: &PluginPackageManifest,
    requested_module_kinds: RequestedModuleKinds,
) -> bool {
    package_manifest
        .modules
        .iter()
        .any(|module| requested_module_kinds.contains(module.kind))
        || package_manifest
            .feature_extensions
            .iter()
            .flat_map(|feature| feature.modules.iter())
            .any(|module| requested_module_kinds.contains(module.kind))
}

#[cfg(test)]
#[path = "tests/load_discovered.rs"]
mod tests;
