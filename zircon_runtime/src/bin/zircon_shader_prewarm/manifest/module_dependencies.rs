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
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::{MaterialOptionTable, ShaderPassType};
    use zircon_runtime::core::resource::ResourceId;

    fn source(
        stable_label: &str,
        kind: ShaderAssetKind,
        import_path: Option<&str>,
        imports: &[&str],
        content_hash: &str,
    ) -> ShaderPrewarmSource {
        ShaderPrewarmSource {
            source_paths: vec![PathBuf::from(stable_label)],
            stable_label: stable_label.to_string(),
            resource_id: ResourceId::from_stable_label(stable_label),
            revision: 1,
            wgsl_source: String::new(),
            include_content_hashes: vec![content_hash.to_string()],
            pass_types: vec![ShaderPassType::Forward],
            kind,
            import_path: import_path.map(ToOwned::to_owned),
            imports: imports.iter().map(|import| (*import).to_string()).collect(),
            material_layout_hash: 0,
            material_option_table: MaterialOptionTable::default(),
        }
    }

    #[test]
    fn indexed_include_dag_compacts_diamond_and_cyclic_module_closures() {
        let diamond_sources = vec![
            source(
                "root",
                ShaderAssetKind::Surface,
                None,
                &["left", "right"],
                "root-hash",
            ),
            source(
                "left",
                ShaderAssetKind::Include,
                Some("left"),
                &["leaf"],
                "left-hash",
            ),
            source(
                "right",
                ShaderAssetKind::Include,
                Some("right"),
                &["leaf"],
                "right-hash",
            ),
            source(
                "leaf",
                ShaderAssetKind::Include,
                Some("leaf"),
                &[],
                "leaf-hash",
            ),
        ];
        let diamond =
            shader_sources_with_module_dependency_hashes(diamond_sources.clone(), &BTreeMap::new());
        let diamond_second =
            shader_sources_with_module_dependency_hashes(diamond_sources, &BTreeMap::new());
        assert_eq!(
            diamond[0].include_content_hashes.len(),
            2,
            "a shared closure is represented by one topology hash instead of cloned hashes"
        );
        assert_eq!(diamond[0].include_content_hashes[0], "root-hash");
        assert_eq!(diamond[0].include_content_hashes[1].len(), 64);
        assert_eq!(
            diamond[0].include_content_hashes, diamond_second[0].include_content_hashes,
            "diamond topology hashes must remain deterministic"
        );

        let cyclic_sources = vec![
            source("root", ShaderAssetKind::Surface, None, &["a"], "root-hash"),
            source("a", ShaderAssetKind::Include, Some("a"), &["b"], "a-hash"),
            source("b", ShaderAssetKind::Include, Some("b"), &["a"], "b-hash"),
        ];
        let first =
            shader_sources_with_module_dependency_hashes(cyclic_sources.clone(), &BTreeMap::new());
        let second = shader_sources_with_module_dependency_hashes(cyclic_sources, &BTreeMap::new());
        assert_eq!(
            first[0].include_content_hashes.len(),
            2,
            "a cycle must be represented by one SCC topology hash"
        );
        assert_eq!(
            first[0].include_content_hashes, second[0].include_content_hashes,
            "cycle handling must remain deterministic"
        );

        let root_cyclic_sources = vec![
            source("root", ShaderAssetKind::Surface, None, &["a"], "root-hash"),
            source(
                "a",
                ShaderAssetKind::Include,
                Some("a"),
                &["root"],
                "a-hash",
            ),
        ];
        let root_cycle =
            shader_sources_with_module_dependency_hashes(root_cyclic_sources, &BTreeMap::new());
        assert_eq!(
            root_cycle[0].include_content_hashes.len(),
            2,
            "a nested cycle must not expand into a transitive source-hash closure"
        );
    }

    #[test]
    fn indexed_include_dag_interns_external_hashes_for_high_fanout_imports() {
        const SOURCE_COUNT: usize = 128;
        let sources = (0..SOURCE_COUNT)
            .map(|source_index| {
                source(
                    &format!("surface-{source_index}"),
                    ShaderAssetKind::Surface,
                    None,
                    &["engine::shared"],
                    "surface-hash",
                )
            })
            .collect::<Vec<_>>();
        let external_modules = BTreeMap::from([(
            "engine::shared".to_string(),
            "shared-module-content-hash".to_string(),
        )]);

        let dag = IndexedIncludeDag::new(&sources, &external_modules);
        assert_eq!(
            dag.external_content_hashes,
            ["shared-module-content-hash".to_string()],
            "an external module hash must be stored once for the complete batch"
        );
        assert!(
            dag.imports_by_source
                .iter()
                .all(|imports| imports.as_slice() == [IndexedIncludeModule::External(0)]),
            "each high-fanout edge must retain only the interned module index"
        );

        let (components, component_for_source) = dag.strongly_connected_components();
        let graph = dag.component_graph(&component_for_source, components.len());
        assert!(
            graph.dependencies.iter().all(|dependencies| {
                dependencies.as_slice() == [IndexedIncludeComponentDependency::External(0)]
            }),
            "the condensed graph must keep external dependencies as scalar indexes"
        );
    }

    #[test]
    fn indexed_include_dag_represents_deep_shared_layers_with_one_topology_hash() {
        const LAYER_COUNT: usize = 16;
        let mut sources = Vec::with_capacity(LAYER_COUNT + 1);
        sources.push(source(
            "root",
            ShaderAssetKind::Surface,
            None,
            &["layer-15", "layer-15"],
            "root-hash",
        ));
        for layer in (0..LAYER_COUNT).rev() {
            let import_path = format!("layer-{layer}");
            let imports = (layer > 0)
                .then(|| format!("layer-{}", layer - 1))
                .into_iter()
                .collect::<Vec<_>>();
            let duplicated_imports = imports
                .iter()
                .flat_map(|import| [import.as_str(), import.as_str()])
                .collect::<Vec<_>>();
            let content_hash = format!("layer-{layer}-hash");
            sources.push(source(
                &import_path,
                ShaderAssetKind::Include,
                Some(&import_path),
                &duplicated_imports,
                &content_hash,
            ));
        }

        let resolved = shader_sources_with_module_dependency_hashes(sources, &BTreeMap::new());
        assert_eq!(
            resolved[0].include_content_hashes.len(),
            2,
            "deep dependency chains must not allocate a per-root transitive hash list"
        );
        assert_eq!(resolved[0].include_content_hashes[0], "root-hash");
        assert_eq!(resolved[0].include_content_hashes[1].len(), 64);
    }

    #[test]
    fn indexed_include_dag_keeps_layered_shared_diamonds_linear_in_node_count() {
        const LAYER_COUNT: usize = 48;
        let mut sources = Vec::with_capacity(1 + LAYER_COUNT * 2);
        sources.push(source(
            "root",
            ShaderAssetKind::Surface,
            None,
            &["layer-0-left", "layer-0-right"],
            "root-hash",
        ));
        for layer in 0..LAYER_COUNT {
            let left = format!("layer-{layer}-left");
            let right = format!("layer-{layer}-right");
            let next_left = format!("layer-{}-left", layer + 1);
            let next_right = format!("layer-{}-right", layer + 1);
            let imports = if layer + 1 < LAYER_COUNT {
                vec![next_left.as_str(), next_right.as_str()]
            } else {
                Vec::new()
            };
            for (label, content_hash) in [
                (left.as_str(), format!("{left}-hash")),
                (right.as_str(), format!("{right}-hash")),
            ] {
                sources.push(source(
                    label,
                    ShaderAssetKind::Include,
                    Some(label),
                    &imports,
                    &content_hash,
                ));
            }
        }

        let resolved = shader_sources_with_module_dependency_hashes(sources, &BTreeMap::new());

        assert_eq!(resolved.len(), 1 + LAYER_COUNT * 2);
        assert_eq!(
            resolved[0].include_content_hashes.len(),
            2,
            "a layered shared DAG must contribute one compact topology hash, not a transitive path vector"
        );
        assert_eq!(resolved[0].include_content_hashes[1].len(), 64);
    }

    #[test]
    fn indexed_include_dag_tracks_reverse_dependents_for_a_changed_leaf() {
        let sources = vec![
            source(
                "root",
                ShaderAssetKind::Surface,
                None,
                &["left", "right"],
                "root-hash",
            ),
            source(
                "left",
                ShaderAssetKind::Include,
                Some("left"),
                &["leaf"],
                "left-hash",
            ),
            source(
                "right",
                ShaderAssetKind::Include,
                Some("right"),
                &["leaf"],
                "right-hash",
            ),
            source(
                "leaf",
                ShaderAssetKind::Include,
                Some("leaf"),
                &[],
                "leaf-hash",
            ),
        ];
        let dag = IndexedIncludeDag::new(&sources, &BTreeMap::new());
        let (components, component_for_source) = dag.strongly_connected_components();
        let graph = dag.component_graph(&component_for_source, components.len());
        let affected_components = graph.reverse_changed_closure([component_for_source[3]]);
        let affected_sources = component_for_source
            .iter()
            .enumerate()
            .filter_map(|(source_index, component)| {
                affected_components
                    .contains(component)
                    .then_some(source_index)
            })
            .collect::<Vec<_>>();

        assert_eq!(affected_sources, [0, 1, 2, 3]);
    }

    #[test]
    fn runtime91_indexed_include_dag_projects_inventory_file_changes_to_reverse_source_closure() {
        let sources = vec![
            source(
                "root",
                ShaderAssetKind::Surface,
                None,
                &["leaf"],
                "root-hash",
            ),
            source(
                "leaf",
                ShaderAssetKind::Include,
                Some("leaf"),
                &[],
                "leaf-hash",
            ),
            source(
                "unrelated",
                ShaderAssetKind::Surface,
                None,
                &[],
                "unrelated-hash",
            ),
        ];
        let changed_paths = BTreeSet::from([PathBuf::from("leaf")]);

        let batch = shader_sources_with_module_dependency_hashes_and_changed_paths(
            sources,
            &BTreeMap::new(),
            &changed_paths,
        )
        .expect("test include graph must satisfy its internal invariants");

        assert_eq!(
            batch.affected_source_indices,
            BTreeSet::from([0, 1]),
            "a changed include must schedule itself and every reverse dependent, but not unrelated sources"
        );
    }
}
