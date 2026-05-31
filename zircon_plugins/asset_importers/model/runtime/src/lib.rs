use std::io::Cursor;

mod cad;

use cad::import_dxf_model;
use ply_rs_bw as ply;
use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, AssetReference, DiagnosticOnlyAssetImporter,
    FunctionAssetImporter, ImportedAsset, ImportedAssetEntry, MeshAsset, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, VirtualGeometryCookConfig,
};
use zircon_runtime::core::math::{Vec2, Vec3};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "asset_importer.model";
pub const IMPORTER_FAMILY: &str = "model";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_model_runtime";
pub const MODULE_NAME: &str = "ModelImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.model";
pub const MESH_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.mesh";
pub const CAD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.cad";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        MESH_IMPORTER_CAPABILITY,
        CAD_IMPORTER_CAPABILITY,
    ]
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
    ModuleDescriptor::new(MODULE_NAME, "Model asset importer family plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.model.gltf", 100, ["gltf", "glb"])
            .with_required_capabilities(["runtime.asset.importer.model.gltf"]),
        descriptor("asset_importer.model.obj", 100, ["obj"])
            .with_required_capabilities(["runtime.asset.importer.model.obj"]),
        descriptor("asset_importer.model.mesh", 110, ["ply", "stl"])
            .with_required_capabilities([MESH_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.model.cad", 110, ["dxf"])
            .with_required_capabilities([CAD_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.model.optional_native_backend",
            80,
            ["fbx", "dae", "3ds", "usd", "usda", "usdc", "usdz"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Model Asset Importers")
        .with_category("asset_importer")
        .with_runtime_crate(RUNTIME_CRATE_NAME)
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_runtime_module(runtime_module_manifest())
        .with_capabilities(runtime_capabilities().iter().copied());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.model.runtime", RUNTIME_CRATE_NAME)
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
        match importer.id.as_str() {
            "asset_importer.model.mesh" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_mesh_model))?,
            "asset_importer.model.cad" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_dxf_model))?,
            "asset_importer.model.gltf" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "gltf/glb import is provided by the split gltf_importer package",
                ))?;
            }
            "asset_importer.model.obj" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "obj import is provided by the split obj_importer package",
                ))?;
            }
            "asset_importer.model.optional_native_backend" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "fbx/dae/3ds/usd import requires a NativeDynamic model backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known model importer ids"),
        }
    }
    Ok(())
}

pub fn import_mesh_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "stl" => import_stl(context),
        "ply" => import_ply(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "model mesh importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_stl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let mut reader = Cursor::new(context.source_bytes.as_slice());
    let mesh = stl_io::read_stl(&mut reader).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse stl {}: {error}",
            context.source_path.display()
        ))
    })?;
    if mesh.faces.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "parse stl {}: file contains no triangles",
            context.source_path.display()
        )));
    }

    let positions = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.0)
        .collect::<Vec<_>>();
    let indices = mesh
        .faces
        .iter()
        .flat_map(|face| face.vertices)
        .map(|index| {
            u32::try_from(index).map_err(|_| {
                AssetImportError::Parse(format!(
                    "parse stl {}: vertex index {index} exceeds u32",
                    context.source_path.display()
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let source_hint = context.uri.to_string();
    let primitive = primitive_from_indexed_mesh(
        &positions,
        &[],
        &[],
        &indices,
        context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str()),
        &source_hint,
    )?;

    model_outcome(context, vec![primitive])
}

fn import_ply(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let parser = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    let ply = parser
        .read_ply(&mut Cursor::new(context.source_bytes.as_slice()))
        .map_err(|error| {
            AssetImportError::Parse(format!(
                "parse ply {}: {error}",
                context.source_path.display()
            ))
        })?;
    let vertices = ply.payload.get("vertex").ok_or_else(|| {
        AssetImportError::Parse(format!(
            "parse ply {}: missing vertex element",
            context.source_path.display()
        ))
    })?;
    let positions = vertices
        .iter()
        .flat_map(|vertex| {
            ["x", "y", "z"]
                .into_iter()
                .map(|key| scalar_f32(vertex, key, context))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let normals = collect_optional_vec3(vertices, ["nx", "ny", "nz"], context)?;
    let texcoords = collect_optional_vec2_candidates(
        vertices,
        [["s", "t"], ["u", "v"], ["texture_u", "texture_v"]],
        context,
    )?;
    let faces = ply.payload.get("face").ok_or_else(|| {
        AssetImportError::Parse(format!(
            "parse ply {}: missing face element",
            context.source_path.display()
        ))
    })?;
    let mut indices = Vec::new();
    for face in faces {
        let face_indices = list_u32(face, "vertex_indices")
            .or_else(|| list_u32(face, "vertex_index"))
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "parse ply {}: face missing vertex_indices list",
                    context.source_path.display()
                ))
            })?;
        if face_indices.len() < 3 {
            return Err(AssetImportError::Parse(format!(
                "parse ply {}: face has fewer than three vertices",
                context.source_path.display()
            )));
        }
        for triangle in 1..face_indices.len() - 1 {
            indices.push(face_indices[0]);
            indices.push(face_indices[triangle]);
            indices.push(face_indices[triangle + 1]);
        }
    }
    if indices.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "parse ply {}: file contains no triangles",
            context.source_path.display()
        )));
    }

    let source_hint = context.uri.to_string();
    let primitive = primitive_from_indexed_mesh(
        &positions,
        &normals,
        &texcoords,
        &indices,
        context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str()),
        &source_hint,
    )?;

    model_outcome(context, vec![primitive])
}

