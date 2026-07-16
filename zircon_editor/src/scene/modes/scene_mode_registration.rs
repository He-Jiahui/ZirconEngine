use std::fmt;
use std::sync::Arc;

use crate::core::editor_authoring_extension::ViewportToolModeDescriptor;
use crate::core::editor_message::SceneModeId;

use super::{EditorSceneMode, SceneModeFactory};

#[derive(Clone)]
pub struct SceneModeRegistration {
    mode_id: SceneModeId,
    descriptor: ViewportToolModeDescriptor,
    factory: Arc<dyn SceneModeFactory>,
}

impl SceneModeRegistration {
    pub fn new<F>(descriptor: ViewportToolModeDescriptor, factory: F) -> Self
    where
        F: SceneModeFactory + 'static,
    {
        Self::from_factory(descriptor, Arc::new(factory))
    }

    pub fn from_factory(
        descriptor: ViewportToolModeDescriptor,
        factory: Arc<dyn SceneModeFactory>,
    ) -> Self {
        let mode_id = SceneModeId::new(descriptor.id());
        Self {
            mode_id,
            descriptor,
            factory,
        }
    }

    pub fn mode_id(&self) -> &SceneModeId {
        &self.mode_id
    }

    pub fn descriptor(&self) -> &ViewportToolModeDescriptor {
        &self.descriptor
    }

    pub(crate) fn create(&self) -> Box<dyn EditorSceneMode> {
        self.factory.create()
    }
}

impl fmt::Debug for SceneModeRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneModeRegistration")
            .field("mode_id", &self.mode_id)
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}
