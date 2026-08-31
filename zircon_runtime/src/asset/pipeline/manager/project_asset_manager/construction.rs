use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::resource::{ResourceManager, RuntimeResourceState};
use crate::core::CoreError;

use super::super::builtins::resource_manager_with_builtins;
use super::super::errors::asset_error_message;
use super::management_generation::ProjectAssetManagementGeneration;
use super::project_asset_manager::{ProjectSourcePathIndex, ProjectWatcherActivation};
use super::ProjectAssetManager;
use crate::asset::artifact::IblBakeArtifactCacheStore;
use crate::asset::project::ProjectManager;
use crate::asset::worker_pool::{
    AssetWorkerPool, AssetWorkerPoolFrameSampler, AssetWorkerPoolOptions,
    AssetWorkerThreadBudgetSource,
};
use crate::asset::{
    AssetId, AssetImportError, AssetImporter, AssetImporterCapabilityReport, AssetImporterHandler,
    AssetImporterRegistry, AssetUri, ShaderAsset,
};
use crate::core::runtime::tasks::{TaskPool, TaskPoolKind, TaskPools};

const DEFAULT_ASSET_WORKER_QUEUE_DEPTH_PER_THREAD: usize = 2;

impl Default for ProjectAssetManager {
    fn default() -> Self {
        Self::new(TaskPools::default().io().clone())
    }
}

impl ProjectAssetManager {
    pub fn new(worker_task_pool: TaskPool) -> Self {
        assert_eq!(
            worker_task_pool.kind(),
            TaskPoolKind::Io,
            "ProjectAssetManager requires the runtime IO task pool"
        );
        let manager = Self {
            worker_task_pool,
            project_generation_gate: Arc::new(RwLock::new(())),
            project_preparation_epoch: Arc::new(AtomicU64::new(0)),
            project: Arc::new(RwLock::new(None)),
            asset_management_generation: Arc::new(RwLock::new(Arc::new(
                ProjectAssetManagementGeneration::empty(),
            ))),
            project_source_paths: Arc::new(RwLock::new(ProjectSourcePathIndex::new())),
            asset_importers: Arc::new(RwLock::new(AssetImporter::default().registry().clone())),
            resource_manager: resource_manager_with_builtins(),
            residency_stripes: Arc::new(std::array::from_fn(|_| Mutex::new(()))),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            generation_wake_subscribers: Arc::new(Mutex::new(Vec::new())),
            watch_error_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher_activation: Arc::new(Mutex::new(Option::<Arc<ProjectWatcherActivation>>::None)),
            watch_refresh_gate: Arc::new(Mutex::new(())),
            watch_diagnostics: Arc::new(Mutex::new(Default::default())),
            transaction_watch_echoes: Arc::new(Mutex::new(Default::default())),
            watchers: Arc::new(Mutex::new(Vec::new())),
        };
        manager.refresh_asset_management_generation();
        manager
    }

    pub fn spawn_worker_pool_with_frame_sampler(
        &self,
    ) -> (AssetWorkerPool, AssetWorkerPoolFrameSampler) {
        let queue_depth = self
            .worker_task_pool
            .parallelism()
            .saturating_mul(DEFAULT_ASSET_WORKER_QUEUE_DEPTH_PER_THREAD);
        let pool = AssetWorkerPool::new(
            self.worker_task_pool.clone(),
            AssetWorkerPoolOptions::new().with_queue_depth(queue_depth),
        );
        let sampler = AssetWorkerPoolFrameSampler::from_pool(&pool);
        (pool, sampler)
    }

    pub fn worker_task_pool(&self) -> &TaskPool {
        &self.worker_task_pool
    }

    pub fn default_worker_count(&self) -> usize {
        self.worker_task_pool.parallelism()
    }

    pub fn default_worker_budget_source(&self) -> AssetWorkerThreadBudgetSource {
        AssetWorkerThreadBudgetSource::TaskPoolIo
    }

    pub fn resource_manager(&self) -> ResourceManager {
        self.resource_manager.clone()
    }

