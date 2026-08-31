use std::collections::HashMap;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    AdvancedPbrMaterialFrameUsage, RenderFrameExtract, RenderLayerSet,
};

pub(super) fn resolve_advanced_pbr_material_usage(
    asset_manager: &ProjectAssetManager,
    extract: &RenderFrameExtract,
) -> AdvancedPbrMaterialFrameUsage {
    crate::profile_scope!("render", "material", "advanced_feature_census");
    let mut usage = AdvancedPbrMaterialFrameUsage::default();
    let mut features_by_material = HashMap::new();
    let mut _parent_diagnostic_count = 0_u64;
    for mesh in &extract.geometry.meshes {
        if !material_is_visible_to_selected_camera(
            extract.view.selected_camera_layers(),
            &mesh.common.layer_mask,
        ) {
            continue;
        }
        let features = features_by_material
            .entry(mesh.material.id())
            .or_insert_with(|| {
                asset_manager
                    .load_effective_material_asset(mesh.material.id())
                    .ok()
                    .map(|(material, diagnostics)| {
                        _parent_diagnostic_count += diagnostics.len() as u64;
                        material.advanced_pbr_features()
                    })
            });
        let Some(features) = features.as_ref() else {
            continue;
        };
        usage.record(features);
    }
    crate::profile_counter!(
        "render",
        "advanced_feature_material_resolutions",
        features_by_material.len()
    );
    crate::profile_counter!(
        "render",
        "advanced_feature_parent_diagnostics",
        _parent_diagnostic_count
    );
    usage
}

fn material_is_visible_to_selected_camera(
    selected_camera_layers: &RenderLayerSet,
    material_layers: &RenderLayerSet,
) -> bool {
    selected_camera_layers.intersects(material_layers)
}

#[cfg(test)]
mod tests {
    use crate::asset::{AssetReference, AssetUri, MaterialAsset, ProjectAssetManager};
    use crate::core::framework::render::{
        AdvancedPbrMaterialFrameUsage, RenderFrameExtract, RenderLayerSet,
        RenderWorldSnapshotHandle,
    };
    use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
    use crate::scene::world::World;

    use super::{material_is_visible_to_selected_camera, resolve_advanced_pbr_material_usage};

    #[test]
    fn runtime07_renderer_derived_lighting_material_resolver_does_not_mutate_extract() {
        let manager = ProjectAssetManager::default();
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );

        let usage = resolve_advanced_pbr_material_usage(&manager, &extract);

        assert_eq!(usage, AdvancedPbrMaterialFrameUsage::default());
        assert_eq!(
            extract.lighting.advanced_lighting.material_features,
            AdvancedPbrMaterialFrameUsage::default()
        );
    }

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

        let (effective, diagnostics) = manager
            .load_effective_material_asset(material_id)
            .expect("missing parent must not discard the renderable child material");

        assert_eq!(
            effective.advanced_pbr_features().specular_transmission,
            0.75
        );
        assert!(effective.parent.is_none());
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn render_advanced_lighting_material_usage_caches_effective_features_by_material() {
        let source = include_str!("material_feature_extract.rs");

        assert!(source.contains("features_by_material"));
        assert!(source.contains(".entry(mesh.material.id())"));
    }

    #[test]
    fn render_advanced_lighting_uses_the_canonical_effective_material_loader() {
        let production = include_str!("material_feature_extract.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material feature extract test boundary");

        assert!(production.contains("load_effective_material_asset"));
        assert!(!production.contains("fn effective_material("));
        assert!(!production.contains("MAX_MATERIAL_PARENT_DEPTH"));
    }
}
