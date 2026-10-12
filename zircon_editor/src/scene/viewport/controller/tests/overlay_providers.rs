use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::editor_extension::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
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
