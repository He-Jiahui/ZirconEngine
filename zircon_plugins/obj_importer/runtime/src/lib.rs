use std::borrow::Cow;

use zircon_runtime::asset::{
    cook_mesh_sdf_or_fallback, cook_virtual_geometry_from_mesh, AssetImportContext,
    AssetImportError, AssetImportOutcome, AssetReference, ImportedAsset, ImportedAssetEntry,
    MeshAsset, MeshSdfCookBudget, MeshSdfCookSettings, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    VirtualGeometryCookRequest,
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
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    ObjImporterRuntimePlugin, OBJ_IMPORTER_DIST_CRATE_NAME, OBJ_IMPORTER_DIST_RUNTIME_ENTRY,
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
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
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
                &virtual_geometry_request,
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
    virtual_geometry_request: &VirtualGeometryCookRequest,
    mesh_sdf_settings: Option<MeshSdfCookSettings>,
    mesh_sdf_budget: &mut MeshSdfCookBudget,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let computed_normals = prepare_vertex_normals(positions, normals, indices)?;
    let vertices = vertices_from_attributes(positions, computed_normals.as_ref(), texcoords);

    let virtual_geometry = virtual_geometry_request
        .cook_config_for(mesh_name, source_hint)
        .and_then(|config| cook_virtual_geometry_from_mesh(&vertices, indices, config));
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

fn prepare_vertex_normals<'a>(
    positions: &[f32],
    normals: &'a [f32],
    indices: &[u32],
) -> Result<Cow<'a, [f32]>, AssetImportError> {
    let vertex_count = positions.len() / 3;
    if normals.is_empty() {
        return generate_normals(positions, indices).map(Cow::Owned);
    }
    validate_triangle_indices(indices, vertex_count)?;
    let required_normal_count = vertex_count * 3;
    if normals.len() >= required_normal_count {
        return Ok(Cow::Borrowed(normals));
    }

    let mut padded = normals.to_vec();
    padded.resize(required_normal_count, 0.0);
    Ok(Cow::Owned(padded))
}

fn vertices_from_attributes(
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
#[path = "tests/lib.rs"]
mod tests;
