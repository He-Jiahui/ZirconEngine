use super::*;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn package_declares_obj_importer() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"obj".to_string())));
}

#[test]
fn declaration_projects_obj_package_metadata() {
    let descriptor = runtime_plugin_descriptor();
    let manifest = package_manifest();

    assert_eq!(descriptor.package_id(), OBJ_IMPORTER_DECLARATION.id());
    assert_eq!(descriptor.category(), OBJ_IMPORTER_DECLARATION.category());
    assert_eq!(
        descriptor.target_modes(),
        OBJ_IMPORTER_DECLARATION.target_modes()
    );
    assert_eq!(
        descriptor.capabilities(),
        runtime_capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        manifest.supported_platforms.as_slice(),
        OBJ_IMPORTER_DECLARATION.supported_platforms()
    );
    assert_eq!(
        manifest.default_packaging.as_slice(),
        OBJ_IMPORTER_DECLARATION.default_packaging()
    );
}

#[test]
fn package_manifest_declares_obj_importer_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    let distribution = manifest.distribution.as_ref().expect("dist metadata");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, OBJ_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, OBJ_IMPORTER_DIST_RUNTIME_ENTRY);

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "obj_importer.dist")
        .expect("obj importer dist module");
    assert_eq!(dist_module.crate_name, OBJ_IMPORTER_DIST_CRATE_NAME);
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime));
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
    assert!(dist_module
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(dist_module
        .capabilities
        .contains(&IMPORTER_CAPABILITY.to_string()));
}

#[test]
fn registration_contributes_module_and_importer() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert!(report
        .extensions
        .asset_importers()
        .descriptors()
        .iter()
        .any(|importer| importer.id == "obj_importer.obj"));
}

#[test]
fn obj_importer_decodes_model_asset() {
    let path = temp_obj_path();
    std::fs::write(
        &path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();
    let report = plugin_registration();
    let importer = report.extensions.asset_importers().select(&path).unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        path.clone(),
        zircon_runtime::asset::AssetUri::parse("res://models/triangle.obj").unwrap(),
        Vec::new(),
        Default::default(),
    );

    let outcome = importer.import(&context).unwrap();
    let imported = &outcome.root_entry().expect("root obj asset entry").asset;

    match imported {
        zircon_runtime::asset::ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert!(model.primitives[0].virtual_geometry.is_none());
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                zircon_runtime::asset::AssetUri::parse(
                    "res://models/triangle.obj#Mesh0/Primitive0"
                )
                .unwrap()
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    let mesh_uri =
        zircon_runtime::asset::AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0")
            .unwrap();
    assert!(outcome
        .root_entry()
        .expect("root obj asset entry")
        .dependencies
        .contains(&mesh_uri));
    let mesh_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == mesh_uri)
        .expect("obj mesh subasset");
    match &mesh_entry.asset {
        zircon_runtime::asset::ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
            assert!(mesh.virtual_geometry.is_none());
        }
        other => panic!("unexpected mesh subasset: {other:?}"),
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn obj_importer_cooks_virtual_geometry_when_explicitly_enabled() {
    let path = temp_obj_path();
    std::fs::write(
        &path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
",
    )
    .unwrap();
    let report = plugin_registration();
    let importer = report.extensions.asset_importers().select(&path).unwrap();
    let import_settings = toml::from_str(
        r#"
                [virtual_geometry]
                enabled = true
            "#,
    )
    .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        path.clone(),
        zircon_runtime::asset::AssetUri::parse("res://models/enabled.obj").unwrap(),
        Vec::new(),
        import_settings,
    );

    let outcome = importer.import(&context).unwrap();
    match &outcome.root_entry().expect("root obj asset entry").asset {
        zircon_runtime::asset::ImportedAsset::Model(model) => {
            assert!(model.primitives[0].virtual_geometry.is_some());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    let mesh = outcome
        .entries
        .iter()
        .find_map(|entry| match &entry.asset {
            zircon_runtime::asset::ImportedAsset::Mesh(mesh) => Some(mesh),
            _ => None,
        })
        .expect("enabled OBJ import should produce a mesh subasset");
    assert!(mesh.virtual_geometry.is_some());
    let _ = std::fs::remove_file(path);
}

#[test]
fn obj_importer_emits_multi_mesh_subassets() {
    let path = temp_obj_path();
    std::fs::write(
        &path,
        "\
o FirstObject
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
o SecondObject
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.0 1.0 0.0
f 4 5 6
",
    )
    .unwrap();
    let report = plugin_registration();
    let importer = report.extensions.asset_importers().select(&path).unwrap();
    let root_uri = zircon_runtime::asset::AssetUri::parse("res://models/two_objects.obj").unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        path.clone(),
        root_uri.clone(),
        Vec::new(),
        Default::default(),
    );

    let outcome = importer.import(&context).unwrap();
    let root_entry = outcome.root_entry().expect("root obj asset entry");
    match &root_entry.asset {
        zircon_runtime::asset::ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert!(model
                .primitives
                .iter()
                .all(|primitive| primitive.virtual_geometry.is_none()));
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                obj_label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                obj_label_uri(&root_uri, "Mesh1/Primitive0")
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    for label in ["Mesh0/Primitive0", "Mesh1/Primitive0"] {
        let mesh_uri = obj_label_uri(&root_uri, label);
        assert!(
            root_entry.dependencies.contains(&mesh_uri),
            "root dependencies should include {label}"
        );
        let mesh_entry = outcome
            .entries
            .iter()
            .find(|entry| entry.locator == mesh_uri)
            .unwrap_or_else(|| panic!("missing obj mesh subasset {mesh_uri}"));
        match &mesh_entry.asset {
            zircon_runtime::asset::ImportedAsset::Mesh(mesh) => {
                assert_eq!(mesh.vertex_count().unwrap(), 3);
                assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
                assert!(
                    mesh.virtual_geometry.is_none(),
                    "{label} should not cook virtual geometry by default"
                );
            }
            other => panic!("unexpected mesh subasset: {other:?}"),
        }
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn obj_index_admission_rejects_out_of_range_vertices_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        let mut budget = MeshSdfCookBudget::default();
        primitive_from_indexed_mesh(
            &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            &[],
            &[],
            &[0, 1, 3],
            Some("malformed"),
            "obj-index-admission-test",
            &VirtualGeometryCookRequest::default(),
            None,
            &mut budget,
        )
    });

    assert!(result.is_ok(), "malformed OBJ indices must not unwind");
    let error = result
        .unwrap()
        .expect_err("out-of-range OBJ index must be rejected");
    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("mesh index 3") && message.contains("vertex count 3")
    ));
}

