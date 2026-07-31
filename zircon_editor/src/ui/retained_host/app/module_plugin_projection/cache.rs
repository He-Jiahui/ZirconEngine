use std::sync::Arc;

use crate::ui::host::EditorPluginStatusReport;
use crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData;

/// Derived retained-host presentation keyed by the immutable manager-owned status report.
///
/// This cache is not a plugin catalog authority. A new host status `Arc` naturally makes the
/// previous presentation unreachable, so the pane rebuilds exactly once for a new generation.
#[derive(Default)]
pub(in crate::ui::retained_host::app) struct ModulePluginPaneProjectionCache {
    cached: Option<CachedModulePluginPane>,
}

struct CachedModulePluginPane {
    status_report: Arc<EditorPluginStatusReport>,
    pane: ModulePluginsPaneViewData,
}

impl ModulePluginPaneProjectionCache {
    pub(in crate::ui::retained_host::app) fn get_or_build(
        &mut self,
        status_report: &Arc<EditorPluginStatusReport>,
        build: impl FnOnce(&EditorPluginStatusReport) -> ModulePluginsPaneViewData,
    ) -> ModulePluginsPaneViewData {
        if let Some(pane) = self.cached(status_report) {
            return pane;
        }

        let pane = build(status_report.as_ref());
        self.store(Arc::clone(status_report), pane.clone());
        pane
    }

    fn cached(
        &self,
        status_report: &Arc<EditorPluginStatusReport>,
    ) -> Option<ModulePluginsPaneViewData> {
        self.cached
            .as_ref()
            .filter(|cached| Arc::ptr_eq(&cached.status_report, status_report))
            .map(|cached| cached.pane.clone())
    }

    fn store(
        &mut self,
        status_report: Arc<EditorPluginStatusReport>,
        pane: ModulePluginsPaneViewData,
    ) {
        self.cached = Some(CachedModulePluginPane {
            status_report,
            pane,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Instant;

    use crate::ui::host::EditorPluginStatus;
    use crate::ui::retained_host::app::module_plugin_projection::pane_data::module_plugins_pane_from_status_report;
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;

    use super::*;

    #[test]
    fn cached_pane_is_reused_only_for_the_same_status_arc() {
        let report = Arc::new(EditorPluginStatusReport::default());
        let mut cache = ModulePluginPaneProjectionCache::default();
        let mut build_count = 0;

        cache.get_or_build(&report, |_| {
            build_count += 1;
            ModulePluginsPaneViewData::default()
        });
        cache.get_or_build(&report, |_| {
            build_count += 1;
            ModulePluginsPaneViewData::default()
        });
        cache.get_or_build(&Arc::new(EditorPluginStatusReport::default()), |_| {
            build_count += 1;
            ModulePluginsPaneViewData::default()
        });

        assert_eq!(build_count, 2);
    }

    #[test]
    fn one_thousand_stable_projection_reads_reuse_one_generation_at_1_100_1000_plugin_scales() {
        const STABLE_READS: usize = 1_000;

        for plugin_count in [1, 100, 1_000] {
            let status_report = status_report_with_plugin_count(plugin_count);
            let mut cache = ModulePluginPaneProjectionCache::default();
            let warm_pane =
                cache.get_or_build(&status_report, module_plugins_pane_from_status_report);
            assert_eq!(warm_pane.plugins.row_count(), plugin_count);

            let mut elapsed_micros = Vec::with_capacity(STABLE_READS);
            let mut stable_projection_build_count = 0;
            let mut stable_projection_clone_bytes = 0;
            for _ in 0..STABLE_READS {
                let started_at = Instant::now();
                let pane = cache.get_or_build(&status_report, |report| {
                    stable_projection_build_count += 1;
                    stable_projection_clone_bytes += projected_status_payload_bytes(report);
                    module_plugins_pane_from_status_report(report)
                });
                elapsed_micros.push(started_at.elapsed().as_micros());
                assert_eq!(pane.plugins.row_count(), plugin_count);
            }
            let p95_micros = p95_micros(&mut elapsed_micros);

            println!(
                "EDITOR12_PLUGIN_PANE_STABLE_READ plugins={plugin_count} samples={STABLE_READS} p95_us={p95_micros} stable_projection_build_count={stable_projection_build_count} stable_projection_clone_bytes={stable_projection_clone_bytes}"
            );
            assert_eq!(stable_projection_build_count, 0);
            assert_eq!(stable_projection_clone_bytes, 0);
        }
    }

    fn status_report_with_plugin_count(plugin_count: usize) -> Arc<EditorPluginStatusReport> {
        Arc::new(EditorPluginStatusReport {
            plugins: (0..plugin_count)
                .map(|index| EditorPluginStatus {
                    plugin_id: format!("test.plugin.{index:04}"),
                    display_name: format!("Test Plugin {index:04}"),
                    package_source: "test".to_string(),
                    load_state: "Active".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: Vec::new(),
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: None,
                    runtime_capabilities: Vec::new(),
                    editor_capabilities: Vec::new(),
                    optional_features: Vec::new(),
                    diagnostics: Vec::new(),
                })
                .collect(),
            diagnostics: vec!["stable projection diagnostic".to_string()],
        })
    }

    fn projected_status_payload_bytes(report: &EditorPluginStatusReport) -> usize {
        report
            .plugins
            .iter()
            .map(|plugin| {
                plugin.plugin_id.len()
                    + plugin.display_name.len()
                    + plugin.package_source.len()
                    + plugin.load_state.len()
                    + plugin.runtime_crate.as_deref().map_or(0, str::len)
                    + plugin.editor_crate.as_deref().map_or(0, str::len)
                    + plugin
                        .runtime_capabilities
                        .iter()
                        .map(String::len)
                        .sum::<usize>()
                    + plugin
                        .editor_capabilities
                        .iter()
                        .map(String::len)
                        .sum::<usize>()
                    + plugin.diagnostics.iter().map(String::len).sum::<usize>()
            })
            .sum::<usize>()
            + report.diagnostics.iter().map(String::len).sum::<usize>()
    }

    fn p95_micros(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let p95_index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[p95_index]
    }
}
