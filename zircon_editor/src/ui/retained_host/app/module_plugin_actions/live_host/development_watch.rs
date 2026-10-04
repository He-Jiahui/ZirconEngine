#![cfg(debug_assertions)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use zircon_runtime::core::framework::channel::ChannelWakeCallback;
use zircon_runtime::plugin::native::host::{NativePluginHostHandle, NativePluginHostWeakHandle};

use super::types::ModulePluginLiveHostProject;
use crate::core::jobs::{
    CancellationToken, EditorJob, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobPriority, JobTicket, MutexGroup,
};
use crate::core::play::NativePluginArtifactAuthorityResolver;
use crate::core::plugin::project_native_plugin_directory;

const RELOAD_DEBOUNCE: Duration = Duration::from_millis(350);
const RELOAD_JOB_MAX_PENDING_AGE: Duration = Duration::from_secs(30);
const RELOAD_JOB_ESTIMATED_BYTES: usize = 4 * 1024;
const NATIVE_PLUGIN_RELOAD_MUTEX_GROUP: &str = "native_plugin_reload";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct DevelopmentPluginWatchKey {
    project: ModulePluginLiveHostProject,
    plugin_id: String,
    artifact_path: PathBuf,
}

impl DevelopmentPluginWatchKey {
    pub(super) fn new(
        project: ModulePluginLiveHostProject,
        plugin_id: &str,
        artifact_path: &Path,
    ) -> Result<Self, String> {
        let project_root = std::fs::canonicalize(project.root()).map_err(|error| {
            format!(
                "cannot watch native plugin `{plugin_id}` under {}: {error}",
                project.root().display()
            )
        })?;
        let artifact_path = std::fs::canonicalize(artifact_path).map_err(|error| {
            format!(
                "cannot watch native plugin `{plugin_id}` artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        if !artifact_path.starts_with(&project_root) {
            return Err(format!(
                "native plugin `{plugin_id}` artifact {} is outside project root {}",
                artifact_path.display(),
                project_root.display()
            ));
        }
        Ok(Self {
            project: project.with_root(project_root),
            plugin_id: plugin_id.to_string(),
            artifact_path,
        })
    }

    pub(super) fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub(super) fn project(&self) -> &ModulePluginLiveHostProject {
        &self.project
    }
}

pub(super) struct DevelopmentPluginWatch {
    watcher: Option<RecommendedWatcher>,
    editor_jobs: EditorJobSystem,
    live_host: NativePluginHostWeakHandle,
    authority_resolver: Option<NativePluginArtifactAuthorityResolver>,
    key: DevelopmentPluginWatchKey,
    schedule: Arc<Mutex<DevelopmentPluginWatchSchedule>>,
    ticket: Option<JobTicket<String>>,
    cancel: Option<CancellationToken>,
}

#[derive(Debug, Default)]
pub(super) struct DevelopmentPluginWatchPoll {
    pub(super) result: Option<Result<String, String>>,
    pub(super) next_deadline: Option<Instant>,
}

#[derive(Debug, Default)]
struct DevelopmentPluginWatchSchedule {
    changed_at: Option<Instant>,
}

impl DevelopmentPluginWatchSchedule {
    fn record_change_at(&mut self, changed_at: Instant) {
        self.changed_at = Some(
            self.changed_at
                .map_or(changed_at, |existing| existing.max(changed_at)),
        );
    }

    fn take_due_at(&mut self, now: Instant) -> Option<Instant> {
        let changed_at = self.changed_at?;
        if now.saturating_duration_since(changed_at) < RELOAD_DEBOUNCE {
            return None;
        }
        self.changed_at.take()
    }
}

impl DevelopmentPluginWatch {
    pub(super) fn start(
        live_host: &NativePluginHostHandle,
        editor_jobs: EditorJobSystem,
        wake_host: ChannelWakeCallback,
        key: DevelopmentPluginWatchKey,
        authority_resolver: Option<NativePluginArtifactAuthorityResolver>,
    ) -> Result<Self, String> {
        let schedule = Arc::new(Mutex::new(DevelopmentPluginWatchSchedule::default()));
        let callback_schedule = Arc::clone(&schedule);
        let callback_plugin_id = key.plugin_id.clone();
        let callback_artifact_path = key.artifact_path.clone();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            let should_reload = event
                .as_ref()
                .is_ok_and(|event| {
                    development_event_requests_reload(event, &callback_artifact_path)
                });
            if !should_reload {
                if let Err(error) = event {
                    eprintln!(
                        "[zircon_editor] native development watch for `{callback_plugin_id}` failed: {error}"
                    );
                }
                return;
            }
            callback_schedule
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .record_change_at(Instant::now());
            wake_host();
        })
        .map_err(|error| format!("native development watcher creation failed: {error}"))?;
        let artifact_parent = key.artifact_path.parent().ok_or_else(|| {
            format!(
                "native plugin `{}` artifact has no parent directory: {}",
                key.plugin_id,
                key.artifact_path.display()
            )
        })?;
        watcher
            .watch(artifact_parent, RecursiveMode::NonRecursive)
            .map_err(|error| {
                format!(
                    "native development watcher could not watch {}: {error}",
                    artifact_parent.display()
                )
            })?;

        Ok(Self {
            watcher: Some(watcher),
            editor_jobs,
            live_host: live_host.downgrade(),
            authority_resolver,
            key,
            schedule,
            ticket: None,
            cancel: None,
        })
    }