    pub fn register_asset_importer(
        &self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), CoreError> {
        self.register_asset_importer_arc(Arc::new(importer))
    }

    pub fn register_asset_importer_arc(
        &self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), CoreError> {
        {
            let project = self.project_read();
            if let Some(project) = project.as_ref() {
                let mut active_registry = project.importer().registry().clone();
                active_registry
                    .register_arc(importer.clone())
                    .map_err(|error| asset_error_message(error.to_string()))?;
            }
        }

        self.importer_registry_write()
            .register_arc(importer.clone())
            .map_err(|error| asset_error_message(error.to_string()))?;
        self.begin_project_preparation();

        let mut project = self.project_write();
        if let Some(project) = project.as_mut() {
            project
                .register_asset_importer_arc(importer)
                .map_err(|error| asset_error_message(error.to_string()))?;
        }
        Ok(())
    }

    pub fn asset_importer_capability_reports(&self) -> Vec<AssetImporterCapabilityReport> {
        let project = self.project_read();
        if let Some(project) = project.as_ref() {
            return project.importer().capability_reports();
        }
        self.active_importer_registry().capability_reports()
    }

    pub fn asset_importer_capability_report_for_source(
        &self,
        source_path: &std::path::Path,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError> {
        let project = self.project_read();
        if let Some(project) = project.as_ref() {
            return project.importer().capability_report_for_source(source_path);
        }
        self.active_importer_registry()
            .capability_report_for_source(source_path)
    }

    #[cfg(test)]
    pub(crate) fn register_first_wave_plugin_fixture_importers_for_test(
        &self,
    ) -> Result<(), CoreError> {
        for importer in AssetImporter::first_wave_plugin_fixture_importers_for_test() {
            self.register_asset_importer(importer)?;
        }
        Ok(())
    }

    pub fn resolve_asset_id(&self, uri: &AssetUri) -> Option<AssetId> {
        self.resource_manager()
            .registry()
            .get_by_locator(uri)
            .map(|record| record.id())
    }

    pub fn current_project_manager(&self) -> Option<ProjectManager> {
        self.project_read().clone()
    }

    pub fn ibl_bake_artifact_cache_store(&self) -> Option<IblBakeArtifactCacheStore> {
        let project = self.project_read();
        project
            .as_ref()
            .map(|project| IblBakeArtifactCacheStore::new(project.paths().cache_root()))
    }

    pub fn runtime_ref_count(&self, id: AssetId) -> Option<usize> {
        self.resource_manager().ref_count(id)
    }

    pub fn runtime_resource_state(&self, id: AssetId) -> Option<RuntimeResourceState> {
        self.resource_manager().runtime_state(id)
    }

    pub fn load_shader_asset_by_uri(&self, uri: &AssetUri) -> Result<ShaderAsset, CoreError> {
        let id = self
            .resolve_asset_id(uri)
            .ok_or_else(|| asset_error_message(format!("missing shader locator {uri}")))?;
        self.load_shader_asset(id)
    }

    fn active_importer_registry(&self) -> AssetImporterRegistry {
        self.importer_registry_read().clone()
    }
}

#[cfg(test)]
mod active_importer_registry_tests {
    use std::hint::black_box;
    use std::path::Path;
    use std::time::Instant;

    use super::*;
    use crate::asset::{AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter};

    const BENCHMARK_IMPORTERS: usize = 96;
    const BENCHMARK_ITERATIONS: usize = 64;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_THRESHOLD_PERCENT: u128 = 90;

    fn fixture_importer(index: usize) -> Arc<dyn AssetImporterHandler> {
        Arc::new(DiagnosticOnlyAssetImporter::new(
            AssetImporterDescriptor::new(
                format!("plugins07.cached_registry.{index}"),
                "plugins07.cached_registry",
                AssetKind::Data,
                1,
            )
            .with_priority(500)
            .with_source_extensions([format!("p7cache{index}")]),
            "performance fixture",
        ))
    }

