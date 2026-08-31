use crate::asset::MaterialAsset;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use crate::core::resource::ResourceId;
use crate::core::CoreError;

use super::super::ProjectAssetManager;

const MAX_MATERIAL_PARENT_DEPTH: usize = 4;

impl ProjectAssetManager {
    pub(crate) fn load_effective_material_asset(
        &self,
        root_id: ResourceId,
    ) -> Result<(MaterialAsset, Vec<RenderMaterialValidationError>), CoreError> {
        let root = self.load_material_asset(root_id)?;
        Ok(self.resolve_effective_material_asset(root_id, root))
    }

    pub(crate) fn resolve_effective_material_asset(
        &self,
        root_id: ResourceId,
        root: MaterialAsset,
    ) -> (MaterialAsset, Vec<RenderMaterialValidationError>) {
        let root_shader = root.shader.clone();
        let mut diagnostics = Vec::new();
        let mut lineage = Vec::with_capacity(MAX_MATERIAL_PARENT_DEPTH + 1);
        lineage.push((root_id, root));

        loop {
            let Some(parent_reference) = lineage
                .last()
                .and_then(|(_, material)| material.parent.clone())
            else {
                break;
            };
            if lineage.len() > MAX_MATERIAL_PARENT_DEPTH {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain exceeds depth limit {MAX_MATERIAL_PARENT_DEPTH}"
                )));
                break;
            }
            let Some(parent_id) = self.resolve_asset_id(&parent_reference.locator) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` is not registered",
                    parent_reference.locator
                )));
                break;
            };
            if lineage.iter().any(|(id, _)| *id == parent_id) {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain contains cycle at {parent_id}"
                )));
                break;
            }
            let Ok(parent) = self.load_material_asset(parent_id) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` failed to load",
                    parent_reference.locator
                )));
                break;
            };
            if parent.shader != root_shader {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` uses shader `{}` but child uses `{}`",
                    parent_reference.locator, parent.shader.locator, root_shader.locator
                )));
                break;
            }
            lineage.push((parent_id, parent));
        }

        let mut effective = lineage
            .pop()
            .map(|(_, material)| material)
            .expect("material lineage contains root");
        while let Some((_, mut child)) = lineage.pop() {
            child.inherit_parent_values_from(&effective);
            effective = child;
        }
        effective.parent = None;
        (effective, diagnostics)
    }
}

fn invalid_parent_diagnostic(diagnostic: String) -> RenderMaterialValidationError {
    RenderMaterialValidationError::InvalidMaterialParent {
        source: RenderMaterialDiagnosticSource::MaterialOverride,
        path: "parent".to_string(),
        diagnostic,
    }
}

#[cfg(test)]
mod tests {
    use crate::asset::{AssetReference, AssetUri, MaterialAsset, ProjectAssetManager};
    use crate::core::framework::render::RenderMaterialValidationError;
    use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};

    fn material() -> MaterialAsset {
        MaterialAsset::from_toml_str(
            r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "builtin://shader/pbr.wgsl"
"#,
        )
        .expect("material document")
    }

    fn insert_material(
        manager: &ProjectAssetManager,
        uri: &str,
        material: MaterialAsset,
    ) -> (ResourceId, AssetUri) {
        let uri = AssetUri::parse(uri).expect("material uri");
        let id = ResourceId::from_locator(&uri);
        manager
            .assets::<MaterialAsset>()
            .insert(
                ResourceRecord::new(id, ResourceKind::Material, uri.clone()),
                material,
            )
            .expect("material insert");
        (id, uri)
    }

    #[test]
    fn effective_material_inherits_parent_values_once_for_all_consumers() {
        let manager = ProjectAssetManager::default();
        let mut parent = material();
        parent
            .property_values
            .insert("roughness".to_string(), toml::Value::Float(0.23));
        let (_, parent_uri) = insert_material(
            &manager,
            "res://materials/effective-parent.zmaterial",
            parent,
        );
        let mut child = material();
        child.parent = Some(AssetReference::from_locator(parent_uri));
        let (child_id, _) =
            insert_material(&manager, "res://materials/effective-child.zmaterial", child);

        let (effective, diagnostics) = manager
            .load_effective_material_asset(child_id)
            .expect("effective child material");

        assert_eq!(effective.roughness, 0.23);
        assert!(effective.parent.is_none());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn missing_parent_keeps_the_renderable_child_and_reports_one_diagnostic() {
        let manager = ProjectAssetManager::default();
        let mut child = material();
        child.roughness = 0.41;
        child.parent = Some(AssetReference::from_locator(
            AssetUri::parse("res://materials/missing-parent.zmaterial")
                .expect("missing parent uri"),
        ));
        let (child_id, _) = insert_material(
            &manager,
            "res://materials/missing-parent-child.zmaterial",
            child,
        );

        let (effective, diagnostics) = manager
            .load_effective_material_asset(child_id)
            .expect("renderable child material");

        assert_eq!(effective.roughness, 0.41);
        assert!(effective.parent.is_none());
        assert!(matches!(
            diagnostics.as_slice(),
            [RenderMaterialValidationError::InvalidMaterialParent { .. }]
        ));
    }
}
