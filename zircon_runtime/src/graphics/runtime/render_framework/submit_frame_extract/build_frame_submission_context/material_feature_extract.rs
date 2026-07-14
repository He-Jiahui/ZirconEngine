use std::collections::BTreeSet;

use crate::asset::{MaterialAsset, ProjectAssetManager};
use crate::core::framework::render::{
    AdvancedPbrMaterialFrameUsage, RenderFrameExtract, RenderLayerSet,
};
use crate::core::resource::ResourceId;

const MAX_MATERIAL_PARENT_DEPTH: usize = 4;

pub(super) fn resolve_advanced_pbr_material_usage(
    asset_manager: &ProjectAssetManager,
    extract: &mut RenderFrameExtract,
) {
    let mut usage = AdvancedPbrMaterialFrameUsage::default();
    for mesh in &extract.geometry.meshes {
        if !material_is_visible_to_selected_camera(
            extract.view.selected_camera_layers(),
            &mesh.render_layer_mask,
        ) {
            continue;
        }
        let Some(material) = effective_material(asset_manager, mesh.material.id()) else {
            continue;
        };
        usage.record(&material.advanced_pbr_features());
    }
    extract.lighting.advanced_lighting.material_features = usage;
}

fn material_is_visible_to_selected_camera(
    selected_camera_layers: &RenderLayerSet,
    material_layers: &RenderLayerSet,
) -> bool {
    selected_camera_layers.intersects(material_layers)
}

fn effective_material(
    asset_manager: &ProjectAssetManager,
    root_id: ResourceId,
) -> Option<MaterialAsset> {
    let root = asset_manager.load_material_asset(root_id).ok()?;
    let root_shader = root.shader.clone();
    let mut visited = BTreeSet::from([root_id]);
    let mut lineage = vec![root];

    while lineage.len() <= MAX_MATERIAL_PARENT_DEPTH {
        let Some(parent_reference) = lineage.last().and_then(|material| material.parent.clone())
        else {
            break;
        };
        let Some(parent_id) = asset_manager.resolve_asset_id(&parent_reference.locator) else {
            break;
        };
        if !visited.insert(parent_id) {
            break;
        }
        let Ok(parent) = asset_manager.load_material_asset(parent_id) else {
            break;
        };
        if parent.shader != root_shader {
            break;
        }
        lineage.push(parent);
    }

    let mut effective = lineage.pop()?;
    while let Some(mut child) = lineage.pop() {
        child.inherit_parent_values_from(&effective);
        effective = child;
    }
    effective.parent = None;
    Some(effective)
}

#[cfg(test)]
mod tests {
    use crate::asset::{AssetReference, AssetUri, MaterialAsset, ProjectAssetManager};
    use crate::core::framework::render::RenderLayerSet;
    use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};

    use super::{effective_material, material_is_visible_to_selected_camera};

    #[test]
    fn render_advanced_lighting_material_usage_ignores_meshes_outside_selected_camera_layers() {
        let selected = RenderLayerSet::from_layers([1, 40]);

        assert!(material_is_visible_to_selected_camera(
            &selected,
            &RenderLayerSet::layer(40)
        ));
        assert!(!material_is_visible_to_selected_camera(
            &selected,
            &RenderLayerSet::layer(2)
        ));
    }

    #[test]
    fn render_advanced_lighting_material_usage_keeps_child_features_when_parent_is_missing() {
        let manager = ProjectAssetManager::default();
        let material_uri = AssetUri::parse("res://materials/advanced-child.zmaterial")
            .expect("child material uri");
        let material_id = ResourceId::from_locator(&material_uri);
        let mut material = MaterialAsset::from_toml_str(
            r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
specular_transmission = 0.75
"#,
        )
        .expect("advanced child material");
        material.parent = Some(AssetReference::from_locator(
            AssetUri::parse("res://materials/missing-parent.zmaterial")
                .expect("missing parent uri"),
        ));
        manager
            .assets::<MaterialAsset>()
            .insert(
                ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
                material,
            )
            .expect("child material insert");

        let effective = effective_material(&manager, material_id)
            .expect("missing parent must not discard the renderable child material");

        assert_eq!(
            effective.advanced_pbr_features().specular_transmission,
            0.75
        );
        assert!(effective.parent.is_none());
    }
}
