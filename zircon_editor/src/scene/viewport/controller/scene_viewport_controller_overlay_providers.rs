use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use thiserror::Error;
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;
use zircon_runtime::scene::Scene;

use crate::core::editor_extension::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use crate::core::extension::{ContributionSource, ContributionTicket};
use crate::core::plugin::run_editor_plugin_boundary;
use crate::scene::modes::ViewportOverlayBuilder;

use super::SceneViewportController;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ViewportOverlayProviderError {
    #[error("duplicate viewport overlay provider `{provider_id}`")]
    DuplicateProvider { provider_id: String },
    #[error("unknown viewport overlay provider `{provider_id}`")]
    UnknownProvider { provider_id: String },
    #[error(
        "viewport overlay provider `{provider_id}` is quarantined after callback failure: {}",
        detail.as_deref().unwrap_or("unknown callback failure")
    )]
    Quarantined {
        provider_id: String,
        detail: Option<String>,
    },
    #[error(
        "viewport overlay provider `{provider_id}` requires disabled capabilities: {}",
        missing.join(", ")
    )]
    DisabledCapabilities {
        provider_id: String,
        missing: Vec<String>,
    },
    #[error("viewport overlay provider `{provider_id}` cleanup failed: {message}")]
    CleanupFailure {
        provider_id: String,
        message: String,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ViewportOverlayProviderRegistry {
    enabled_capabilities: BTreeSet<String>,
    providers: BTreeMap<String, ActiveViewportOverlayProvider>,
}

#[derive(Clone)]
struct ActiveViewportOverlayProvider {
    owner_id: String,
    contribution_owner: ViewportOverlayProviderContributionOwner,
    required_capabilities: Vec<String>,
    provider: Arc<dyn ViewportOverlayProvider>,
    enabled: bool,
    faulted: Arc<AtomicBool>,
    last_failure: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Debug)]
struct ViewportOverlayProviderContributionOwner {
    ticket: ContributionTicket,
    source: ContributionSource,
}

pub(crate) struct ViewportOverlayProviderRetirement {
    retired: Vec<(String, ActiveViewportOverlayProvider)>,
}

impl std::fmt::Debug for ActiveViewportOverlayProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last_failure = self
            .last_failure
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        formatter
            .debug_struct("ActiveViewportOverlayProvider")
            .field("owner_id", &self.owner_id)
            .field("contribution_owner", &self.contribution_owner)
            .field("required_capabilities", &self.required_capabilities)
            .field("enabled", &self.enabled)
            .field("faulted", &self.faulted.load(Ordering::Acquire))
            .field("last_failure", &*last_failure)
            .finish_non_exhaustive()
    }
}

impl ViewportOverlayProviderRegistry {
    fn validate_install(
        &self,
        registrations: &[ViewportOverlayProviderRegistration],
    ) -> Result<(), ViewportOverlayProviderError> {
        let mut ids = BTreeSet::new();
        for registration in registrations {
            let provider_id = registration.provider_id();
            if self.providers.contains_key(provider_id) || !ids.insert(provider_id.to_string()) {
                return Err(ViewportOverlayProviderError::DuplicateProvider {
                    provider_id: provider_id.to_string(),
                });
            }
        }
        Ok(())
    }