pub(crate) fn model_outcome(
    context: &AssetImportContext,
    primitives: Vec<ModelPrimitiveAsset>,
) -> Result<AssetImportOutcome, AssetImportError> {
    let model = ModelAsset {
        uri: context.uri.clone(),
        primitives,
    };
    Ok(model_outcome_with_mesh_subassets(
        context.uri.clone(),
        model,
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
            .expect("generated model mesh subasset uri must be valid")
        })
        .collect::<Vec<_>>();
    for (primitive, mesh_uri) in model.primitives.iter_mut().zip(mesh_uris.iter()) {
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
    }

    mesh_uris.into_iter().zip(model.primitives.iter()).fold(
        AssetImportOutcome::new(root_uri.clone(), ImportedAsset::Model(model.clone())),
        |outcome, (mesh_uri, primitive)| {
            outcome
                .with_dependency(mesh_uri.clone())
                .with_entry(ImportedAssetEntry::new(
                    mesh_uri.clone(),
                    ImportedAsset::Mesh(MeshAsset::from_model_primitive(mesh_uri, primitive)),
                ))
        },
    )
}

pub(crate) fn primitive_from_indexed_mesh(
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
    validate_indices(indices, vertex_count)?;
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
        mesh: None,
        virtual_geometry,
    })
}

fn validate_indices(indices: &[u32], vertex_count: usize) -> Result<(), AssetImportError> {
    for index in indices {
        if *index as usize >= vertex_count {
            return Err(AssetImportError::Parse(format!(
                "model index {index} exceeds vertex count {vertex_count}"
            )));
        }
    }
    Ok(())
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

fn scalar_f32(
    element: &ply::ply::DefaultElement,
    key: &str,
    context: &AssetImportContext,
) -> Result<f32, AssetImportError> {
    element
        .get(key)
        .and_then(|property| property.to_f32_lossy())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "parse ply {}: vertex missing numeric `{key}`",
                context.source_path.display()
            ))
        })
}

fn list_u32(element: &ply::ply::DefaultElement, key: &str) -> Option<Vec<u32>> {
    element.get(key).and_then(|property| property.to_u32_list())
}

fn collect_optional_vec3(
    vertices: &[ply::ply::DefaultElement],
    keys: [&str; 3],
    context: &AssetImportContext,
) -> Result<Vec<f32>, AssetImportError> {
    if vertices
        .iter()
        .all(|vertex| keys.iter().all(|key| vertex.contains_key(*key)))
    {
        vertices
            .iter()
            .flat_map(|vertex| keys.into_iter().map(|key| scalar_f32(vertex, key, context)))
            .collect()
    } else {
        Ok(Vec::new())
    }
}

