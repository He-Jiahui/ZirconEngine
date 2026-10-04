use crate::asset::project::ProjectManager;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::world::SceneComponentSerializer;

use super::RuntimeExtensionRegistry;

/// Registers owner qualified scene codecs on a project owner before Worlds are created.
///
/// The extension registry remains the authority for descriptors and owner lifetime.  Runtime
/// callers register the descriptor in `RuntimeExtensionRegistry`, then attach the matching
/// callback here before native plugin code can be unloaded.  The owner revocation listener
/// removes callbacks from the project registry before the registry permits plugin unload.
impl RuntimeExtensionRegistry {
    /// Records a callback supplied by a plugin owner. The descriptor must already be registered
    /// under that same owner; the normal project-open path applies this list to its ProjectManager.
    pub fn register_scene_component_codec_for_owner(
        &mut self,
        owner: crate::plugin::PluginModuleId,
        serializer: SceneComponentSerializer,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let descriptor = self
            .components
            .iter()
            .find(|(entry_owner, key, descriptor)| {
                *entry_owner == owner
                    && key.as_str() == serializer.type_id
                    && descriptor.plugin_id == serializer.provider_id
            })
            .map(|(_, _, descriptor)| descriptor)
            .ok_or_else(|| {
                RuntimeExtensionRegistryError::InvalidComponentType(format!(
                    "scene codec {} has no owner-qualified descriptor",
                    serializer.type_id
                ))
            })?;
        if descriptor.type_id != serializer.type_id
            || descriptor.plugin_id != serializer.provider_id
            || serializer.schema_id.is_empty()
            || serializer.schema_version == 0
        {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!(
                    "scene codec {} identity does not match its owner descriptor",
                    serializer.type_id
                ),
            ));
        }
        if self
            .scene_component_codecs
            .iter()
            .any(|(_, registered)| registered.type_id == serializer.type_id)
        {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!("duplicate scene codec {}", serializer.type_id),
            ));
        }
        self.scene_component_codecs.push((owner, serializer));
        Ok(())
    }

    /// Applies codec callbacks through the real runtime project-open ingress. A failed callback
    /// admission leaves the ProjectManager unchanged and reports the owner-qualified failure.
    pub fn apply_scene_component_codecs_to_project_manager(
        &mut self,
        project: &ProjectManager,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let before = self.clone();
        let project_before = project.scene_component_registry_snapshot();
        self.finalize();
        let codecs = self.scene_component_codecs.clone();
        for (_owner, serializer) in codecs {
            if let Err(error) =
                self.register_scene_component_codec_for_project_manager(project, serializer)
            {
                // A later duplicate must not leave an earlier provider visible on the
                // ProjectManager. Restore the exact owner snapshot before returning so the
                // extension and project registries remain one transaction.
                project.restore_scene_component_registry(project_before.clone());
                *self = before;
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn register_scene_component_codec_for_project_manager(
        &mut self,
        project: &ProjectManager,
        serializer: SceneComponentSerializer,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let before = self.clone();
        let project_before = project.scene_component_registry_snapshot();
        let result = (|| {
            // Project admission runs before the extension transaction is published. A rejected
            // duplicate therefore rolls both owner tables back to their exact prior state.
            self.finalize();
            let (owner, descriptor) = self
                .components
                .iter()
                .find(|(_, key, descriptor)| {
                    key.as_str() == serializer.type_id
                        && descriptor.plugin_id == serializer.provider_id
                })
                .map(|(owner, _, descriptor)| (owner, descriptor.clone()))
                .ok_or_else(|| {
                    RuntimeExtensionRegistryError::InvalidComponentType(format!(
                        "scene codec {} has no matching registered component descriptor",
                        serializer.type_id
                    ))
                })?;
            if serializer.type_id != descriptor.type_id
                || serializer.provider_id != descriptor.plugin_id
                || serializer.schema_id.is_empty()
                || serializer.schema_version == 0
            {
                return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                    format!(
                    "scene codec {} identity does not match its registered component descriptor",
                    serializer.type_id
                ),
                ));
            }
            project
                .register_scene_component_provider_for_owner(owner, serializer, descriptor)
                .map_err(|error| {
                    RuntimeExtensionRegistryError::WorldRegistration(error.to_string())
                })?;
            let weak_registry = project.scene_component_registry_weak();
            self.register_owner_revocation_listener(owner, move |revoked_owner| {
                ProjectManager::revoke_scene_component_provider_owner_from_weak(
                    &weak_registry,
                    revoked_owner,
                );
            });
            Ok(())
        })();
        if result.is_err() {
            project.restore_scene_component_registry(project_before);
            *self = before;
        }
        result
    }

    /// Registers the descriptor and codec together using the existing plugin id contract.
    /// Repeated calls are rejected by the typed extension point before ProjectManager changes.
    pub fn register_scene_component_provider_for_project_manager(
        &mut self,
        project: &ProjectManager,
        descriptor: ComponentTypeDescriptor,
        serializer: SceneComponentSerializer,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if descriptor.type_id != serializer.type_id
            || descriptor.plugin_id != serializer.provider_id
            || serializer.schema_id.is_empty()
            || serializer.schema_version == 0
        {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!(
                    "scene codec {} identity does not match its component descriptor",
                    serializer.type_id
                ),
            ));
        }
        let before = self.clone();
        if let Err(error) = self.register_component(descriptor) {
            return Err(error);
        }
        if let Err(error) =
            self.register_scene_component_codec_for_project_manager(project, serializer)
        {
            *self = before;
            return Err(error);
        }
        Ok(())
    }
}
