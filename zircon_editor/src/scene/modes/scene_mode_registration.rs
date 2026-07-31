use std::fmt;
use std::sync::Arc;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;

use super::{EditorSceneMode, SceneModeFactory};

#[derive(Clone)]
pub struct SceneModeRegistration {
    mode_id: SceneModeId,
    descriptor: SceneModeDescriptor,
    factory: Arc<dyn SceneModeFactory>,
    owner_id: String,
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
        }
    }

    pub(crate) fn with_owner_id(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = owner_id.into();
        self
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
}

impl fmt::Debug for SceneModeRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneModeRegistration")
            .field("mode_id", &self.mode_id)
            .field("descriptor", &self.descriptor)
            .field("owner_id", &self.owner_id)
            .finish_non_exhaustive()
    }
}

impl PartialEq for SceneModeRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.mode_id == other.mode_id
            && self.descriptor == other.descriptor
            && self.owner_id == other.owner_id
            && Arc::ptr_eq(&self.factory, &other.factory)
    }
}