fn collect_optional_vec2_candidates(
    vertices: &[ply::ply::DefaultElement],
    candidates: [[&str; 2]; 3],
    context: &AssetImportContext,
) -> Result<Vec<f32>, AssetImportError> {
    let Some(keys) = candidates.into_iter().find(|keys| {
        vertices
            .iter()
            .all(|vertex| vertex.contains_key(keys[0]) && vertex.contains_key(keys[1]))
    }) else {
        return Ok(Vec::new());
    };
    vertices
        .iter()
        .flat_map(|vertex| keys.into_iter().map(|key| scalar_f32(vertex, key, context)))
        .collect()
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Model, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
        .with_additional_output_kinds([AssetKind::Mesh])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_model_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"stl".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&MESH_IMPORTER_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&CAD_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_stl_ply_and_dxf_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 5);
        assert_eq!(
            report
                .extensions
                .asset_importers()
                .select(std::path::Path::new("mesh.stl"))
                .unwrap()
                .descriptor()
                .id
                .as_str(),
            "asset_importer.model.mesh"
        );
        assert_eq!(
            report
                .extensions
                .asset_importers()
                .select(std::path::Path::new("mesh.dxf"))
                .unwrap()
                .descriptor()
                .id
                .as_str(),
            "asset_importer.model.cad"
        );
    }

    #[test]
    fn stl_importer_decodes_ascii_triangle() {
        let outcome = import_fixture_outcome("triangle.stl", ascii_stl_fixture());
        let imported = root_imported(&outcome);

        assert_single_mesh_subasset(&outcome, "triangle.stl");

        match imported {
            ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].vertices.len(), 3);
                assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
                assert!(model.primitives[0].virtual_geometry.is_some());
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn ply_importer_decodes_ascii_triangle() {
        let outcome = import_fixture_outcome("triangle.ply", ascii_ply_fixture());
        let imported = root_imported(&outcome);

        assert_single_mesh_subasset(&outcome, "triangle.ply");

        match imported {
            ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].vertices.len(), 3);
                assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
                assert_eq!(model.primitives[0].vertices[1].uv[0], 1.0);
                assert!(model.primitives[0].virtual_geometry.is_some());
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn dxf_importer_decodes_3dface_triangle() {
        let outcome = import_fixture_outcome("triangle.dxf", ascii_dxf_3dface_fixture());
        let imported = root_imported(&outcome);

        assert_single_mesh_subasset(&outcome, "triangle.dxf");

        match imported {
            ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].vertices.len(), 3);
                assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
                assert!(model.primitives[0].virtual_geometry.is_some());
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    fn assert_single_mesh_subasset(outcome: &AssetImportOutcome, path: &str) {
        let mesh_uri = zircon_runtime::asset::AssetUri::parse(&format!(
            "res://models/{path}#Mesh0/Primitive0"
        ))
        .expect("test mesh subasset uri");
        let root = outcome.root_entry().expect("root model asset entry");
        assert!(
            root.dependencies.contains(&mesh_uri),
            "root dependencies should include {mesh_uri}"
        );
        match &root.asset {
            ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].mesh.as_ref().unwrap().locator, mesh_uri);
            }
            other => panic!("unexpected root model asset: {other:?}"),
        }
        let mesh_entry = outcome
            .entries
            .iter()
            .find(|entry| entry.locator == mesh_uri)
            .unwrap_or_else(|| panic!("missing mesh subasset {mesh_uri}"));
        match &mesh_entry.asset {
            ImportedAsset::Mesh(mesh) => {
                assert_eq!(mesh.vertex_count().unwrap(), 3);
                assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
                assert!(
                    mesh.virtual_geometry.is_some(),
                    "{mesh_uri} should preserve cooked virtual geometry"
                );
            }
            other => panic!("unexpected mesh subasset {mesh_uri}: {other:?}"),
        }
    }

    fn root_imported(outcome: &AssetImportOutcome) -> ImportedAsset {
        outcome
            .root_entry()
            .expect("root model asset entry")
            .asset
            .clone()
    }

    fn import_fixture_outcome(path: &str, source: &str) -> AssetImportOutcome {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        let uri = format!("res://models/{path}");
        let context = AssetImportContext::new(
            path.into(),
            zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
            source.as_bytes().to_vec(),
            Default::default(),
        );
        importer.import(&context).unwrap()
    }

    fn ascii_stl_fixture() -> &'static str {
        r#"solid triangle
facet normal 0 0 1
  outer loop
    vertex 0 0 0
    vertex 1 0 0
    vertex 0 1 0
  endloop
endfacet
endsolid triangle
"#
    }

    fn ascii_ply_fixture() -> &'static str {
        r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
property float nx
property float ny
property float nz
property float u
property float v
element face 1
property list uchar int vertex_indices
end_header
0 0 0 0 0 1 0 0
1 0 0 0 0 1 1 0
0 1 0 0 0 1 0 1
3 0 1 2
"#
    }

    fn ascii_dxf_3dface_fixture() -> &'static str {
        r#"0
SECTION
2
ENTITIES
0
3DFACE
8
0
10
0.0
20
0.0
30
0.0
11
1.0
21
0.0
31
0.0
12
0.0
22
1.0
32
0.0
13
0.0
23
1.0
33
0.0
0
ENDSEC
0
EOF
"#
    }
}
