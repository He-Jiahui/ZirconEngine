//! 材质特性按当前相机可见网格解析，供高级 PBR 图通道门控而不修改共享场景提取。
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
#[path = "tests/material_feature_extract.rs"]
mod tests;
