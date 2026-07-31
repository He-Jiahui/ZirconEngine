use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use super::super::DesktopExportExecutionSummary;
use super::{
    DesktopExportProgressSnapshot,
    worker::{DesktopExportJobProgress, DesktopExportJobResult},
};
use crate::core::jobs::{CancellationToken, EditorJobSystem, JobTicket};
use zircon_runtime::asset::project::ProjectManifest;

#[derive(Debug)]
pub(super) struct DesktopExportQueuedJob {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) project_root: PathBuf,
    pub(super) manifest: ProjectManifest,
    pub(super) output_root: PathBuf,
    pub(super) cancel: CancellationToken,
}

pub(super) struct DesktopExportActiveJob {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) cancel: CancellationToken,
    pub(super) progress: Option<DesktopExportProgressSnapshot>,
    pub(super) ticket: JobTicket<DesktopExportJobResult>,
}

pub(in crate::ui::retained_host::app) struct DesktopExportJobQueue {
    pub(super) jobs: EditorJobSystem,
    pub(super) next_id: u64,
    pub(super) pending: VecDeque<DesktopExportQueuedJob>,
    pub(super) active: Option<DesktopExportActiveJob>,
    pub(super) completed: VecDeque<DesktopExportExecutionSummary>,
    pub(super) progress_sender: Sender<DesktopExportJobProgress>,
    pub(super) progress_receiver: Receiver<DesktopExportJobProgress>,
}

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn new(jobs: EditorJobSystem) -> Self {
        let (progress_sender, progress_receiver) = mpsc::channel();
        Self {
            jobs,
            next_id: 1,
            pending: VecDeque::new(),
            active: None,
            completed: VecDeque::new(),
            progress_sender,
            progress_receiver,
        }
    }
}