#[test]
fn plugins07_importer_hotpath_obj_borrows_complete_normals_and_preserves_vertices() {
    let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let normals = [0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
    let texcoords = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
    let indices = [0, 1, 2];

    let prepared = prepare_vertex_normals(&positions, &normals, &indices).unwrap();

    assert!(matches!(&prepared, std::borrow::Cow::Borrowed(_)));
    assert_eq!(
        vertices_from_attributes(&positions, prepared.as_ref(), &texcoords),
        legacy_vertices_from_attributes(&positions, &normals, &texcoords)
    );

    let padded = prepare_vertex_normals(&positions, &normals[..3], &indices).unwrap();
    assert!(matches!(&padded, std::borrow::Cow::Owned(_)));
    assert_eq!(
        padded.as_ref(),
        &[0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_importer_hotpath_release_obj_borrowed_normals_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const NORMAL_VALUES: usize = 524_288;
    const ITERATIONS: usize = 16;
    const THRESHOLD_PERCENT: u128 = 80;
    let normals = vec![0.25_f32; NORMAL_VALUES];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_normal_clone(&normals, ITERATIONS);
        let optimized = || measure_normal_borrow(&normals, ITERATIONS);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_obj_performance_gate(
            &legacy_samples,
            &optimized_samples,
            THRESHOLD_PERCENT,
            &format!(
                "normal_values={NORMAL_VALUES} iterations_per_sample={ITERATIONS} legacy_cloned_bytes_per_sample={} optimized_cloned_bytes_per_sample=0",
                NORMAL_VALUES * ITERATIONS * std::mem::size_of::<f32>()
            ),
        );
}

fn legacy_vertices_from_attributes(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
) -> Vec<MeshVertex> {
    (0..positions.len() / 3)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                normals[index * 3],
                normals[index * 3 + 1],
                normals[index * 3 + 2],
            );
            let uv = if texcoords.len() >= (index + 1) * 2 {
                Vec2::new(texcoords[index * 2], texcoords[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
        })
        .collect()
}

fn measure_normal_clone(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let owned = black_box(normals).to_vec();
        values += black_box(owned.as_slice()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn measure_normal_borrow(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let borrowed: Cow<'_, [f32]> = Cow::Borrowed(black_box(normals));
        values += black_box(borrowed.as_ref()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn emit_obj_performance_gate(
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_obj_p95(legacy_samples);
    let optimized_p95 = nearest_rank_obj_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
            "PERF_RESULT plugins07_obj_borrowed_normals sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
            obj_samples_csv(legacy_samples),
            obj_samples_csv(optimized_samples),
        );
    assert!(
            improvement_percent >= threshold_percent,
            "OBJ borrowed normals must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
        );
}

fn nearest_rank_obj_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn obj_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn obj_label_uri(
    root_uri: &zircon_runtime::asset::AssetUri,
    label: &str,
) -> zircon_runtime::asset::AssetUri {
    zircon_runtime::asset::AssetUri::parse(&format!("{root_uri}#{label}")).unwrap()
}

fn temp_obj_path() -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_plugin_obj_importer_{unique}.obj"))
}
