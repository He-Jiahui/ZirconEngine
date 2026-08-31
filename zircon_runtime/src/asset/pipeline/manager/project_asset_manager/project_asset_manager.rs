use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::framework::channel::{ChannelSender, ChannelWakeCallback};
use crate::core::resource::{ResourceManager, ResourceScheme};
use crate::core::runtime::tasks::TaskPool;

use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchBatchDiagnostics, AssetWatchError, AssetWatcher};
use crate::asset::{AssetImporterRegistry, AssetUri};

use super::management_generation::ProjectAssetManagementGeneration;
use super::source_write_watch_echo::TransactionWatchEchoes;
use super::watch_diagnostics::ProjectAssetWatchDiagnostics;

pub(in crate::asset::pipeline::manager) type ProjectSourcePathIndex =
    HashMap<ResourceScheme, HashMap<String, PathBuf>>;

pub(in crate::asset::pipeline::manager) const PROJECT_RESIDENCY_STRIPE_COUNT: usize = 64;

pub(in crate::asset::pipeline::manager) struct ProjectAssetChangeSubscriber {
    sender: ChannelSender<AssetChange>,
    wake: Option<ChannelWakeCallback>,
}

pub(in crate::asset::pipeline::manager) struct ProjectAssetGenerationWakeSubscriber {
    sender: ChannelSender<()>,
    wake: ChannelWakeCallback,
}

impl ProjectAssetGenerationWakeSubscriber {
    pub(in crate::asset::pipeline::manager) fn new(
        sender: ChannelSender<()>,
        wake: ChannelWakeCallback,
    ) -> Self {
        Self { sender, wake }
    }

    pub(in crate::asset::pipeline::manager) fn try_enqueue(&self) -> Option<bool> {
        match self.sender.try_send(()) {
            Ok(()) => Some(true),
            Err(crossbeam_channel::TrySendError::Full(())) => Some(false),
            Err(crossbeam_channel::TrySendError::Disconnected(())) => None,
        }
    }

    pub(in crate::asset::pipeline::manager) fn wake_callback(&self) -> ChannelWakeCallback {
        Arc::clone(&self.wake)
    }
}

impl ProjectAssetChangeSubscriber {
    pub(in crate::asset::pipeline::manager) fn new(
        sender: ChannelSender<AssetChange>,
        wake: Option<ChannelWakeCallback>,
    ) -> Self {
        Self { sender, wake }
    }

    pub(in crate::asset::pipeline::manager) fn send(&self, change: AssetChange) -> bool {
        self.sender.send(change).is_ok()
    }

    pub(in crate::asset::pipeline::manager) fn wake(&self) {
        if let Some(wake) = self.wake.as_ref() {
            wake();
        }
    }
}

pub(in crate::asset::pipeline::manager) struct ProjectWatcherActivation {
    pub(in crate::asset::pipeline::manager) state: Mutex<ProjectWatcherActivationState>,
}