    fn legacy_active_importer_registry(
        plugin_importers: &[Arc<dyn AssetImporterHandler>],
    ) -> AssetImporterRegistry {
        let mut registry = AssetImporter::default().registry().clone();
        for importer in plugin_importers {
            registry.register_arc(importer.clone()).unwrap();
        }
        registry
    }

    fn measure_registry_acquisition(
        iterations: usize,
        source_path: &Path,
        mut acquire: impl FnMut() -> AssetImporterRegistry,
    ) -> u128 {
        let timer = Instant::now();
        let mut checksum = 0_i64;
        for _ in 0..iterations {
            let registry = black_box(acquire());
            checksum += i64::from(
                registry
                    .select(black_box(source_path))
                    .unwrap()
                    .descriptor()
                    .priority,
            );
        }
        black_box(checksum);
        timer.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 - 1) / 100]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn cached_active_importer_registry_owns_default_generation() {
        let manager = ProjectAssetManager::default();
        let expected = AssetImporter::default().registry().descriptors();

        assert!(
            !expected.is_empty(),
            "default importer registry is nonempty"
        );
        assert_eq!(manager.importer_registry_read().descriptors(), expected);
    }

    #[test]
    fn cached_active_importer_registry_rejects_conflict_without_mutation() {
        let manager = ProjectAssetManager::default();
        let before = manager.importer_registry_read().descriptors();
        let duplicate = before
            .first()
            .expect("default registry is nonempty")
            .clone();

        let error = manager
            .register_asset_importer(DiagnosticOnlyAssetImporter::new(
                duplicate,
                "duplicate fixture",
            ))
            .expect_err("duplicate default importer must be rejected");

        assert!(error.to_string().contains("already registered"));
        assert_eq!(manager.importer_registry_read().descriptors(), before);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_cached_active_importer_registry_acquisition() {
        let manager = ProjectAssetManager::default();
        let plugin_importers = (0..BENCHMARK_IMPORTERS)
            .map(fixture_importer)
            .collect::<Vec<_>>();
        for importer in &plugin_importers {
            manager
                .register_asset_importer_arc(importer.clone())
                .unwrap();
        }
        let source_path = Path::new("fixture.p7cache95");
        assert_eq!(
            legacy_active_importer_registry(&plugin_importers)
                .select(source_path)
                .unwrap()
                .descriptor()
                .id,
            manager
                .active_importer_registry()
                .select(source_path)
                .unwrap()
                .descriptor()
                .id
        );

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_registry_acquisition(
                    BENCHMARK_ITERATIONS,
                    source_path,
                    || legacy_active_importer_registry(&plugin_importers),
                ));
                optimized_samples.push(measure_registry_acquisition(
                    BENCHMARK_ITERATIONS,
                    source_path,
                    || manager.active_importer_registry(),
                ));
            } else {
                optimized_samples.push(measure_registry_acquisition(
                    BENCHMARK_ITERATIONS,
                    source_path,
                    || manager.active_importer_registry(),
                ));
                legacy_samples.push(measure_registry_acquisition(
                    BENCHMARK_ITERATIONS,
                    source_path,
                    || legacy_active_importer_registry(&plugin_importers),
                ));
            }
        }

        let legacy_raw = legacy_samples.clone();
        let optimized_raw = optimized_samples.clone();
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);

        println!(
            "PERF_RESULT plugins07_cached_active_importer_registry importers={} iterations_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_registry_rebuilds_per_sample={} optimized_registry_rebuilds_per_sample=0 legacy_plugin_registrations_per_sample={} optimized_plugin_registrations_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
            BENCHMARK_IMPORTERS,
            BENCHMARK_ITERATIONS,
            BENCHMARK_SAMPLE_PAIRS,
            BENCHMARK_ITERATIONS,
            BENCHMARK_IMPORTERS * BENCHMARK_ITERATIONS,
            legacy_p95_ns,
            optimized_p95_ns,
            improvement_percent,
            BENCHMARK_THRESHOLD_PERCENT,
            sample_csv(&legacy_raw),
            sample_csv(&optimized_raw),
        );

        assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
        assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
        assert!(
            improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
            "cached active importer registry P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
        );
    }
}
