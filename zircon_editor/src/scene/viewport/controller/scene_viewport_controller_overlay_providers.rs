use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;
use zircon_runtime::scene::Scene;

use crate::core::editor_extension::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use crate::core::plugin::run_editor_plugin_boundary;
use crate::scene::modes::ViewportOverlayBuilder;

use super::SceneViewportController;

#[derive(Clone, Debug, Default)]
pub(crate) struct ViewportOverlayProviderRegistry {
    enabled_capabilities: BTreeSet<String>,
    providers: BTreeMap<String, ActiveViewportOverlayProvider>,
}

#[derive(Clone)]
struct ActiveViewportOverlayProvider {
    owner_id: String,
    required_capabilities: Vec<String>,
    provider: Arc<dyn ViewportOverlayProvider>,
    enabled: bool,
    faulted: Cell<bool>,
    last_failure: RefCell<Option<String>>,
}

impl std::fmt::Debug for ActiveViewportOverlayProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ActiveViewportOverlayProvider")
            .field("owner_id", &self.owner_id)
            .field("required_capabilities", &self.required_capabilities)
            .field("enabled", &self.enabled)
            .field("faulted", &self.faulted)
            .field("last_failure", &self.last_failure)
            .finish_non_exhaustive()
    }
}

impl ViewportOverlayProviderRegistry {
    fn validate_install(
        &self,
        registrations: &[ViewportOverlayProviderRegistration],
    ) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for registration in registrations {
            let provider_id = registration.provider_id();
            if self.providers.contains_key(provider_id) || !ids.insert(provider_id.to_string()) {
                return Err(format!(
                    "duplicate viewport overlay provider `{provider_id}`"
                ));
            }
        }
        Ok(())
    }

    fn prepare(
        &self,
        owner_id: &str,
        registrations: impl IntoIterator<Item = ViewportOverlayProviderRegistration>,
    ) -> Result<Self, String> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        self.validate_install(&registrations)?;
        let mut candidate = self.clone();
        for registration in registrations {
            let provider_id = registration.provider_id().to_string();
            let provider = ActiveViewportOverlayProvider {
                owner_id: owner_id.to_string(),
                required_capabilities: registration.required_capabilities().to_vec(),
                provider: registration.create(),
                enabled: false,
                faulted: Cell::new(false),
                last_failure: RefCell::new(None),
            };
            candidate.providers.insert(provider_id, provider);
        }
        Ok(candidate)
    }

    fn set_enabled_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.enabled_capabilities = capabilities
            .into_iter()
            .map(|capability| capability.as_ref().to_string())
            .collect();
        for provider in self.providers.values_mut() {
            if !provider
                .required_capabilities
                .iter()
                .all(|capability| self.enabled_capabilities.contains(capability))
            {
                provider.enabled = false;
            }
        }
    }

    fn toggle(&mut self, provider_id: &str) -> Result<bool, String> {
        let provider = self
            .providers
            .get_mut(provider_id)
            .ok_or_else(|| format!("unknown viewport overlay provider `{provider_id}`"))?;
        if provider.faulted.get() {
            let message = provider.last_failure.borrow();
            return Err(format!(
                "viewport overlay provider `{provider_id}` is quarantined after callback failure: {}",
                message.as_deref().unwrap_or("unknown callback failure")
            ));
        }
        let missing = provider
            .required_capabilities
            .iter()
            .filter(|capability| !self.enabled_capabilities.contains(capability.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(format!(
                "viewport overlay provider `{provider_id}` requires disabled capabilities: {}",
                missing.join(", ")
            ));
        }
        provider.enabled = !provider.enabled;
        Ok(provider.enabled)
    }

    fn extract(&self, scene: &Scene, selected: Option<u64>) -> Vec<SceneGizmoOverlayExtract> {
        let context = ViewportOverlayProviderContext::new(scene, selected);
        self.providers
            .values()
            .filter(|provider| {
                provider.enabled
                    && !provider.faulted.get()
                    && provider
                        .required_capabilities
                        .iter()
                        .all(|capability| self.enabled_capabilities.contains(capability))
            })
            .flat_map(|provider| {
                match run_editor_plugin_boundary(
                    &provider.owner_id,
                    "viewport overlay extraction",
                    || Ok(provider.provider.extract(&context)),
                ) {
                    Ok(gizmos) => gizmos,
                    Err(error) => {
                        provider.faulted.set(true);
                        provider.last_failure.replace(Some(error.to_string()));
                        Vec::new()
                    }
                }
            })
            .collect()
    }
}

impl SceneViewportController {
    pub(crate) fn prepare_viewport_overlay_providers(
        &self,
        owner_id: &str,
        registrations: impl IntoIterator<Item = ViewportOverlayProviderRegistration>,
    ) -> Result<ViewportOverlayProviderRegistry, String> {
        self.overlay_providers.prepare(owner_id, registrations)
    }

    pub(crate) fn install_prepared_viewport_overlay_providers(
        &mut self,
        registry: ViewportOverlayProviderRegistry,
    ) {
        self.overlay_providers = registry;
        self.interaction_extract.invalidate();
    }

    pub(crate) fn set_viewport_overlay_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.overlay_providers
            .set_enabled_capabilities(capabilities);
        self.interaction_extract.invalidate();
    }

    pub(in crate::scene::viewport) fn toggle_viewport_overlay_provider(
        &mut self,
        provider_id: &str,
    ) -> Result<bool, String> {
        let enabled = self.overlay_providers.toggle(provider_id)?;
        self.interaction_extract.invalidate();
        Ok(enabled)
    }

    pub(in crate::scene::viewport) fn viewport_overlay_gizmos(
        &self,
        scene: &Scene,
        selected: Option<u64>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        let mut builder = ViewportOverlayBuilder::default();
        self.state.scene_modes.build_overlay(&mut builder);
        let mut gizmos = builder.finish();
        gizmos.extend(self.overlay_providers.extract(scene, selected));
        gizmos
    }
}
