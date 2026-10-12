use crate::scene::dynamic_scene::{
    DynamicScene, EntityRemap, ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap,
    ScenePatchPreviewReport, ScenePatchPreviewResource,
};
use crate::scene::World;

pub(super) fn build_preview_report(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
    resources: Vec<ScenePatchPreviewResource>,
) -> ScenePatchPreviewReport {
    let component_type_count = scene.component_types.len();
    let existing_component_type_count = scene
        .component_types
        .iter()
        .filter(|descriptor| {
            world
                .component_type_descriptor(&descriptor.type_id)
                .is_some()
        })
        .count();
    let new_component_type_count =
        component_type_count.saturating_sub(existing_component_type_count);
    let component_types = scene
        .component_types
        .iter()
        .map(|descriptor| ScenePatchPreviewComponentType {
            type_id: descriptor.type_id.clone(),
            plugin_id: descriptor.plugin_id.clone(),
            display_name: descriptor.display_name.clone(),
            already_registered: world
                .component_type_descriptor(&descriptor.type_id)
                .is_some(),
        })
        .collect();
    let remapped_entity_count = remap
        .iter()
        .filter(|(source, target)| source != target)
        .count();
    let entity_remaps = remap
        .iter()
        .map(
            |(source_entity, target_entity)| ScenePatchPreviewEntityRemap {
                source_entity,
                target_entity,
            },
        )
        .collect();
    let entity_count = scene.entities.len();
    let component_instance_count: usize = scene
        .entities
        .iter()
        .map(|entity| entity.components.len())
        .sum();
    ScenePatchPreviewReport {
        component_type_count,
        existing_component_type_count,
        new_component_type_count,
        component_instance_count,
        entity_count,
        resource_count: scene.resources.len(),
        target_entity_count: world.node_records().len(),
        preserved_entity_count: entity_count.saturating_sub(remapped_entity_count),
        remapped_entity_count,
        component_types,
        resources,
        entity_remaps,
    }
}
