//! 这些只读访问器向 Runtime 与 Editor 暴露同一项目代的注册表、catalog 和引用诊断；克隆共享 Arc 只延长快照寿命，不触发重新扫描。

use crate::asset::registry::AssetRegistryIndex;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::core::resource::ResourceRegistry;
use crate::plugin::PluginModuleId;
use crate::scene::world::{SceneComponentSerializer, SceneComponentSerializerRegistry};

use super::super::{PackageAssetRegistry, ProjectManifest, ProjectPaths};
use super::ProjectManager;
use crate::asset::AssetImportError;
use std::path::Path;
use std::sync::{Arc, RwLock, Weak};

use super::super::ProjectCatalogInputGeneration;
use super::super::{
    ProjectReferenceDiagnostic, ProjectReferenceDiagnosticPhase, ProjectReferenceDiagnosticsEvent,
    ProjectReferenceDiagnosticsSnapshot,
};

impl ProjectManager {
    /// Returns the codecs currently registered on this project/runtime owner.
    ///
    /// The snapshot is intentionally cloned for a single persistence operation;
    /// registration remains attached to this `ProjectManager` and is shared by its
    /// clones through the interior lock.
    pub(crate) fn scene_component_serializer_registry(&self) -> SceneComponentSerializerRegistry {
        self.scene_component_registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Admit one provider codec and its dynamic World descriptor to this project.
    ///
    /// The registry validates identity, schema version, and duplicates before it
    /// mutates the owner. A failed registration therefore leaves all existing
    /// providers available to subsequent World creation and reopen operations.
    pub(crate) fn validate_scene_component_provider_for_owner(
        &self,
        owner: PluginModuleId,
        serializer: SceneComponentSerializer,
        descriptor: &ComponentTypeDescriptor,
    ) -> Result<(), crate::scene::world::SceneProjectError> {
        self.scene_component_registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .validate_provider_for_owner(owner, serializer, descriptor)
    }

    pub(crate) fn scene_component_registry_weak(
        &self,
    ) -> Weak<RwLock<SceneComponentSerializerRegistry>> {
        Arc::downgrade(&self.scene_component_registry)
    }

    pub(crate) fn scene_component_registry_snapshot(&self) -> SceneComponentSerializerRegistry {
        self.scene_component_registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub(crate) fn restore_scene_component_registry(
        &self,
        snapshot: SceneComponentSerializerRegistry,
    ) {
        *self
            .scene_component_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = snapshot;
    }

    pub(crate) fn revoke_scene_component_provider_owner_from_weak(
        registry: &Weak<RwLock<SceneComponentSerializerRegistry>>,
        owner: PluginModuleId,
    ) {
        if let Some(registry) = registry.upgrade() {
            registry
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .revoke_owner(owner);
        }
    }

    pub(crate) fn register_scene_component_provider(
        &self,
        serializer: SceneComponentSerializer,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), crate::scene::world::SceneProjectError> {
        self.scene_component_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register_provider(serializer, descriptor)
    }

    /// Installs one owner-qualified runtime provider before World creation.  The owner token is
    /// retained with the codec so revocation can remove every callback before native code unload.
    pub(crate) fn register_scene_component_provider_for_owner(
        &self,
        owner: PluginModuleId,
        serializer: SceneComponentSerializer,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), crate::scene::world::SceneProjectError> {
        self.scene_component_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register_provider_for_owner(Some(owner), serializer, descriptor)
    }

    pub(crate) fn revoke_scene_component_provider_owner(&self, owner: PluginModuleId) {
        self.scene_component_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .revoke_owner(owner);
    }

    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn registry(&self) -> &ResourceRegistry {
        &self.registry
    }

    pub fn asset_registry(&self) -> &AssetRegistryIndex {
        &self.asset_registry
    }

    /// Returns the immutable registry generation used by Runtime and Editor projections.
    pub fn asset_registry_shared(&self) -> Arc<AssetRegistryIndex> {
        Arc::clone(&self.asset_registry)
    }

    pub(crate) fn source_resource_records(
        &self,
        locator: &crate::asset::AssetUri,
    ) -> Vec<crate::core::resource::ResourceRecord> {
        self.asset_registry
            .source_entries(locator)
            .into_iter()
            .filter_map(|entry| self.registry.get_by_locator(entry.path()).cloned())
            .collect()
    }

    pub fn package_assets(&self) -> &PackageAssetRegistry {
        &self.package_assets
    }

    pub fn catalog_input_generation(&self) -> Arc<ProjectCatalogInputGeneration> {
        Arc::clone(&self.catalog_input_generation)
    }

    pub(crate) fn replace_reference_diagnostics(
        &self,
        document: crate::asset::AssetUri,
        phase: ProjectReferenceDiagnosticPhase,
        diagnostics: Vec<ProjectReferenceDiagnostic>,
    ) -> ProjectReferenceDiagnosticsEvent {
        self.reference_diagnostics
            .replace_document(document, phase, diagnostics)
    }

    pub fn reference_diagnostics(&self) -> ProjectReferenceDiagnosticsSnapshot {
        self.reference_diagnostics.snapshot()
    }

    pub fn latest_reference_diagnostics_event(&self) -> Option<ProjectReferenceDiagnosticsEvent> {
        self.reference_diagnostics.latest_event()
    }

    pub fn project_asset_roots(&self) -> &[std::path::PathBuf] {
        self.package_assets.project_roots()
    }

    pub(super) fn registry_scan_roots(&self) -> Vec<std::path::PathBuf> {
        self.package_assets
            .project_roots()
            .iter()
            .cloned()
            .chain(
                self.package_assets
                    .iter()
                    .map(|(_, root)| root.to_path_buf()),
            )
            .collect()
    }

    pub fn primary_project_asset_root(&self) -> Result<&Path, AssetImportError> {
        self.package_assets.primary_project_root()
    }

    pub fn project_asset_root_for_source_path(
        &self,
        source_path: &Path,
    ) -> Result<&Path, AssetImportError> {
        self.resolve_project_source_path(source_path)
            .map(|(root, _, _)| root)
    }
}
