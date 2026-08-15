use std::sync::{Arc, Mutex};

use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, ViewportOverlayProvider,
    ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

use crate::overlay::{
    build_navigation_overlay, NavigationOverlayOptions, NAVIGATION_OVERLAY_PROVIDER_ID,
};
use crate::runtime_mirror::NavigationPieMirror;
use crate::NAVIGATION_GIZMOS_CAPABILITY;

pub(crate) struct NavigationViewportOverlayProvider {
    mirror: Arc<Mutex<NavigationPieMirror>>,
}

impl NavigationViewportOverlayProvider {
    pub(crate) fn new(mirror: Arc<Mutex<NavigationPieMirror>>) -> Self {
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
        vec![build_navigation_overlay(
            selected.unwrap_or_default(),
            &frame.nav_mesh,
            Some(frame),
            NavigationOverlayOptions::default(),
        )]
    }
}

impl ViewportOverlayProvider for NavigationViewportOverlayProvider {
    fn extract(
        &self,
        context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        self.extract_current(context.selected())
    }
}

pub(crate) fn navigation_viewport_overlay_provider_registration(
    mirror: Arc<Mutex<NavigationPieMirror>>,
) -> ViewportOverlayProviderRegistration {
    ViewportOverlayProviderRegistration::new(NAVIGATION_OVERLAY_PROVIDER_ID, move || {
        Arc::new(NavigationViewportOverlayProvider::new(mirror.clone()))
            as Arc<dyn ViewportOverlayProvider>
    })
    .with_required_capabilities([NAVIGATION_GIZMOS_CAPABILITY])
}

pub(crate) fn register_navigation_viewport_overlay_provider(
    registry: &mut EditorExtensionRegistry,
    mirror: Arc<Mutex<NavigationPieMirror>>,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_viewport_overlay_provider(navigation_viewport_overlay_provider_registration(
        mirror,
    ))
}
