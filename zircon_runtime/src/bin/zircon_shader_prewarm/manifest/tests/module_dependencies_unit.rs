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
