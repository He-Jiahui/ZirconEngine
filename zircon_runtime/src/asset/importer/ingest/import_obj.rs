use super::auxiliary_source::AuxiliarySourceResolver;
use super::model_mesh_subassets::model_outcome_with_mesh_subassets;
use super::primitive_from_indexed_mesh::{primitive_from_indexed_mesh, MissingNormalPolicy};
use std::cell::RefCell;
use std::io::{BufReader, Cursor};

use crate::asset::assets::ModelAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, MeshSdfCookBudget};

pub(crate) fn import_obj(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let mut source = Cursor::new(context.source_bytes.as_slice());
    let material_root = context.source_path.parent().ok_or_else(|| {
        AssetImportError::Parse("OBJ source path has no parent for material resolution".to_string())
    })?;
    let material_resolver = RefCell::new(None);
    let auxiliary_error = RefCell::new(None);
    let loaded = tobj::load_obj_buf(
        &mut source,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |material_path| {
            let result: Result<tobj::MTLLoadResult, AssetImportError> = (|| {
                if material_resolver.borrow().is_none() {
                    material_resolver
                        .replace(Some(AuxiliarySourceResolver::new(context, material_root)?));
                }
                let resolver = material_resolver.borrow();
                let resolver = resolver
                    .as_ref()
                    .expect("material resolver was initialized above");
                let lexical_path = resolver.resolve_lexical_path(material_path)?;
                if let Some(bytes) = context.source_file_snapshot(&lexical_path) {
                    return Ok(tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(bytes))));
                }
                let path = resolver.resolve_path(material_path)?;
                if let Some(bytes) = context.source_file_snapshot(&path) {
                    Ok(tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(bytes))))
                } else {
                    Ok(tobj::load_mtl(path))
                }
            })();
            match result {
                Ok(materials) => materials,
                Err(error) => {
                    auxiliary_error.replace(Some(error));
                    Err(tobj::LoadError::OpenFileFailed)
                }
            }
        },
    );
    if let Some(error) = auxiliary_error.into_inner() {
        return Err(error);
    }
    let (models, materials) =
        loaded.map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;
    materials.map_err(|error| AssetImportError::Parse(format!("parse obj material: {error}")))?;

    let source_hint = context.uri.to_string();
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    let mesh_sdf_request = context.mesh_sdf_cook_request()?;
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();
    let primitives = models
        .into_iter()
        .map(|model| {
            primitive_from_indexed_mesh(
                &model.mesh.positions,
                &model.mesh.normals,
                MissingNormalPolicy::Smooth,
                &model.mesh.texcoords,
                &[],
                &[],
                &[],
                &model.mesh.indices,
                &[],
                &[],
                Some(model.name.as_str()),
                &source_hint,
                &virtual_geometry_request,
                &mesh_sdf_request,
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

#[cfg(test)]
#[path = "tests/import_obj_plugins07_obj_snapshot_tests.rs"]
mod plugins07_obj_snapshot_tests;
