use std::collections::BTreeMap;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::plugin::run_editor_plugin_boundary;

use super::{
    EditorSceneMode, SceneModeRegistration, SceneModeRegistryError,
    isolated_scene_mode::IsolatedSceneMode,
};

#[derive(Clone, Debug, Default)]
pub struct SceneModeRegistry {
    registrations: BTreeMap<SceneModeId, SceneModeRegistration>,
}

impl SceneModeRegistry {
    pub fn register(
        &mut self,
        registration: SceneModeRegistration,
    ) -> Result<(), SceneModeRegistryError> {
        let mode_id = registration.mode_id().clone();
        if self.registrations.contains_key(&mode_id) {
            return Err(SceneModeRegistryError::DuplicateMode { mode_id });
        }
        self.registrations.insert(mode_id, registration);
        Ok(())
    }

    pub fn create(
        &self,
        mode_id: &SceneModeId,
    ) -> Result<Box<dyn EditorSceneMode>, SceneModeRegistryError> {
        let registration =
            self.registrations
                .get(mode_id)
                .ok_or_else(|| SceneModeRegistryError::UnknownMode {
                    mode_id: mode_id.clone(),
                })?;
        let owner_id = registration.owner_id();
        let mode =
            run_editor_plugin_boundary(
                owner_id,
                "scene mode factory",
                || Ok(registration.create()),
            )
            .map_err(|error| SceneModeRegistryError::CallbackFailure {
                mode_id: mode_id.clone(),
                operation: "factory",
                message: error.to_string(),
            })?;
        let isolated = IsolatedSceneMode::new(owner_id.to_string(), mode_id.clone(), mode);
        let produced_mode_id = isolated.validate_inner_id().map_err(|message| {
            SceneModeRegistryError::CallbackFailure {
                mode_id: mode_id.clone(),
                operation: "id",
                message,
            }
        })?;
        if &produced_mode_id != mode_id {
            return Err(SceneModeRegistryError::FactoryModeIdMismatch {
                registered_mode_id: mode_id.clone(),
                produced_mode_id,
            });
        }
        Ok(Box::new(isolated))
    }

    pub fn descriptor(&self, mode_id: &SceneModeId) -> Option<&SceneModeDescriptor> {
        self.registrations
            .get(mode_id)
            .map(SceneModeRegistration::descriptor)
    }

    pub fn registrations(&self) -> impl Iterator<Item = &SceneModeRegistration> {
        self.registrations.values()
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }
}
