use std::collections::{HashMap, hash_map::Entry};

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::extension::ContributionTicket;
use crate::core::plugin::run_editor_plugin_boundary;

use super::{
    EditorSceneMode, SceneModeRegistration, SceneModeRegistryError,
    isolated_scene_mode::IsolatedSceneMode,
};

#[derive(Clone, Debug, Default)]
pub struct SceneModeRegistry {
    registrations: HashMap<SceneModeId, SceneModeRegistration>,
    ordered_mode_ids: Vec<SceneModeId>,
}

impl SceneModeRegistry {
    pub fn register(
        &mut self,
        registration: SceneModeRegistration,
    ) -> Result<(), SceneModeRegistryError> {
        let mode_id = registration.mode_id().clone();
        match self.registrations.entry(mode_id.clone()) {
            Entry::Occupied(_) => Err(SceneModeRegistryError::DuplicateMode { mode_id }),
            Entry::Vacant(entry) => {
                let ordered_index = self
                    .ordered_mode_ids
                    .partition_point(|registered| registered < &mode_id);
                self.ordered_mode_ids.insert(ordered_index, mode_id);
                entry.insert(registration);
                Ok(())
            }
        }
    }

    pub fn create(
        &self,
        mode_id: &SceneModeId,
    ) -> Result<Box<dyn EditorSceneMode>, SceneModeRegistryError> {
        self.create_with_contribution(mode_id).map(|(mode, _)| mode)
    }

    pub(crate) fn create_with_contribution(
        &self,
        mode_id: &SceneModeId,
    ) -> Result<(Box<dyn EditorSceneMode>, Option<ContributionTicket>), SceneModeRegistryError>
    {
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
        Ok((Box::new(isolated), registration.contribution_ticket()))
    }

    pub(crate) fn without_contribution(
        &self,
        ticket: ContributionTicket,
    ) -> (Self, Vec<SceneModeId>) {
        let removed = self
            .registrations()
            .filter(|registration| registration.contribution_ticket() == Some(ticket))
            .map(|registration| registration.mode_id().clone())
            .collect::<Vec<_>>();
        let mut candidate = self.clone();
        candidate
            .registrations
            .retain(|_, registration| registration.contribution_ticket() != Some(ticket));
        candidate
            .ordered_mode_ids
            .retain(|mode_id| candidate.registrations.contains_key(mode_id));
        (candidate, removed)
    }

    pub fn descriptor(&self, mode_id: &SceneModeId) -> Option<&SceneModeDescriptor> {
        self.registrations
            .get(mode_id)
            .map(SceneModeRegistration::descriptor)
    }

    pub fn registrations(&self) -> impl Iterator<Item = &SceneModeRegistration> {
        self.ordered_mode_ids
            .iter()
            .filter_map(|mode_id| self.registrations.get(mode_id))
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }
}

#[cfg(test)]
#[path = "scene_mode_registry/entry_registration_tests.rs"]
mod entry_registration_tests;

#[cfg(test)]
#[path = "scene_mode_registry/hash_lookup_tests.rs"]
mod hash_lookup_tests;
