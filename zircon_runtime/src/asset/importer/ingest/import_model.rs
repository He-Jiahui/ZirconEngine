use super::model_mesh_subassets::model_outcome_with_mesh_subassets;
use super::primitive_from_indexed_mesh::backfill_virtual_geometry_for_model;
use crate::asset::assets::ModelAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let mut model = ModelAsset::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    backfill_virtual_geometry_for_model(&mut model, &virtual_geometry_request);
    Ok(
        model_outcome_with_mesh_subassets(context.uri.clone(), model)
            .with_reference_repairs(context.reference_repairs()),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use zircon_runtime_interface::project::RelPath;

    use super::*;
    use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
    use crate::asset::{AssetKind, AssetUri, AssetUuid, ReferenceRepairKind};

    #[test]
    fn importer_outcome_exposes_complete_guid_repair() {
        let root = std::env::temp_dir().join(format!(
            "zircon_import_reference_repair_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("models")).unwrap();
        fs::write(root.join("models/hero.glb"), b"model").unwrap();
        let resolved: AssetUuid = "c1111111-2222-4333-8444-555555555555".parse().unwrap();
        let stale: AssetUuid = "c2111111-2222-4333-8444-555555555555".parse().unwrap();
        let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
            resolved,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "digest",
        )])
        .unwrap();
        let source = format!(
            "uri = \"res://models/hero.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"{stale}\"\npath_hint = \"assets/models/hero.glb\"\nsub = \"Mesh0\"\n"
        );
        let context = AssetImportContext::new(
            root.join("models/hero.model.toml"),
            AssetUri::parse("res://models/hero.model.toml").unwrap(),
            source.into_bytes(),
            toml::Table::new(),
        )
        .with_project_resolver(
            Arc::new(registry),
            Arc::new(vec![(RelPath::parse("assets").unwrap(), root.clone())]),
        );

        let outcome = import_model(&context).unwrap();
        assert_eq!(outcome.reference_repairs.len(), 1);
        let repair = &outcome.reference_repairs[0];
        assert_eq!(repair.kind, ReferenceRepairKind::Guid);
        assert_eq!(repair.stale.guid(), stale);
        assert_eq!(repair.stale.sub(), Some("Mesh0"));
        assert_eq!(repair.resolved.guid(), resolved);
        assert_eq!(
            repair.resolved.path_hint().as_str(),
            "assets/models/hero.glb"
        );
        assert_eq!(repair.resolved.sub(), None);
        fs::remove_dir_all(root).unwrap();
    }
}
