use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

use zircon_runtime::core::framework::render::ShaderAssetKind;

use super::super::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};
use super::revision::asset_scan_revision_from_base_revision_and_content_hashes;
use super::ShaderPrewarmSource;

#[cfg(test)]
pub(super) fn shader_sources_with_module_dependency_hashes(
    sources: Vec<ShaderPrewarmSource>,
    external_include_modules: &BTreeMap<String, String>,
) -> Vec<ShaderPrewarmSource> {
    shader_sources_with_module_dependency_hashes_and_changed_paths(
        sources,
        external_include_modules,
        &BTreeSet::new(),
    )
    .expect("test include graph must satisfy its internal invariants")
    .sources
}

pub(super) struct ShaderPrewarmSourceDependencyBatch {
    pub(super) sources: Vec<ShaderPrewarmSource>,
    pub(super) affected_source_indices: BTreeSet<usize>,
}

/// Hashes the whole compact include graph once, then projects a file-level
/// inventory delta through reverse import edges for incremental prewarm work.
pub(super) fn shader_sources_with_module_dependency_hashes_and_changed_paths(
    mut sources: Vec<ShaderPrewarmSource>,
    external_include_modules: &BTreeMap<String, String>,
    changed_paths: &BTreeSet<PathBuf>,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSourceDependencyBatch> {
    let dag = IndexedIncludeDag::new(&sources, external_include_modules);
    let analysis = dag.analyze();
    let affected_source_indices =
        dag.reverse_changed_source_closure(&sources, changed_paths, &analysis);
    let topology_hashes_by_source = dag.topology_hashes_by_source(&sources, &analysis)?;
    for (source, topology_hash) in sources.iter_mut().zip(topology_hashes_by_source) {
        let Some(topology_hash) = topology_hash else {
            continue;
        };
        source.revision = asset_scan_revision_from_base_revision_and_content_hashes(
            source.revision,
            std::slice::from_ref(&topology_hash),
        );
        source.include_content_hashes.push(topology_hash);
    }

    Ok(ShaderPrewarmSourceDependencyBatch {
        sources,
        affected_source_indices,
    })
}

struct IndexedIncludeDag {
    external_content_hashes: Vec<String>,
    imports_by_source: Vec<Vec<IndexedIncludeModule>>,
}

impl IndexedIncludeDag {
    fn new(
        sources: &[ShaderPrewarmSource],
        external_include_modules: &BTreeMap<String, String>,
    ) -> Self {
        let mut external_content_hashes = Vec::with_capacity(external_include_modules.len());
        let mut include_modules =
            HashMap::with_capacity(external_include_modules.len() + sources.len());
        for (external_index, (import_path, content_hash)) in
            external_include_modules.iter().enumerate()
        {
            external_content_hashes.push(content_hash.clone());
            include_modules.insert(
                import_path.as_str(),
                IndexedIncludeModule::External(external_index),
            );
        }
        include_modules.extend(sources.iter().enumerate().filter_map(|(index, source)| {
            (source.kind == ShaderAssetKind::Include)
                .then(|| source.import_path.as_deref().map(|path| (path, index)))
                .flatten()
                .map(|(path, index)| (path, IndexedIncludeModule::Local(index)))
        }));
        let imports_by_source = sources
            .iter()
            .map(|source| {
                let mut seen = HashSet::new();
                source
                    .imports
                    .iter()
                    .filter_map(|import_path| include_modules.get(import_path.as_str()).copied())
                    .filter(|module| seen.insert(*module))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Self {
            external_content_hashes,
            imports_by_source,
        }
    }

    fn analyze(&self) -> IndexedIncludeAnalysis {
        let (components, component_for_source) = self.strongly_connected_components();
        let graph = self.component_graph(&component_for_source, components.len());
        IndexedIncludeAnalysis {
            components,
            component_for_source,
            graph,
        }
    }

    /// Produces one compact dependency identity per source in O(V + E) work.
    ///
    /// SCC compression gives every import cycle a stable component identity;
    /// the condensed graph is acyclic, so component hashes are evaluated once
    /// in dependency order instead of cloning a transitive closure for every
    /// source variant.
    fn topology_hashes_by_source(
        &self,
        sources: &[ShaderPrewarmSource],
        analysis: &IndexedIncludeAnalysis,
    ) -> ShaderPrewarmAssetScanResult<Vec<Option<String>>> {
        let component_hashes = self.component_hashes(
            sources,
            &analysis.components,
            &analysis.component_for_source,
            &analysis.graph.dependencies,
        )?;
        Ok(self
            .imports_by_source
            .iter()
            .enumerate()
            .map(|(source_index, imports)| {
                (!imports.is_empty())
                    .then(|| component_hashes[analysis.component_for_source[source_index]].clone())
            })
            .collect())
    }

    fn reverse_changed_source_closure(
        &self,
        sources: &[ShaderPrewarmSource],
        changed_paths: &BTreeSet<PathBuf>,
        analysis: &IndexedIncludeAnalysis,
    ) -> BTreeSet<usize> {
        let directly_changed_sources = sources
            .iter()
            .enumerate()
            .filter_map(|(source_index, source)| {
                source
                    .source_paths
                    .iter()
                    .any(|path| changed_paths.contains(path))
                    .then_some(source_index)
            })
            .collect::<Vec<_>>();
        if directly_changed_sources.is_empty() {
            return BTreeSet::new();
        }
        let affected_components = analysis.graph.reverse_changed_closure(
            directly_changed_sources
                .into_iter()
                .map(|source_index| analysis.component_for_source[source_index]),
        );
        let affected_components = affected_components.into_iter().collect::<HashSet<_>>();
        analysis
            .component_for_source
            .iter()
            .enumerate()
            .filter_map(|(source_index, component)| {
                affected_components
                    .contains(component)
                    .then_some(source_index)
            })
            .collect()
    }

    fn strongly_connected_components(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        let source_count = self.imports_by_source.len();
        let mut reverse_edges = vec![Vec::new(); source_count];
        for (source_index, imports) in self.imports_by_source.iter().enumerate() {
            for import in imports {
                if let IndexedIncludeModule::Local(dependency_index) = import {
                    reverse_edges[*dependency_index].push(source_index);
                }
            }
        }

        let mut visited = vec![false; source_count];
        let mut finish_order = Vec::with_capacity(source_count);
        for root in 0..source_count {
            if visited[root] {
                continue;
            }
            let mut stack = vec![(root, false)];
            while let Some((source_index, leaving)) = stack.pop() {
                if leaving {
                    finish_order.push(source_index);
                    continue;
                }
                if visited[source_index] {
                    continue;
                }
                visited[source_index] = true;
                stack.push((source_index, true));
                for import in self.imports_by_source[source_index].iter().rev() {
                    if let IndexedIncludeModule::Local(dependency_index) = import {
                        if !visited[*dependency_index] {
                            stack.push((*dependency_index, false));
                        }
                    }
                }
            }
        }

        let mut component_for_source = vec![usize::MAX; source_count];
        let mut components = Vec::new();
        for root in finish_order.into_iter().rev() {
            if component_for_source[root] != usize::MAX {
                continue;
            }
            let component_index = components.len();
            let mut members = Vec::new();
            let mut stack = vec![root];
            while let Some(source_index) = stack.pop() {
                if component_for_source[source_index] != usize::MAX {
                    continue;
                }
                component_for_source[source_index] = component_index;
                members.push(source_index);
                for dependent_index in reverse_edges[source_index].iter().rev() {
                    if component_for_source[*dependent_index] == usize::MAX {
                        stack.push(*dependent_index);
                    }
                }
            }
            members.sort_unstable();
            components.push(members);
        }
        (components, component_for_source)
    }

    fn component_graph(
        &self,
        component_for_source: &[usize],
        component_count: usize,
    ) -> IndexedIncludeComponentGraph {
        let mut dependencies = vec![Vec::new(); component_count];
        let mut seen_dependencies = vec![HashSet::new(); component_count];
        for (source_index, imports) in self.imports_by_source.iter().enumerate() {
            let component_index = component_for_source[source_index];
            for import in imports {
                let dependency = match import {
                    IndexedIncludeModule::Local(dependency_index) => {
                        let dependency_component = component_for_source[*dependency_index];
                        (dependency_component != component_index).then_some(
                            IndexedIncludeComponentDependency::Local(dependency_component),
                        )
                    }
                    IndexedIncludeModule::External(external_index) => {
                        Some(IndexedIncludeComponentDependency::External(*external_index))
                    }
                };
                if let Some(dependency) = dependency {
                    if seen_dependencies[component_index].insert(dependency) {
                        dependencies[component_index].push(dependency);
                    }
                }
            }
        }
        let mut reverse_dependents = vec![Vec::new(); component_count];
        for (component_index, component_dependencies) in dependencies.iter().enumerate() {
            for dependency in component_dependencies {
                if let IndexedIncludeComponentDependency::Local(dependency_component) = dependency {
                    reverse_dependents[*dependency_component].push(component_index);
                }
            }
        }
        IndexedIncludeComponentGraph {
            dependencies,
            reverse_dependents,
        }
    }

    fn component_hashes(
        &self,
        sources: &[ShaderPrewarmSource],
        components: &[Vec<usize>],
        component_for_source: &[usize],
        dependencies: &[Vec<IndexedIncludeComponentDependency>],
    ) -> ShaderPrewarmAssetScanResult<Vec<String>> {
        let mut hashes = vec![None; components.len()];
        let mut visiting = vec![false; components.len()];
        for root in 0..components.len() {
            if hashes[root].is_some() {
                continue;
            }
            let mut stack = vec![(root, false)];
            while let Some((component_index, leaving)) = stack.pop() {
                if leaving {
                    hashes[component_index] = Some(self.component_hash(
                        sources,
                        &components[component_index],
                        component_for_source,
                        &hashes,
                    )?);
                    visiting[component_index] = false;
                    continue;
                }
                if hashes[component_index].is_some() || visiting[component_index] {
                    continue;
                }
                visiting[component_index] = true;
                stack.push((component_index, true));
                for dependency in dependencies[component_index].iter().rev() {
                    if let IndexedIncludeComponentDependency::Local(dependency_component) =
                        dependency
                    {
                        if hashes[*dependency_component].is_none() {
                            stack.push((*dependency_component, false));
                        }
                    }
                }
            }
        }
        hashes.into_iter().collect::<Option<Vec<_>>>().ok_or(
            ShaderPrewarmAssetScanError::IncludeDependencyGraphInvariant {
                detail: "a condensed component did not receive a topology hash",
            },
        )
    }

    fn component_hash(
        &self,
        sources: &[ShaderPrewarmSource],
        members: &[usize],
        component_for_source: &[usize],
        component_hashes: &[Option<String>],
    ) -> ShaderPrewarmAssetScanResult<String> {
        let mut hasher = blake3::Hasher::new();
        hash_field(&mut hasher, b"zircon-prewarm-include-topology-v1");
        for source_index in members {
            let source = &sources[*source_index];
            hash_field(&mut hasher, source.stable_label.as_bytes());
            for source_hash in &source.include_content_hashes {
                hash_field(&mut hasher, source_hash.as_bytes());
            }
            for import in &self.imports_by_source[*source_index] {
                match import {
                    IndexedIncludeModule::Local(dependency_index) => {
                        let dependency_component = component_for_source[*dependency_index];
                        if dependency_component == component_for_source[*source_index] {
                            hash_field(&mut hasher, b"local-cycle-member");
                            hash_field(
                                &mut hasher,
                                sources[*dependency_index].stable_label.as_bytes(),
                            );
                        } else {
                            hash_field(&mut hasher, b"local-component");
                            let dependency_hash = component_hashes
                                .get(dependency_component)
                                .and_then(Option::as_ref)
                                .ok_or(
                                    ShaderPrewarmAssetScanError::IncludeDependencyGraphInvariant {
                                        detail: "a dependency component was not hashed first",
                                    },
                                )?;
                            hash_field(&mut hasher, dependency_hash.as_bytes());
                        }
                    }
                    IndexedIncludeModule::External(external_index) => {
                        hash_field(&mut hasher, b"external-module");
                        let external_hash =
                            self.external_content_hashes.get(*external_index).ok_or(
                                ShaderPrewarmAssetScanError::IncludeDependencyGraphInvariant {
                                    detail: "an external include edge had no interned content hash",
                                },
                            )?;
                        hash_field(&mut hasher, external_hash.as_bytes());
                    }
                }
            }
        }
        Ok(hasher.finalize().to_hex().to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum IndexedIncludeModule {
    Local(usize),
    External(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum IndexedIncludeComponentDependency {
    Local(usize),
    External(usize),
}

struct IndexedIncludeAnalysis {
    components: Vec<Vec<usize>>,
    component_for_source: Vec<usize>,
    graph: IndexedIncludeComponentGraph,
}

struct IndexedIncludeComponentGraph {
    dependencies: Vec<Vec<IndexedIncludeComponentDependency>>,
    reverse_dependents: Vec<Vec<usize>>,
}

impl IndexedIncludeComponentGraph {
    /// Returns the changed component plus every source component that imports it.
    ///
    /// A persistent warm inventory can reuse this compact reverse closure to
    /// recompute only affected topology hashes after a one-percent edit.
    fn reverse_changed_closure(
        &self,
        changed_components: impl IntoIterator<Item = usize>,
    ) -> Vec<usize> {
        let mut changed = vec![false; self.reverse_dependents.len()];
        let mut pending = changed_components
            .into_iter()
            .filter(|component| *component < changed.len())
            .collect::<Vec<_>>();
        while let Some(component) = pending.pop() {
            if changed[component] {
                continue;
            }
            changed[component] = true;
            pending.extend(self.reverse_dependents[component].iter().copied());
        }
        changed
            .into_iter()
            .enumerate()
            .filter_map(|(component, changed)| changed.then_some(component))
            .collect()
    }
}

fn hash_field(hasher: &mut blake3::Hasher, field: &[u8]) {
    hasher.update(&(field.len() as u64).to_le_bytes());
    hasher.update(field);
}

#[cfg(test)]
#[path = "tests/module_dependencies_unit.rs"]
mod tests;
