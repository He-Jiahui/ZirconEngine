use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetReference, ImportedAsset,
    ImportedAssetEntry, MeshAsset, MeshSdfCookBudget, MeshSdfCookSettings, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, VirtualGeometryCookConfig, cook_mesh_sdf_or_fallback,
    cook_virtual_geometry_from_mesh,
};
use zircon_runtime::core::math::{Vec2, Vec3};

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, OBJ_IMPORTER_DECLARATION,
    PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    OBJ_IMPORTER_DIST_CRATE_NAME, OBJ_IMPORTER_DIST_RUNTIME_ENTRY, ObjImporterRuntimePlugin,
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
};

pub fn import_obj(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let (models, _) = tobj::load_obj(
        &context.source_path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;

    let source_hint = context.uri.to_string();
    let mesh_sdf_settings = context.mesh_sdf_cook_request()?.settings();
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();
    let primitives = models
        .into_iter()
        .map(|model| {
            primitive_from_indexed_mesh(
                &model.mesh.positions,
                &model.mesh.normals,
                &model.mesh.texcoords,
                &model.mesh.indices,
                Some(model.name.as_str()),
                &source_hint,
                mesh_sdf_settings,
                &mut mesh_sdf_budget,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(model_outcome_with_mesh_subassets(
        context.uri.clone(),
        ModelAsset {
            uri: context.uri.clone(),
            primitives,
        },
    ))
}

fn model_outcome_with_mesh_subassets(
    root_uri: zircon_runtime::asset::AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let mesh_uris = (0..model.primitives.len())
        .map(|primitive_index| {
            zircon_runtime::asset::AssetUri::parse(&format!(
                "{root_uri}#Mesh{primitive_index}/Primitive0"
            ))
            .expect("generated obj mesh subasset uri must be valid")
        })
        .collect::<Vec<_>>();
    for (primitive, mesh_uri) in model.primitives.iter_mut().zip(mesh_uris.iter()) {
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
    }

    let mesh_entries = mesh_uris
        .into_iter()
        .zip(model.primitives.iter_mut())
        .map(|(mesh_uri, primitive)| {
            let mut mesh = MeshAsset::from_model_primitive(mesh_uri.clone(), primitive);
            mesh.mesh_sdf = primitive.mesh_sdf.take();
            ImportedAssetEntry::new(mesh_uri, ImportedAsset::Mesh(mesh))
        })
        .collect::<Vec<_>>();
    mesh_entries.into_iter().fold(
        AssetImportOutcome::new(root_uri, ImportedAsset::Model(model)),
        |outcome, entry| {
            outcome
                .with_dependency(entry.locator.clone())
                .with_entry(entry)
        },
    )
}

fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
    mesh_name: Option<&str>,
    source_hint: &str,
    mesh_sdf_settings: Option<MeshSdfCookSettings>,
    mesh_sdf_budget: &mut MeshSdfCookBudget,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let mut computed_normals = if normals.is_empty() {
        generate_normals(positions, indices)?
    } else {
        validate_triangle_indices(indices, vertex_count)?;
        normals.to_vec()
    };
    if computed_normals.len() < vertex_count * 3 {
        computed_normals.resize(vertex_count * 3, 0.0);
    }

    let vertices: Vec<MeshVertex> = (0..vertex_count)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                computed_normals[index * 3],
                computed_normals[index * 3 + 1],
                computed_normals[index * 3 + 2],
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
        .collect();

    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        indices,
        VirtualGeometryCookConfig {
            mesh_name: mesh_name.map(str::to_owned),
            source_hint: Some(source_hint.to_string()),
            ..VirtualGeometryCookConfig::default()
        },
    );
    let mesh_sdf = match mesh_sdf_settings {
        Some(settings) => cook_mesh_sdf_or_fallback(&vertices, indices, settings, mesh_sdf_budget)
            .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?,
        None => None,
    };

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        mesh: None,
        mesh_sdf,
        virtual_geometry,
    })
}

fn validate_triangle_indices(indices: &[u32], vertex_count: usize) -> Result<(), AssetImportError> {
    if indices.len() % 3 != 0 {
        return Err(AssetImportError::Parse(format!(
            "triangle index count {} was not a multiple of 3",
            indices.len()
        )));
    }
    for (element, &index) in indices.iter().enumerate() {
        let index = usize::try_from(index).map_err(|_| {
            AssetImportError::Parse(format!(
                "mesh index {index} at element {element} exceeds platform limits"
            ))
        })?;
        if index >= vertex_count {
            return Err(AssetImportError::Parse(format!(
                "mesh index {index} at element {element} exceeds vertex count {vertex_count}"
            )));
        }
    }
    Ok(())
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Result<Vec<f32>, AssetImportError> {
    let vertex_count = positions.len() / 3;
    validate_triangle_indices(indices, vertex_count)?;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let position_a = position(a);
        let position_b = position(b);
        let position_c = position(c);
        let face_normal = (position_b - position_a)
            .cross(position_c - position_a)
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    Ok(normals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_obj_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(
            manifest
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
        );
        assert!(
            manifest
                .asset_importers
                .iter()
                .any(|importer| importer.source_extensions.contains(&"obj".to_string()))
        );
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
        assert!(dist_module.target_modes.contains(
            &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        ));
        assert!(
            dist_module.target_modes.contains(
                &zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost
            )
        );
        assert!(
            dist_module
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
        );
        assert!(
            dist_module
                .capabilities
                .contains(&IMPORTER_CAPABILITY.to_string())
        );
    }

    #[test]
    fn registration_contributes_module_and_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(
            report
                .extensions
                .modules()
                .iter()
                .any(|module| module.name == MODULE_NAME)
        );
        assert!(
            report
                .extensions
                .asset_importers()
                .descriptors()
                .iter()
                .any(|importer| importer.id == "obj_importer.obj")
        );
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
        assert!(
            outcome
                .root_entry()
                .expect("root obj asset entry")
                .dependencies
                .contains(&mesh_uri)
        );
        let mesh_entry = outcome
            .entries
            .iter()
            .find(|entry| entry.locator == mesh_uri)
            .expect("obj mesh subasset");
        match &mesh_entry.asset {
            zircon_runtime::asset::ImportedAsset::Mesh(mesh) => {
                assert_eq!(mesh.vertex_count().unwrap(), 3);
                assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
                assert!(mesh.virtual_geometry.is_some());
            }
            other => panic!("unexpected mesh subasset: {other:?}"),
        }
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
        let root_uri =
            zircon_runtime::asset::AssetUri::parse("res://models/two_objects.obj").unwrap();
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
                        mesh.virtual_geometry.is_some(),
                        "{label} should retain cooked virtual geometry"
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
}
