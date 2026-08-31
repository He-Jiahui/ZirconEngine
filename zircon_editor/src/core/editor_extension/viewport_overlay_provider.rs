use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;
use zircon_runtime::scene::Scene;

/// Immutable scene data supplied to a viewport overlay provider for one render extraction.
pub struct ViewportOverlayProviderContext<'a> {
    scene: &'a Scene,
    selected: Option<u64>,
}

impl<'a> ViewportOverlayProviderContext<'a> {
    pub(crate) fn new(scene: &'a Scene, selected: Option<u64>) -> Self {
        Self { scene, selected }
    }

    pub fn scene(&self) -> &'a Scene {
        self.scene
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }
}

/// Plugin-owned producer for editor viewport gizmo extracts.
pub trait ViewportOverlayProvider: Send + Sync {
    fn extract(
        &self,
        context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract>;
}

pub trait ViewportOverlayProviderFactory: Send + Sync {
    fn create(&self) -> Arc<dyn ViewportOverlayProvider>;
}

impl<F> ViewportOverlayProviderFactory for F
where
    F: Fn() -> Arc<dyn ViewportOverlayProvider> + Send + Sync,
{
    fn create(&self) -> Arc<dyn ViewportOverlayProvider> {
        self()
    }
}

#[derive(Clone)]
pub struct ViewportOverlayProviderRegistration {
    provider_id: String,
    required_capabilities: Vec<String>,
    factory: Arc<dyn ViewportOverlayProviderFactory>,
}

impl ViewportOverlayProviderRegistration {
    pub fn new<F>(provider_id: impl Into<String>, factory: F) -> Self
    where
        F: ViewportOverlayProviderFactory + 'static,
    {
        Self {
            provider_id: provider_id.into(),
            required_capabilities: Vec::new(),
            factory: Arc::new(factory),
        }
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let capabilities = capabilities.into_iter();
        let (lower_bound, _) = capabilities.size_hint();
        self.required_capabilities.reserve(lower_bound);
        self.required_capabilities
            .extend(capabilities.map(Into::into));
        self.required_capabilities.sort_unstable();
        self.required_capabilities.dedup();
        self
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub(crate) fn create(&self) -> Arc<dyn ViewportOverlayProvider> {
        self.factory.create()
    }
}

impl fmt::Debug for ViewportOverlayProviderRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ViewportOverlayProviderRegistration")
            .field("provider_id", &self.provider_id)
            .field("required_capabilities", &self.required_capabilities)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ViewportOverlayProviderRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.provider_id == other.provider_id
            && self.required_capabilities == other.required_capabilities
            && Arc::ptr_eq(&self.factory, &other.factory)
    }
}

#[cfg(test)]
#[path = "viewport_overlay_provider/optimization_tests.rs"]
mod optimization_tests;