pub(in crate::asset::pipeline::manager) struct ProjectWatcherActivationState {
    pub(in crate::asset::pipeline::manager) lifecycle: ProjectWatcherLifecycle,
    pub(in crate::asset::pipeline::manager) changes: Vec<AssetChange>,
    pub(in crate::asset::pipeline::manager) coalescible_change_indices: HashMap<AssetUri, usize>,
    pub(in crate::asset::pipeline::manager) queued_change_bytes: usize,
    pub(in crate::asset::pipeline::manager) requires_reconciliation: bool,
    pub(in crate::asset::pipeline::manager) diagnostics: AssetWatchBatchDiagnostics,
    pub(in crate::asset::pipeline::manager) errors: VecDeque<AssetWatchError>,
    pub(in crate::asset::pipeline::manager) worker_scheduled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::asset::pipeline::manager) enum ProjectWatcherLifecycle {
    Pending,
    Draining,
    Active,
    Retired,
}

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) worker_task_pool: TaskPool,
    pub(in crate::asset::pipeline::manager) project_generation_gate: Arc<RwLock<()>>,
    pub(in crate::asset::pipeline::manager) project_preparation_epoch: Arc<AtomicU64>,
    pub(in crate::asset::pipeline::manager) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(in crate::asset::pipeline::manager) asset_management_generation:
        Arc<RwLock<Arc<ProjectAssetManagementGeneration>>>,
    pub(in crate::asset::pipeline::manager) project_source_paths:
        Arc<RwLock<ProjectSourcePathIndex>>,
    pub(in crate::asset::pipeline::manager) asset_importers: Arc<RwLock<AssetImporterRegistry>>,
    pub(in crate::asset::pipeline::manager) resource_manager: ResourceManager,
    pub(in crate::asset::pipeline::manager) residency_stripes:
        Arc<[Mutex<()>; PROJECT_RESIDENCY_STRIPE_COUNT]>,
    pub(in crate::asset::pipeline::manager) change_subscribers:
        Arc<Mutex<Vec<ProjectAssetChangeSubscriber>>>,
    pub(in crate::asset::pipeline::manager) generation_wake_subscribers:
        Arc<Mutex<Vec<ProjectAssetGenerationWakeSubscriber>>>,
    pub(in crate::asset::pipeline::manager) watch_error_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetWatchError>>>>,
    pub(in crate::asset::pipeline::manager) watcher_activation:
        Arc<Mutex<Option<Arc<ProjectWatcherActivation>>>>,
    pub(in crate::asset::pipeline::manager) watch_refresh_gate: Arc<Mutex<()>>,
    pub(in crate::asset::pipeline::manager) watch_diagnostics:
        Arc<Mutex<ProjectAssetWatchDiagnostics>>,
    pub(in crate::asset::pipeline::manager) transaction_watch_echoes:
        Arc<Mutex<TransactionWatchEchoes>>,
    pub(in crate::asset::pipeline::manager) watchers: Arc<Mutex<Vec<AssetWatcher>>>,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crossbeam_channel::unbounded;

    use super::{ProjectAssetChangeSubscriber, ProjectAssetManager};
    use crate::asset::watch::{AssetChange, AssetChangeKind};
    use crate::asset::AssetUri;

    #[test]
    fn delivered_asset_change_invokes_the_subscription_wake() {
        let (sender, receiver) = unbounded();
        let wake_count = Arc::new(AtomicUsize::new(0));
        let wake_count_for_callback = Arc::clone(&wake_count);
        let subscriber = ProjectAssetChangeSubscriber::new(
            sender,
            Some(Arc::new(move || {
                wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
            })),
        );
        let change = AssetChange::new(
            AssetChangeKind::Modified,
            AssetUri::parse("res://textures/albedo.png").unwrap(),
            None,
        );

        assert!(subscriber.send(change));
        subscriber.wake();

        assert_eq!(receiver.len(), 1);
        assert_eq!(wake_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn project_generation_wake_tokens_coalesce_at_capacity_one() {
        let manager = ProjectAssetManager::default();
        let wake_count = Arc::new(AtomicUsize::new(0));
        let wake_count_for_callback = Arc::clone(&wake_count);
        let receiver = manager.subscribe_project_generation_wake(Arc::new(move || {
            wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
        }));
        let change = AssetChange::new(
            AssetChangeKind::Modified,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            None,
        );

        let generation = manager.project_generation_write();
        manager.publish_project_generation(generation, vec![change.clone()]);
        let generation = manager.project_generation_write();
        manager.publish_project_generation(generation, vec![change.clone()]);
        assert_eq!(receiver.len(), 1);
        assert_eq!(wake_count.load(Ordering::Relaxed), 1);

        receiver.try_recv().unwrap();
        let generation = manager.project_generation_write();
        manager.publish_project_generation(generation, vec![change]);
        assert_eq!(receiver.len(), 1);
        assert_eq!(wake_count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn empty_committed_project_generation_still_wakes_reactive_consumers() {
        let manager = ProjectAssetManager::default();
        let wake_count = Arc::new(AtomicUsize::new(0));
        let wake_count_for_callback = Arc::clone(&wake_count);
        let receiver = manager.subscribe_project_generation_wake(Arc::new(move || {
            wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
        }));
        let generation = manager.project_generation_write();

        manager.publish_project_generation(generation, Vec::new());

        assert_eq!(receiver.len(), 1);
        assert_eq!(wake_count.load(Ordering::Relaxed), 1);
    }
}
