use std::collections::{HashMap, HashSet};

use crate::asset::project::ProjectPaths;
use crate::asset::{
    ArtifactStore, AssetId, AssetImportError, AssetKind, AssetUri, ImportedAsset, ShaderAsset,
};
use crate::core::framework::render::{
    is_builtin_shader_module_token, is_generated_shader_module_token,
};
use crate::core::resource::ResourceRecord;

#[derive(Clone, Debug)]
struct IndexedShaderImports {
    locator: AssetUri,
    include_path: Option<String>,
    imports: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::asset::project::manager) struct ShaderImportDependencyIndex {
    shaders_by_id: HashMap<AssetId, IndexedShaderImports>,
    includes_by_path: HashMap<String, HashSet<AssetId>>,
    consumers_by_path: HashMap<String, HashSet<AssetId>>,
}

impl ShaderImportDependencyIndex {
    pub(super) fn from_artifacts(
        artifact_store: &ArtifactStore,
        paths: &ProjectPaths,
        imported: &[ResourceRecord],
    ) -> Result<Self, AssetImportError> {
        let mut index = Self::default();
        for record in imported {
            if record.kind != AssetKind::Shader {
                continue;
            }
            let Some(artifact_uri) = record.artifact_locator.as_ref() else {
                continue;
            };
            let ImportedAsset::Shader(shader) = artifact_store.read(paths, artifact_uri)? else {
                continue;
            };
            index.insert(record.id(), record.primary_locator.clone(), &shader);
        }
        Ok(index)
    }

    pub(super) fn append_dependencies(
        &self,
        dependencies_by_id: &mut HashMap<AssetId, Vec<AssetUri>>,
    ) {
        for shader_id in self.shaders_by_id.keys().copied() {
            let dependencies = dependencies_by_id.entry(shader_id).or_default();
            self.append_dependency_locators(shader_id, dependencies);
        }
    }

    pub(super) fn import_path_owners_excluding(
        &self,
        excluded: &HashSet<AssetId>,
    ) -> HashMap<String, AssetUri> {
        self.includes_by_path
            .iter()
            .filter_map(|(path, owners)| {
                owners
                    .iter()
                    .filter(|id| !excluded.contains(id))
                    .filter_map(|id| self.shaders_by_id.get(id))
                    .min_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()))
                    .map(|owner| (path.clone(), owner.locator.clone()))
            })
            .collect()
    }

    pub(super) fn prepare_source_replacement<'a>(
        &self,
        removed_ids: &HashSet<AssetId>,
        ready_shaders: impl IntoIterator<Item = (&'a ResourceRecord, &'a ShaderAsset)>,
    ) -> (Self, HashSet<AssetId>) {
        let ready_shaders = ready_shaders.into_iter();
        let (ready_lower_bound, ready_upper_bound) = ready_shaders.size_hint();
        let ready_capacity = ready_upper_bound.unwrap_or(ready_lower_bound);
        let replacement_path_capacity = removed_ids.len().saturating_add(ready_capacity);
        let mut next = self.clone();
        let mut affected_paths = HashSet::with_capacity(replacement_path_capacity);
        let mut affected_ids = removed_ids.clone();
        for id in removed_ids {
            if let Some(shader) = self.shaders_by_id.get(id) {
                if let Some(path) = &shader.include_path {
                    affected_paths.insert(path.clone());
                }
            }
            next.remove(*id);
        }
        for (record, shader) in ready_shaders {
            affected_ids.insert(record.id());
            if let Some(path) = shader.import_path.as_ref().filter(|path| !path.is_empty()) {
                affected_paths.insert(path.clone());
            }
            next.insert(record.id(), record.primary_locator.clone(), shader);
        }
        for path in affected_paths {
            affected_ids.extend(
                self.consumers_by_path
                    .get(&path)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            affected_ids.extend(
                next.consumers_by_path
                    .get(&path)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
        }
        (next, affected_ids)
    }

    pub(super) fn dependency_locators(&self, id: AssetId) -> Vec<AssetUri> {
        let mut dependencies = Vec::new();
        self.append_dependency_locators(id, &mut dependencies);
        dependencies
    }

    fn append_dependency_locators(&self, id: AssetId, dependencies: &mut Vec<AssetUri>) {
        let Some(shader) = self.shaders_by_id.get(&id) else {
            return;
        };
        dependencies.reserve(shader.imports.len());
        let mut seen_provider_ids = HashSet::with_capacity(shader.imports.len());
        for provider_id in shader.imports.iter().filter_map(|path| {
            let owners = self.includes_by_path.get(path)?;
            if owners.len() != 1 {
                return None;
            }
            owners.iter().next().copied()
        }) {
            if !seen_provider_ids.insert(provider_id) {
                continue;
            }
            let Some(provider) = self.shaders_by_id.get(&provider_id) else {
                continue;
            };
            // Preserve one runtime-owned occurrence even when metadata names the same path.
            // Targeted replacement can then remove only the runtime edge.
            dependencies.push(provider.locator.clone());
        }
    }

    fn insert(&mut self, id: AssetId, locator: AssetUri, shader: &ShaderAsset) {
        self.remove(id);
        let include_path = shader
            .kind
            .is_include()
            .then(|| shader.import_path.clone())
            .flatten()
            .filter(|path| !path.is_empty());
        let imports = shader
            .imports
            .iter()
            .filter(|import| {
                import.redirect.is_none() && !generated_or_builtin_module(&import.source)
            })
            .map(|import| import.source.clone())
            .collect::<Vec<_>>();
        if let Some(path) = &include_path {
            self.includes_by_path
                .entry(path.clone())
                .or_default()
                .insert(id);
        }
        for path in &imports {
            self.consumers_by_path
                .entry(path.clone())
                .or_default()
                .insert(id);
        }
        self.shaders_by_id.insert(
            id,
            IndexedShaderImports {
                locator,
                include_path,
                imports,
            },
        );
    }

    fn remove(&mut self, id: AssetId) {
        let Some(shader) = self.shaders_by_id.remove(&id) else {
            return;
        };
        if let Some(path) = shader.include_path {
            if let Some(owners) = self.includes_by_path.get_mut(&path) {
                owners.remove(&id);
            }
        }
        for path in shader.imports {
            if let Some(consumers) = self.consumers_by_path.get_mut(&path) {
                consumers.remove(&id);
            }
        }
        self.includes_by_path.retain(|_, owners| !owners.is_empty());
        self.consumers_by_path
            .retain(|_, consumers| !consumers.is_empty());
    }
}

fn generated_or_builtin_module(import_path: &str) -> bool {
    is_builtin_shader_module_token(import_path) || is_generated_shader_module_token(import_path)
}

#[cfg(test)]
#[path = "shader_import_dependencies/tests/optimization_batch_ix_runtime634_tests.rs"]
mod optimization_batch_ix_runtime634_tests;

#[cfg(test)]
#[path = "shader_import_dependencies/tests/optimization_batch_runtime867_shader_dependency_direct_append_tests.rs"]
mod optimization_batch_runtime867_shader_dependency_direct_append_tests;

#[cfg(test)]
#[path = "tests/shader_import_dependencies.rs"]
mod tests;