    fn prepare_contribution(
        &self,
        ticket: ContributionTicket,
        source: ContributionSource,
        owner_id: &str,
        registrations: impl IntoIterator<Item = ViewportOverlayProviderRegistration>,
    ) -> Result<Self, ViewportOverlayProviderError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        self.validate_install(&registrations)?;
        let mut candidate = self.clone();
        for registration in registrations {
            let provider_id = registration.provider_id().to_string();
            let provider = ActiveViewportOverlayProvider {
                owner_id: owner_id.to_string(),
                contribution_owner: ViewportOverlayProviderContributionOwner {
                    ticket,
                    source: source.clone(),
                },
                required_capabilities: registration.required_capabilities().to_vec(),
                provider: registration.create(),
                enabled: false,
                faulted: Arc::new(AtomicBool::new(false)),
                last_failure: Arc::new(Mutex::new(None)),
            };
            candidate.providers.insert(provider_id, provider);
        }
        Ok(candidate)
    }

    fn without_contribution(
        &self,
        ticket: ContributionTicket,
    ) -> (Self, ViewportOverlayProviderRetirement) {
        let mut candidate = self.clone();
        let provider_ids = candidate
            .providers
            .iter()
            .filter(|(_, provider)| provider.contribution_owner.ticket == ticket)
            .map(|(provider_id, _)| provider_id.clone())
            .collect::<Vec<_>>();
        let retired = provider_ids
            .into_iter()
            .filter_map(|provider_id| {
                candidate
                    .providers
                    .remove(&provider_id)
                    .map(|provider| (provider_id, provider))
            })
            .collect();
        (candidate, ViewportOverlayProviderRetirement { retired })
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

    fn toggle(&mut self, provider_id: &str) -> Result<bool, ViewportOverlayProviderError> {
        let provider = self.providers.get_mut(provider_id).ok_or_else(|| {
            ViewportOverlayProviderError::UnknownProvider {
                provider_id: provider_id.to_string(),
            }
        })?;
        if provider.faulted.load(Ordering::Acquire) {
            let message = provider
                .last_failure
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            return Err(ViewportOverlayProviderError::Quarantined {
                provider_id: provider_id.to_string(),
                detail: message.clone(),
            });
        }
        let missing = provider
            .required_capabilities
            .iter()
            .filter(|capability| !self.enabled_capabilities.contains(capability.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(ViewportOverlayProviderError::DisabledCapabilities {
                provider_id: provider_id.to_string(),
                missing,
            });
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
                    && !provider.faulted.load(Ordering::Acquire)
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
                        *provider
                            .last_failure
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                            Some(error.to_string());
                        provider.faulted.store(true, Ordering::Release);
                        Vec::new()
                    }
                }
            })
            .collect()
    }
}

impl ViewportOverlayProviderRetirement {
    #[cfg(test)]
    fn provider_ids(&self) -> Vec<&str> {
        self.retired
            .iter()
            .map(|(provider_id, _)| provider_id.as_str())
            .collect()
    }

