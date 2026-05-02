use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, FunctionAssetImporter, ImportedAsset, MeshVertex,
    ModelAsset, ModelPrimitiveAsset, VirtualGeometryCookConfig,
};
use zircon_runtime::core::math::{Vec2, Vec3};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "obj_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_obj_importer_runtime";
pub const MODULE_NAME: &str = "ObjImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.obj_importer";
pub const IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.obj";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Wavefront OBJ model importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        AssetImporterDescriptor::new("obj_importer.obj", PLUGIN_ID, AssetKind::Model, 1)
            .with_priority(120)
            .with_source_extensions(["obj"])
            .with_required_capabilities([IMPORTER_CAPABILITY]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "OBJ Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("obj_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        registry.register_asset_importer(FunctionAssetImporter::new(importer, import_obj))?;
    }
    Ok(())
}

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
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AssetImportOutcome::new(ImportedAsset::Model(ModelAsset {
        uri: context.uri.clone(),
        primitives,
    })))
}

fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
    mesh_name: Option<&str>,
    source_hint: &str,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let mut computed_normals = if normals.is_empty() {
        generate_normals(positions, indices)
    } else {
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

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        virtual_geometry,
    })
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
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
        let face_normal = (position(b) - position(a))
            .cross(position(c) - position(a))
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].vertices.len(), 3);
                assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
        let _ = std::fs::remove_file(path);
    }

    fn temp_obj_path() -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zircon_plugin_obj_importer_{unique}.obj"))
    }
}
