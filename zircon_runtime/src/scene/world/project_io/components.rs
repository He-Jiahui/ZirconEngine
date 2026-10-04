//! Owner-bound scene component codec registry.
//!
//! The registry is passed by the World/project persistence caller. It is deliberately
//! not global: provider descriptors and codecs are admitted together and the same
//! registry instance is used for create, save, and reopen.

use std::collections::HashSet;

use crate::asset::assets::{SceneComponentAssetRecord, SceneEntityAsset};
use crate::asset::project::ProjectManager;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::PluginModuleId;
use crate::scene::components::NodeRecord;
use crate::scene::world::World;
use serde_json::Value;

use super::render2d::{capture_mesh2d, capture_sprite2d, instantiate_mesh2d, instantiate_sprite2d};
use super::SceneProjectError;

pub(crate) const SPRITE_TYPE_ID: &str = "zircon.render2d.sprite";
pub(crate) const SPRITE_SCHEMA_ID: &str = "zircon.render2d.sprite.v1";
pub(crate) const MESH_TYPE_ID: &str = "zircon.render2d.mesh";
pub(crate) const MESH_SCHEMA_ID: &str = "zircon.render2d.mesh.v1";
pub(crate) const BUILTIN_PROVIDER_ID: &str = "zircon.runtime.render2d";
pub(crate) const SCHEMA_VERSION: u32 = 1;

/// Callback identity is stable across project documents and artifact cache records.
#[derive(Clone, Copy, Debug)]
pub struct SceneComponentSerializer {
    pub type_id: &'static str,
    pub schema_id: &'static str,
    pub schema_version: u32,
    pub provider_id: &'static str,
    pub capture: fn(
        &ProjectManager,
        &World,
        &NodeRecord,
    ) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError>,
    pub instantiate: fn(
        &ProjectManager,
        &SceneComponentAssetRecord,
        &mut NodeRecord,
    ) -> Result<Option<(String, Value)>, SceneProjectError>,
}

#[derive(Clone, Debug)]
struct RegisteredSerializer {
    serializer: SceneComponentSerializer,
    owner: Option<PluginModuleId>,
    /// A dynamic provider descriptor is installed into every World created by this registry.
    descriptor: Option<ComponentTypeDescriptor>,
}

/// A per-runtime owner of scene component codecs and dynamic component descriptors.
#[derive(Clone, Debug)]
pub(crate) struct SceneComponentSerializerRegistry {
    serializers: Vec<RegisteredSerializer>,
}

const BUILTIN_SERIALIZERS: &[SceneComponentSerializer] = &[
    SceneComponentSerializer {
        type_id: SPRITE_TYPE_ID,
        schema_id: SPRITE_SCHEMA_ID,
        schema_version: SCHEMA_VERSION,
        provider_id: BUILTIN_PROVIDER_ID,
        capture: capture_sprite2d,
        instantiate: instantiate_sprite2d,
    },
    SceneComponentSerializer {
        type_id: MESH_TYPE_ID,
        schema_id: MESH_SCHEMA_ID,
        schema_version: SCHEMA_VERSION,
        provider_id: BUILTIN_PROVIDER_ID,
        capture: capture_mesh2d,
        instantiate: instantiate_mesh2d,
    },
];

impl SceneComponentSerializerRegistry {
    pub(crate) fn builtin() -> Self {
        Self {
            serializers: BUILTIN_SERIALIZERS
                .iter()
                .copied()
                .map(|serializer| RegisteredSerializer {
                    serializer,
                    owner: None,
                    descriptor: None,
                })
                .collect(),
        }
    }

    /// Admit a provider codec and its World descriptor as one owner-bound operation.
    ///
    /// The registry is owned by the caller that creates/saves/reopens a World.  A
    /// descriptor is retained beside its callbacks and installed into each World at
    /// the load boundary.  All identity and duplicate checks happen before the
    /// registry is mutated.
    pub(crate) fn register_provider(
        &mut self,
        serializer: SceneComponentSerializer,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), SceneProjectError> {
        self.register_provider_for_owner(None, serializer, descriptor)
    }

    pub(crate) fn register_provider_for_owner(
        &mut self,
        owner: Option<PluginModuleId>,
        serializer: SceneComponentSerializer,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), SceneProjectError> {
        self.validate_registration(serializer, &descriptor)?;
        self.serializers.push(RegisteredSerializer {
            serializer,
            owner,
            descriptor: Some(descriptor),
        });
        Ok(())
    }

