use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{
    atomic::AtomicBool,
    mpsc::{self, Receiver, Sender},
    Arc,
};

use super::{worker::DesktopExportJobMessage, DesktopExportProgressSnapshot};
use zircon_runtime::asset::project::ProjectManifest;

#[derive(Debug)]
pub(super) struct DesktopExportQueuedJob {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) project_root: PathBuf,
    pub(super) manifest: ProjectManifest,
    pub(super) output_root: PathBuf,
    pub(super) cancel_requested: Arc<AtomicBool>,
}

#[derive(Debug)]
pub(super) struct DesktopExportActiveJob {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) cancel_requested: Arc<AtomicBool>,
    pub(super) progress: Option<DesktopExportProgressSnapshot>,
}

pub(in crate::ui::retained_host::app) struct DesktopExportJobQueue {
    pub(super) next_id: u64,
    pub(super) pending: VecDeque<DesktopExportQueuedJob>,
    pub(super) active: Option<DesktopExportActiveJob>,
    pub(super) sender: Sender<DesktopExportJobMessage>,
    pub(super) receiver: Receiver<DesktopExportJobMessage>,
}

impl Default for DesktopExportJobQueue {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            next_id: 1,
            pending: VecDeque::new(),
            active: None,
            sender,
            receiver,
        }
    }
}
