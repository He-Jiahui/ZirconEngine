use std::collections::{HashMap, HashSet};

use crate::asset::project::ProjectPaths;
use crate::asset::{
    ArtifactStore, AssetId, AssetImportError, AssetUri, ImportedAsset, ShaderAsset,
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
pub(super) struct ShaderImportDependencyIndex {
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
            for locator in self.dependency_locators(shader_id) {
                // Preserve one runtime-owned occurrence even when metadata names the same path.
                // Targeted replacement can then remove only the runtime edge.
                dependencies.push(locator);
            }
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

    pub(super) fn prepare_source_replacement(
        &self,
        removed_ids: &HashSet<AssetId>,
        ready_payloads: &[(ResourceRecord, ImportedAsset)],
    ) -> (Self, HashSet<AssetId>) {
        let mut next = self.clone();
        let mut affected_paths = HashSet::new();
        let mut affected_ids = removed_ids.clone();
        for id in removed_ids {
            if let Some(shader) = self.shaders_by_id.get(id) {
                if let Some(path) = &shader.include_path {
                    affected_paths.insert(path.clone());
                }
            }
            next.remove(*id);
        }
        for (record, asset) in ready_payloads {
            affected_ids.insert(record.id());
            if let ImportedAsset::Shader(shader) = asset {
                if let Some(path) = shader.import_path.as_ref().filter(|path| !path.is_empty()) {
                    affected_paths.insert(path.clone());
                }
                next.insert(record.id(), record.primary_locator.clone(), shader);
            }
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
        let Some(shader) = self.shaders_by_id.get(&id) else {
            return Vec::new();
        };
        let mut dependencies = Vec::new();
        for locator in shader.imports.iter().filter_map(|path| {
            let owners = self.includes_by_path.get(path)?;
            if owners.len() != 1 {
                return None;
            }
            owners
                .iter()
                .next()
                .and_then(|owner| self.shaders_by_id.get(owner))
                .map(|provider| provider.locator.clone())
        }) {
            if !dependencies.contains(&locator) {
                dependencies.push(locator);
            }
        }
        dependencies
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
