use std::collections::BTreeSet;

use crate::plugin::ComponentTypeDescriptor;

use super::{DynamicScene, DYNAMIC_SCENE_FORMAT_VERSION};
use crate::scene::dynamic_scene::DynamicSceneError;

pub(super) fn ensure_scene_supported(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    validate_format_version(scene)?;
    ensure_component_type_descriptors(scene)?;
    ensure_unique_sources(scene)
}

fn validate_format_version(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    if scene.format_version == DYNAMIC_SCENE_FORMAT_VERSION {
        return Ok(());
    }
    Err(DynamicSceneError::UnsupportedFormatVersion {
        expected: DYNAMIC_SCENE_FORMAT_VERSION,
        actual: scene.format_version,
    })
}

fn ensure_unique_sources(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    let mut seen = BTreeSet::new();
    for entity in &scene.entities {
        if !seen.insert(entity.source_entity) {
            return Err(DynamicSceneError::DuplicateSourceEntity {
                entity: entity.source_entity,
            });
        }
    }
    Ok(())
}

fn ensure_component_type_descriptors(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    let mut seen = BTreeSet::new();
    for descriptor in &scene.component_types {
        if !seen.insert(descriptor.type_id.clone()) {
            return Err(DynamicSceneError::DuplicateComponentTypeDescriptor {
                type_id: descriptor.type_id.clone(),
            });
        }
        validate_component_type_descriptor(descriptor)?;
    }
    Ok(())
}

fn validate_component_type_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<(), DynamicSceneError> {
    if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
        return Err(DynamicSceneError::InvalidComponentTypeDescriptor {
            type_id: descriptor.type_id.clone(),
            reason: format!(
                "component type must be prefixed by plugin id `{}`",
                descriptor.plugin_id
            ),
        });
    }
    crate::scene::reflect::registration_from_component_descriptor(descriptor).map_err(|error| {
        DynamicSceneError::InvalidComponentTypeDescriptor {
            type_id: descriptor.type_id.clone(),
            reason: error.to_string(),
        }
    })?;
    Ok(())
}

fn component_type_belongs_to_plugin(type_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = type_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}
