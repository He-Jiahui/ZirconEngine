use std::collections::BTreeMap;

use crate::core::editor_authoring_extension::ViewportToolModeDescriptor;
use crate::core::editor_message::SceneModeId;

use super::{EditorSceneMode, SceneModeRegistration, SceneModeRegistryError};

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
        let mode = registration.create();
        if mode.id() != mode_id {
            return Err(SceneModeRegistryError::FactoryModeIdMismatch {
                registered_mode_id: mode_id.clone(),
                produced_mode_id: mode.id().clone(),
            });
        }
        Ok(mode)
    }

    pub fn descriptor(&self, mode_id: &SceneModeId) -> Option<&ViewportToolModeDescriptor> {
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
