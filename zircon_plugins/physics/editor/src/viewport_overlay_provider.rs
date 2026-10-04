use std::sync::{Arc, Mutex};

use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, ViewportOverlayProvider,
    ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

use crate::overlay::{build_physics_overlay_extracts, PHYSICS_OVERLAY_PROVIDER_ID};
use crate::runtime_mirror::PhysicsPieMirror;
use crate::PHYSICS_AUTHORING_CAPABILITY;

pub(crate) struct PhysicsViewportOverlayProvider {
    mirror: Arc<Mutex<PhysicsPieMirror>>,
}

impl PhysicsViewportOverlayProvider {
    pub(crate) fn new(mirror: Arc<Mutex<PhysicsPieMirror>>) -> Self {
        Self { mirror }
    }

    pub(crate) fn extract_current(&self, selected: Option<u64>) -> Vec<SceneGizmoOverlayExtract> {
        let mirror = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(frame) = mirror.frame() else {
            return Vec::new();
        };
        build_physics_overlay_extracts(&frame.overlay_frame, selected)
    }
}

impl ViewportOverlayProvider for PhysicsViewportOverlayProvider {
    fn extract(
        &self,
        context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        self.extract_current(context.selected())
    }
}

pub(crate) fn physics_viewport_overlay_provider_registration(
    mirror: Arc<Mutex<PhysicsPieMirror>>,
) -> ViewportOverlayProviderRegistration {
    ViewportOverlayProviderRegistration::new(PHYSICS_OVERLAY_PROVIDER_ID, move || {
        Arc::new(PhysicsViewportOverlayProvider::new(mirror.clone()))
            as Arc<dyn ViewportOverlayProvider>
    })
    .with_required_capabilities([PHYSICS_AUTHORING_CAPABILITY])
}

pub(crate) fn register_physics_viewport_overlay_provider(
    registry: &mut EditorExtensionRegistry,
    mirror: Arc<Mutex<PhysicsPieMirror>>,
) -> Result<(), EditorExtensionRegistryError> {
    registry
        .register_viewport_overlay_provider(physics_viewport_overlay_provider_registration(mirror))
}