    pub(crate) fn revoke_owner(&mut self, owner: PluginModuleId) {
        self.serializers
            .retain(|registered| registered.owner != Some(owner));
    }

    /// Checks an owner-bound provider without changing this registry. ProjectManager uses
    /// this admission step before the extension registry commits its descriptor so a duplicate
    /// schema/type cannot leave the two owner tables out of sync.
    pub(crate) fn validate_provider_for_owner(
        &self,
        _owner: PluginModuleId,
        serializer: SceneComponentSerializer,
        descriptor: &ComponentTypeDescriptor,
    ) -> Result<(), SceneProjectError> {
        self.validate_registration(serializer, descriptor)
    }

    /// A dynamic row must have a live codec before a World can be published or saved. Built-in
    /// script/prefab rows are persisted by their dedicated fields; every other row is owned by
    /// this registry and a revoked provider fails closed before any document write.
    pub(crate) fn validate_live_world_components(
        &self,
        world: &World,
    ) -> Result<(), SceneProjectError> {
        const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";
        const PREFAB_INSTANCE_COMPONENT: &str = "zircon.prefab.instance";
        for entity in &world.entities {
            for instance in world.dynamic_components_for_entity(*entity) {
                if instance.component_id == SCRIPT_BINDINGS_COMPONENT
                    || instance.component_id == PREFAB_INSTANCE_COMPONENT
                {
                    continue;
                }
                if !self
                    .serializers
                    .iter()
                    .any(|registered| registered.serializer.type_id == instance.component_id)
                {
                    return Err(SceneProjectError::SceneAsset(format!(
                        "dynamic scene component {} has no live serializer; provider may have been revoked",
                        instance.component_id
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_registration(
        &self,
        serializer: SceneComponentSerializer,
        descriptor: &ComponentTypeDescriptor,
    ) -> Result<(), SceneProjectError> {
        if serializer.type_id.is_empty()
            || serializer.schema_id.is_empty()
            || serializer.provider_id.is_empty()
            || serializer.schema_version == 0
        {
            return Err(SceneProjectError::SceneAsset(
                "scene component provider identity must be non-empty and versioned".into(),
            ));
        }
        if descriptor.type_id != serializer.type_id {
            return Err(SceneProjectError::SceneAsset(format!(
                "scene component descriptor type {} does not match codec type {}",
                descriptor.type_id, serializer.type_id
            )));
        }
        if descriptor.plugin_id != serializer.provider_id {
            return Err(SceneProjectError::SceneAsset(format!(
                "scene component descriptor provider {} does not match codec provider {}",
                descriptor.plugin_id, serializer.provider_id
            )));
        }
        if self
            .serializers
            .iter()
            .any(|registered| registered.serializer.type_id == serializer.type_id)
        {
            return Err(SceneProjectError::SceneAsset(format!(
                "duplicate scene component type {}",
                serializer.type_id
            )));
        }
        if self.serializers.iter().any(|registered| {
            registered.serializer.schema_id == serializer.schema_id
                && registered.serializer.schema_version == serializer.schema_version
        }) {
            return Err(SceneProjectError::SceneAsset(format!(
                "duplicate scene component schema {} v{}",
                serializer.schema_id, serializer.schema_version
            )));
        }
        Ok(())
    }

    /// Install provider descriptors before dynamic rows are attached to a newly created World.
    pub(crate) fn install_into_world(&self, world: &mut World) -> Result<(), SceneProjectError> {
        let descriptors = self
            .serializers
            .iter()
            .filter_map(|registered| registered.descriptor.clone())
            .collect::<Vec<_>>();
        let mut seen = HashSet::new();
        for descriptor in &descriptors {
            if !seen.insert(descriptor.type_id.as_str())
                || world
                    .component_type_descriptor(&descriptor.type_id)
                    .is_some()
            {
                return Err(SceneProjectError::SceneAsset(format!(
                    "duplicate scene component descriptor {}",
                    descriptor.type_id
                )));
            }
        }
        // Validate the complete descriptor set on a shadow World before touching
        // the caller's World.  This includes descriptor-property and reflection
        // checks that are intentionally owned by World::register_component_type.
        let mut shadow_world = world.clone();
        for descriptor in &descriptors {
            if world
                .component_type_descriptor(&descriptor.type_id)
                .is_some()
                || world
                    .registered_dynamic_component_id(&descriptor.type_id)
                    .is_some()
            {
                return Err(SceneProjectError::SceneAsset(format!(
                    "World already owns scene component descriptor {}",
                    descriptor.type_id
                )));
            }
            shadow_world
                .register_component_type(descriptor.clone())
                .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))?;
        }
        for descriptor in descriptors {
            world
                .register_component_type(descriptor)
                .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))?;
        }
        Ok(())
    }

    pub(crate) fn validate_scene(
        &self,
        scene: &crate::asset::assets::SceneAsset,
    ) -> Result<(), SceneProjectError> {
        scene
            .validate_component_references()
            .map_err(SceneProjectError::SceneAsset)?;
        for entity in &scene.entities {
            self.validate_rows(&entity.components)?;
        }
        Ok(())
    }

    pub(crate) fn capture(
        &self,
        project: &ProjectManager,
        world: &World,
        record: &NodeRecord,
    ) -> Result<Vec<SceneComponentAssetRecord>, SceneProjectError> {
        let mut rows = Vec::new();
        for registered in &self.serializers {
            let serializer = &registered.serializer;
            if let Some(row) = (serializer.capture)(project, world, record)? {
                validate_row_identity(serializer, &row)?;
                rows.push(row);
            }
        }
        Ok(rows)
    }

    pub(crate) fn instantiate(
        &self,
        project: &ProjectManager,
        entity: &SceneEntityAsset,
        record: &mut NodeRecord,
    ) -> Result<Vec<(String, Value)>, SceneProjectError> {
        self.validate_rows(&entity.components)?;
        let mut dynamic_components = Vec::new();
        for row in &entity.components {
            let registered = self
                .serializers
                .iter()
                .find(|registered| registered.serializer.type_id == row.type_id)
                .ok_or_else(|| {
                    SceneProjectError::SceneAsset(format!(
                        "unknown scene component type {}",
                        row.type_id
                    ))
                })?;
            if let Some(dynamic) = (registered.serializer.instantiate)(project, row, record)? {
                dynamic_components.push(dynamic);
            }
        }
        Ok(dynamic_components)
    }

    pub(super) fn validate_rows(
        &self,
        rows: &[SceneComponentAssetRecord],
    ) -> Result<(), SceneProjectError> {
        let mut seen_types = HashSet::new();
        let mut seen_schemas = HashSet::new();
        for row in rows {
            if !seen_types.insert(row.type_id.as_str()) {
                return Err(SceneProjectError::SceneAsset(format!(
                    "duplicate scene component row type {}",
                    row.type_id
                )));
            }
            if !seen_schemas.insert((
                row.schema_id.as_str(),
                row.schema_version,
                row.provider_id.as_str(),
            )) {
                return Err(SceneProjectError::SceneAsset(format!(
                    "duplicate scene component schema {} v{} from {}",
                    row.schema_id, row.schema_version, row.provider_id
                )));
            }
            let registered = self
                .serializers
                .iter()
                .find(|registered| registered.serializer.type_id == row.type_id)
                .ok_or_else(|| {
                    SceneProjectError::SceneAsset(format!(
                        "unknown scene component type {}",
                        row.type_id
                    ))
                })?;
            validate_row_identity(&registered.serializer, row)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn registered_type_ids(&self) -> Vec<&'static str> {
        self.serializers
            .iter()
            .map(|registered| registered.serializer.type_id)
            .collect()
    }
}

fn validate_row_identity(
    serializer: &SceneComponentSerializer,
    row: &SceneComponentAssetRecord,
) -> Result<(), SceneProjectError> {
    if row.schema_id != serializer.schema_id
        || row.schema_version != serializer.schema_version
        || row.provider_id != serializer.provider_id
    {
        return Err(SceneProjectError::SceneAsset(format!(
            "scene component {} schema/provider mismatch: expected {} v{} from {}, got {} v{} from {}",
            row.type_id,
            serializer.schema_id,
            serializer.schema_version,
            serializer.provider_id,
            row.schema_id,
            row.schema_version,
            row.provider_id
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/components.rs"]
mod tests;
