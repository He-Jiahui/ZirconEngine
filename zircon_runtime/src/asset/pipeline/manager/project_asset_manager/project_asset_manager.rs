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
use super::watcher_lifecycle::ProjectWatcherAdmission;

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
    pub(super) watcher_admission: Arc<ProjectWatcherAdmission>,
    pub(in crate::asset::pipeline::manager) watch_refresh_gate: Arc<Mutex<()>>,
    pub(in crate::asset::pipeline::manager) watch_diagnostics:
        Arc<Mutex<ProjectAssetWatchDiagnostics>>,
    pub(in crate::asset::pipeline::manager) transaction_watch_echoes:
        Arc<Mutex<TransactionWatchEchoes>>,
    pub(in crate::asset::pipeline::manager) watchers: Arc<Mutex<Vec<AssetWatcher>>>,
}

#[cfg(test)]
#[path = "tests/project_asset_manager.rs"]
mod tests;
