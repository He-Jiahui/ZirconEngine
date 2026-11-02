use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, ViewportOverlayProvider,
    ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

use crate::capability::AI_DEBUG_CAPABILITY;
use crate::extension_ids::AI_PERCEPTION_OVERLAY_PROVIDER_ID;
use crate::overlay::build_ai_perception_overlay;
use crate::runtime_mirror::AiPieMirror;

pub(crate) struct AiViewportOverlayProvider {
    mirror: Arc<Mutex<AiPieMirror>>,
}

impl AiViewportOverlayProvider {
    pub(crate) fn new(mirror: Arc<Mutex<AiPieMirror>>) -> Self {
        Self { mirror }
    }

    pub(crate) fn extract_current(&self, selected: Option<u64>) -> Vec<SceneGizmoOverlayExtract> {
        let mirror = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Like navigation/physics, extract accepted data from the active PIE
        // mirror. Keep each world's geometry separate, including repeated IDs.
        let worlds = mirror
            .agents()
            .map(|frame| frame.report.world)
            .collect::<BTreeSet<_>>();
        worlds
            .iter()
            .map(|world| build_ai_perception_overlay(selected.unwrap_or_default(), world, &mirror))
            .filter(|overlay| !overlay.pick_shapes.is_empty())
            .collect()
    }
}

impl ViewportOverlayProvider for AiViewportOverlayProvider {
    fn extract(
        &self,
        context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        self.extract_current(context.selected())
    }
}

pub(crate) fn ai_viewport_overlay_provider_registration(
    mirror: Arc<Mutex<AiPieMirror>>,
) -> ViewportOverlayProviderRegistration {
    ViewportOverlayProviderRegistration::new(AI_PERCEPTION_OVERLAY_PROVIDER_ID, move || {
        Arc::new(AiViewportOverlayProvider::new(mirror.clone())) as Arc<dyn ViewportOverlayProvider>
    })
    .with_required_capabilities([AI_DEBUG_CAPABILITY])
}

pub(crate) fn register_ai_viewport_overlay_provider(
    registry: &mut EditorExtensionRegistry,
    mirror: Arc<Mutex<AiPieMirror>>,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_viewport_overlay_provider(ai_viewport_overlay_provider_registration(mirror))
}