    pub(crate) fn cleanup(self) -> Result<(), ViewportOverlayProviderError> {
        let mut first_error = None;
        for (provider_id, provider) in self.retired {
            let owner_id = provider.owner_id.clone();
            if let Err(error) =
                run_editor_plugin_boundary(&owner_id, "viewport overlay provider drop", move || {
                    drop(provider);
                    Ok(())
                })
            {
                first_error.get_or_insert(ViewportOverlayProviderError::CleanupFailure {
                    provider_id,
                    message: error.to_string(),
                });
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl SceneViewportController {
    pub(crate) fn prepare_viewport_overlay_providers(
        &self,
        ticket: ContributionTicket,
        source: ContributionSource,
        owner_id: &str,
        registrations: impl IntoIterator<Item = ViewportOverlayProviderRegistration>,
    ) -> Result<ViewportOverlayProviderRegistry, ViewportOverlayProviderError> {
        self.overlay_providers
            .prepare_contribution(ticket, source, owner_id, registrations)
    }

    pub(crate) fn prepare_viewport_overlay_provider_retirement(
        &self,
        ticket: ContributionTicket,
    ) -> (
        ViewportOverlayProviderRegistry,
        ViewportOverlayProviderRetirement,
    ) {
        self.overlay_providers.without_contribution(ticket)
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
    ) -> Result<bool, ViewportOverlayProviderError> {
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::core::editor_extension::{
        ViewportOverlayProvider, ViewportOverlayProviderContext,
        ViewportOverlayProviderRegistration,
    };
    use crate::core::extension::{
        ContributionBatch, ContributionSource, ContributionStore, ContributionTicket,
        PluginContributionId,
    };
    use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

    use super::{ViewportOverlayProviderError, ViewportOverlayProviderRegistry};

    struct EmptyOverlayProvider;

    impl ViewportOverlayProvider for EmptyOverlayProvider {
        fn extract(
            &self,
            _context: &ViewportOverlayProviderContext<'_>,
        ) -> Vec<SceneGizmoOverlayExtract> {
            Vec::new()
        }
    }

    fn registration(
        provider_id: &str,
        required_capabilities: impl IntoIterator<Item = &'static str>,
    ) -> ViewportOverlayProviderRegistration {
        ViewportOverlayProviderRegistration::new(provider_id, || {
            Arc::new(EmptyOverlayProvider) as Arc<dyn ViewportOverlayProvider>
        })
        .with_required_capabilities(required_capabilities)
    }

    fn plugin_owner(
        store: &mut ContributionStore,
        plugin_id: &str,
    ) -> (ContributionTicket, ContributionSource) {
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(plugin_id).expect("plugin id should be valid"),
        );
        let ticket = store
            .contribute(source.clone(), ContributionBatch::default())
            .expect("an empty contribution batch should allocate a ticket");
        (ticket, source)
    }

    #[test]
    fn toggle_reports_an_unknown_provider_with_its_typed_id() {
        let mut registry = ViewportOverlayProviderRegistry::default();

        let error = registry.toggle("missing.viewport.overlay").unwrap_err();

        assert!(matches!(
            error,
            ViewportOverlayProviderError::UnknownProvider { provider_id }
                if provider_id == "missing.viewport.overlay"
        ));
    }

    #[test]
    fn prepare_rejects_duplicate_provider_ids_with_a_typed_error() {
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_owner(&mut store, "test");
        let registry = ViewportOverlayProviderRegistry::default();

        let error = registry
            .prepare_contribution(
                ticket,
                source,
                "test",
                [
                    registration("test.viewport.overlay", []),
                    registration("test.viewport.overlay", []),
                ],
            )
            .unwrap_err();

        assert!(matches!(
            error,
            ViewportOverlayProviderError::DuplicateProvider { provider_id }
                if provider_id == "test.viewport.overlay"
        ));
    }

    #[test]
    fn toggle_reports_missing_capabilities_without_flattening_them() {
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_owner(&mut store, "test");
        let mut registry = ViewportOverlayProviderRegistry::default()
            .prepare_contribution(
                ticket,
                source,
                "test",
                [registration("test.viewport.overlay", ["render.debug"])],
            )
            .unwrap();

        let error = registry.toggle("test.viewport.overlay").unwrap_err();

        assert!(matches!(
            error,
            ViewportOverlayProviderError::DisabledCapabilities {
                provider_id,
                missing,
            } if provider_id == "test.viewport.overlay" && missing == ["render.debug"]
        ));
    }

    #[test]
    fn ticket_retirement_candidate_preserves_live_and_other_provider_state() {
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_owner(&mut store, "weather");
        let (lighting_ticket, lighting_source) = plugin_owner(&mut store, "lighting");
        let mut live = ViewportOverlayProviderRegistry::default()
            .prepare_contribution(
                weather_ticket,
                weather_source,
                "weather",
                [registration("plugin.weather.overlay", [])],
            )
            .unwrap()
            .prepare_contribution(
                lighting_ticket,
                lighting_source,
                "lighting",
                [registration("plugin.lighting.overlay", [])],
            )
            .unwrap();
        assert!(live.toggle("plugin.weather.overlay").unwrap());
        assert!(live.toggle("plugin.lighting.overlay").unwrap());

        let (mut candidate, retirement) = live.without_contribution(weather_ticket);

        assert_eq!(retirement.provider_ids(), ["plugin.weather.overlay"]);
        assert!(!live.toggle("plugin.weather.overlay").unwrap());
        assert!(matches!(
            candidate.toggle("plugin.weather.overlay"),
            Err(ViewportOverlayProviderError::UnknownProvider { provider_id })
                if provider_id == "plugin.weather.overlay"
        ));
        assert!(!candidate.toggle("plugin.lighting.overlay").unwrap());
    }

    #[test]
    fn retired_provider_drop_is_deferred_until_cleanup_after_publication() {
        struct DropProbe(Arc<AtomicUsize>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        impl ViewportOverlayProvider for DropProbe {
            fn extract(
                &self,
                _context: &ViewportOverlayProviderContext<'_>,
            ) -> Vec<SceneGizmoOverlayExtract> {
                Vec::new()
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        let provider_drops = Arc::clone(&drops);
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_owner(&mut store, "weather");
        let live = ViewportOverlayProviderRegistry::default()
            .prepare_contribution(
                ticket,
                source,
                "weather",
                [ViewportOverlayProviderRegistration::new(
                    "plugin.weather.overlay",
                    move || {
                        Arc::new(DropProbe(Arc::clone(&provider_drops)))
                            as Arc<dyn ViewportOverlayProvider>
                    },
                )],
            )
            .unwrap();

        let (candidate, retirement) = live.without_contribution(ticket);
        drop(live);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(candidate);

        retirement.cleanup().unwrap();

        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
