use std::fmt;
use std::sync::Arc;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::extension::{ContributionSource, ContributionTicket};

use super::{EditorSceneMode, SceneModeFactory, SceneModeRegistryError};

#[derive(Clone, Debug)]
struct SceneModeContributionOwner {
    ticket: ContributionTicket,
    source: ContributionSource,
}

#[derive(Clone)]
pub struct SceneModeRegistration {
    mode_id: SceneModeId,
    descriptor: SceneModeDescriptor,
    factory: Arc<dyn SceneModeFactory>,
    owner_id: String,
    contribution_owner: Option<SceneModeContributionOwner>,
}

impl SceneModeRegistration {
    pub fn new<F>(descriptor: SceneModeDescriptor, factory: F) -> Self
    where
        F: SceneModeFactory + 'static,
    {
        Self::from_factory(descriptor, Arc::new(factory))
    }

    pub fn from_factory(
        descriptor: SceneModeDescriptor,
        factory: Arc<dyn SceneModeFactory>,
    ) -> Self {
        let mode_id = SceneModeId::new(descriptor.id());
        Self {
            mode_id,
            descriptor,
            factory,
            owner_id: "editor.scene.direct".to_string(),
            contribution_owner: None,
        }
    }

    pub(crate) fn with_owner_id(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = owner_id.into();
        self
    }

    pub(crate) fn bind_contribution_owner(
        mut self,
        ticket: ContributionTicket,
        source: ContributionSource,
        owner_id: impl Into<String>,
    ) -> Result<Self, SceneModeRegistryError> {
        if self.contribution_owner.is_some() {
            return Err(SceneModeRegistryError::ContributionAlreadyOwned {
                mode_id: self.mode_id.clone(),
            });
        }
        self.owner_id = owner_id.into();
        self.contribution_owner = Some(SceneModeContributionOwner { ticket, source });
        Ok(self)
    }

    pub fn mode_id(&self) -> &SceneModeId {
        &self.mode_id
    }

    pub fn descriptor(&self) -> &SceneModeDescriptor {
        &self.descriptor
    }

    pub(crate) fn create(&self) -> Box<dyn EditorSceneMode> {
        self.factory.create()
    }

    pub(crate) fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub(crate) fn contribution_ticket(&self) -> Option<ContributionTicket> {
        self.contribution_owner.as_ref().map(|owner| owner.ticket)
    }
}

impl fmt::Debug for SceneModeRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneModeRegistration")
            .field("mode_id", &self.mode_id)
            .field("descriptor", &self.descriptor)
            .field("owner_id", &self.owner_id)
            .field("contribution_owner", &self.contribution_owner)
            .finish_non_exhaustive()
    }
}

impl PartialEq for SceneModeRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.mode_id == other.mode_id
            && self.descriptor == other.descriptor
            && self.owner_id == other.owner_id
            && self
                .contribution_owner
                .as_ref()
                .map(|owner| (&owner.ticket, &owner.source))
                == other
                    .contribution_owner
                    .as_ref()
                    .map(|owner| (&owner.ticket, &owner.source))
            && Arc::ptr_eq(&self.factory, &other.factory)
    }
}
