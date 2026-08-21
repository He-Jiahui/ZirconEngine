#[cfg(feature = "first-party-editor-catalog")]
use zircon_editor::EditorPluginRegistrationReport;
#[cfg(feature = "first-party-editor-catalog")]
use zircon_runtime::core::framework::project::ProjectPluginManifest;

#[cfg(feature = "first-party-editor-catalog")]
use super::EntryConfig;

#[cfg(feature = "first-party-editor-catalog")]
pub fn first_party_editor_plugin_registrations_for_config(
    config: &EntryConfig,
) -> Vec<EditorPluginRegistrationReport> {
    let manifest = config.project_plugin_manifest().unwrap_or_default();
    first_party_editor_plugin_registrations_for_manifest(config.target_mode, &manifest)
}

#[cfg(feature = "first-party-editor-catalog")]
pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    first_party_editor_plugin_registrations_for_manifest_impl(target_mode, manifest)
}

#[cfg(feature = "first-party-editor-catalog")]
fn first_party_editor_plugin_registrations_for_manifest_impl(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    zircon_first_party_editor_catalog::first_party_editor_plugin_registrations_for_manifest(
        target_mode,
        manifest,
    )
}

#[cfg(all(test, feature = "first-party-editor-catalog"))]
mod tests {
    use std::time::Instant;

    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

    use super::first_party_editor_plugin_registrations_for_manifest;

    #[cfg(feature = "first-party-navigation-editor-plugin")]
    #[test]
    fn app_composition_projects_selected_navigation_editor_provider() {
        let manifest = ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, false)
                    .with_target_modes([RuntimeTargetMode::EditorHost]),
            ],
        };

        let registrations = first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &manifest,
        );

        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "navigation");
        assert_eq!(
            registrations[0].runtime_event_consumers.manifests().len(),
            1
        );
        assert_provider_resolution_performance(RuntimePluginId::Navigation, "navigation");
    }

    #[cfg(feature = "first-party-neural-editor-plugin")]
    #[test]
    fn app_composition_projects_selected_neural_editor_provider() {
        let manifest = editor_manifest(RuntimePluginId::new("neural"));

        let registrations = first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &manifest,
        );

        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "neural");
        assert!(
            registrations[0]
                .capabilities
                .iter()
                .any(|capability| capability == "editor.extension.neural_authoring")
        );
        assert_provider_resolution_performance(RuntimePluginId::new("neural"), "neural");
    }

    fn assert_provider_resolution_performance(plugin_id: RuntimePluginId, provider: &str) {
        const SAMPLE_COUNT: usize = 21;
        const ITERATIONS: usize = 1_024;
        const MAX_P95_US: u128 = 250_000;

        let manifest = editor_manifest(plugin_id);
        let mut samples = Vec::with_capacity(SAMPLE_COUNT);
        for _ in 0..SAMPLE_COUNT {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let registrations = first_party_editor_plugin_registrations_for_manifest(
                    RuntimeTargetMode::EditorHost,
                    &manifest,
                );
                assert_eq!(registrations.len(), 1);
            }
            samples.push(started.elapsed().as_micros());
        }
        samples.sort_unstable();
        let p50_us = nearest_rank(&samples, 50);
        let p95_us = nearest_rank(&samples, 95);
        assert!(
            p95_us <= MAX_P95_US,
            "{provider} editor provider resolution P95 {p95_us}us exceeds {MAX_P95_US}us"
        );
        println!(
            "PERF-MVP-PLUGINS06 provider={provider} sample_count={SAMPLE_COUNT} \
             iterations={ITERATIONS} p50_us={p50_us} p95_us={p95_us} \
             max_p95_us={MAX_P95_US} registration_count=1"
        );
    }

    fn editor_manifest(plugin_id: RuntimePluginId) -> ProjectPluginManifest {
        ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(plugin_id, true, false)
                    .with_target_modes([RuntimeTargetMode::EditorHost]),
            ],
        }
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let rank = samples.len().saturating_mul(percentile).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }
}