    pub(super) fn poll(&mut self, now: Instant) -> DevelopmentPluginWatchPoll {
        let mut poll = DevelopmentPluginWatchPoll::default();
        if let Some(result) = self.ticket.as_ref().and_then(JobTicket::try_take) {
            self.ticket.take();
            self.cancel.take();
            poll.result = match result {
                Ok(diagnostic) => Some(Ok(diagnostic)),
                Err(JobError::Cancelled) => None,
                Err(error) => Some(Err(format!(
                    "native plugin `{}` development hot reload failed: {error}",
                    self.key.plugin_id
                ))),
            };
        }
        if self.ticket.is_some() {
            return poll;
        }

        let changed_at = {
            let mut schedule = self
                .schedule
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(changed_at) = schedule.changed_at else {
                return poll;
            };
            let due_at = changed_at + RELOAD_DEBOUNCE;
            if due_at > now {
                poll.next_deadline = Some(due_at);
                return poll;
            }
            schedule
                .take_due_at(now)
                .expect("a due development watch timestamp must remain present")
        };
        let cancel = CancellationToken::default();
        let spec = EditorJobSpec::new(
            format!("Hot reload native plugin {}", self.key.plugin_id),
            JobCategory::Compile,
        )
        .with_priority(JobPriority::Background)
        .with_mutex_group(
            MutexGroup::parse(NATIVE_PLUGIN_RELOAD_MUTEX_GROUP)
                .expect("the built-in native plugin reload mutex group must be valid"),
        )
        .with_cancel(cancel.clone())
        .with_estimated_bytes(RELOAD_JOB_ESTIMATED_BYTES)
        .with_max_pending_age(RELOAD_JOB_MAX_PENDING_AGE);
        let job = DevelopmentPluginReloadJob {
            live_host: self.live_host.clone(),
            authority_resolver: self.authority_resolver.clone(),
            key: self.key.clone(),
        };
        match self.editor_jobs.submit(spec, job) {
            Ok(ticket) => {
                self.ticket = Some(ticket);
                self.cancel = Some(cancel);
            }
            Err(error) => {
                self.schedule
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .record_change_at(now.max(changed_at));
                poll.next_deadline = Some(now + RELOAD_DEBOUNCE);
                poll.result = Some(Err(format!(
                    "native plugin `{}` development hot reload admission failed: {error}",
                    self.key.plugin_id
                )));
            }
        }
        poll
    }
}

impl Drop for DevelopmentPluginWatch {
    fn drop(&mut self) {
        self.watcher.take();
        if let Some(cancel) = self.cancel.take() {
            cancel.cancel();
        }
        if let Some(ticket) = self.ticket.take() {
            self.editor_jobs.cancel(ticket.id());
        }
    }
}

struct DevelopmentPluginReloadJob {
    live_host: NativePluginHostWeakHandle,
    authority_resolver: Option<NativePluginArtifactAuthorityResolver>,
    key: DevelopmentPluginWatchKey,
}

impl EditorJob for DevelopmentPluginReloadJob {
    type Output = String;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let Some(_live_host) = self.live_host.upgrade() else {
            return Err(JobError::Cancelled);
        };
        context.check_cancelled()?;
        let native_plugin_root = project_native_plugin_directory(self.key.project.root());
        if let Some(resolver) = self.authority_resolver.as_ref() {
            let _authority = resolver(self.key.project.root())
                .map_err(|error| JobError::failed(std::io::Error::other(error)))?;
        }
        context.check_cancelled()?;
        std::fs::metadata(&self.key.artifact_path)
            .map_err(|error| JobError::failed(std::io::Error::other(error)))?;
        context.check_cancelled()?;
        Ok(format!(
            "native plugin `{}` artifact change is ready under {}",
            self.key.plugin_id,
            native_plugin_root.display()
        ))
    }
}

fn development_event_requests_reload(event: &Event, artifact_path: &Path) -> bool {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return false;
    }
    event.paths.iter().any(|path| path == artifact_path)
}

#[cfg(test)]
#[path = "tests/development_watch.rs"]
mod tests;
