#![cfg(debug_assertions)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use zircon_runtime::core::framework::channel::ChannelWakeCallback;
use zircon_runtime::plugin::native::host::{NativePluginHostHandle, NativePluginHostWeakHandle};

use crate::core::jobs::{
    CancellationToken, EditorJob, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobPriority, JobTicket, MutexGroup,
};

const RELOAD_DEBOUNCE: Duration = Duration::from_millis(350);
const RELOAD_JOB_MAX_PENDING_AGE: Duration = Duration::from_secs(30);
const RELOAD_JOB_ESTIMATED_BYTES: usize = 4 * 1024;
const NATIVE_PLUGIN_RELOAD_MUTEX_GROUP: &str = "native_plugin_reload";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct DevelopmentPluginWatchKey {
    project_root: PathBuf,
    plugin_id: String,
    artifact_path: PathBuf,
}

impl DevelopmentPluginWatchKey {
    pub(super) fn new(
        project_root: &Path,
        plugin_id: &str,
        artifact_path: &Path,
    ) -> Result<Self, String> {
        let project_root = std::fs::canonicalize(project_root).map_err(|error| {
            format!(
                "cannot watch native plugin `{plugin_id}` under {}: {error}",
                project_root.display()
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
            project_root,
            plugin_id: plugin_id.to_string(),
            artifact_path,
        })
    }

    pub(super) fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

pub(super) struct DevelopmentPluginWatch {
    watcher: Option<RecommendedWatcher>,
    editor_jobs: EditorJobSystem,
    live_host: NativePluginHostWeakHandle,
    key: DevelopmentPluginWatchKey,
    schedule: Arc<Mutex<DevelopmentPluginWatchSchedule>>,
    ticket: Option<JobTicket<String>>,
    cancel: Option<CancellationToken>,
}

#[derive(Debug, Default)]
pub(super) struct DevelopmentPluginWatchPoll {
    pub(super) diagnostic: Option<String>,
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
            poll.diagnostic = match result {
                Ok(diagnostic) => Some(diagnostic),
                Err(JobError::Cancelled) => None,
                Err(error) => Some(format!(
                    "native plugin `{}` development hot reload failed: {error}",
                    self.key.plugin_id
                )),
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
                poll.diagnostic = Some(format!(
                    "native plugin `{}` development hot reload admission failed: {error}",
                    self.key.plugin_id
                ));
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
    key: DevelopmentPluginWatchKey,
}

impl EditorJob for DevelopmentPluginReloadJob {
    type Output = String;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let Some(live_host) = self.live_host.upgrade() else {
            return Err(JobError::Cancelled);
        };
        context.check_cancelled()?;
        let outcome = live_host
            .hot_reload_editor_plugin(&self.key.project_root, &self.key.plugin_id)
            .map_err(|error| JobError::failed(std::io::Error::other(error)))?;
        let diagnostics = if outcome.diagnostics.is_empty() {
            "no diagnostics".to_string()
        } else {
            outcome.diagnostics.join("; ")
        };
        Ok(format!(
            "native plugin `{}` hot reloaded after artifact change: {diagnostics}",
            self.key.plugin_id
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
mod tests {
    use notify::event::ModifyKind;

    use super::*;

    #[test]
    fn development_watch_uses_the_editor_job_owner_without_a_private_worker() {
        let source = include_str!("development_watch.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("development watch production source");

        assert!(production.contains("EditorJobSystem"));
        assert!(production.contains("JobTicket<String>"));
        assert!(production.contains("wake_host();"));
        for retired_owner in [
            "std::thread",
            "JoinHandle",
            "sync_channel",
            "RecvTimeoutError",
            ".join()",
        ] {
            assert!(
                !production.contains(retired_owner),
                "retired private worker owner remains: {retired_owner}"
            );
        }
    }

    #[test]
    fn development_watch_schedule_coalesces_to_the_latest_change_time() {
        let start = Instant::now();
        let mut schedule = DevelopmentPluginWatchSchedule::default();
        schedule.record_change_at(start);
        schedule.record_change_at(start + Duration::from_millis(100));

        assert_eq!(
            schedule.take_due_at(start + Duration::from_millis(449)),
            None
        );
        assert_eq!(
            schedule.take_due_at(start + Duration::from_millis(450)),
            Some(start + Duration::from_millis(100))
        );
        assert_eq!(schedule.take_due_at(start + Duration::from_secs(1)), None);
    }

    #[test]
    fn development_watch_filters_for_the_exact_loaded_artifact() {
        let artifact_path = PathBuf::from("project/native/plugin.dll");
        assert!(development_event_requests_reload(
            &Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![artifact_path.clone()],
                attrs: Default::default(),
            },
            &artifact_path,
        ));

        for path in [
            PathBuf::from("project/other/native/other.dll"),
            PathBuf::from("project/plugin.toml"),
            PathBuf::from("project/native/plugin.pdb"),
        ] {
            assert!(!development_event_requests_reload(
                &Event {
                    kind: EventKind::Modify(ModifyKind::Any),
                    paths: vec![path],
                    attrs: Default::default(),
                },
                &artifact_path,
            ));
        }
        assert!(!development_event_requests_reload(
            &Event {
                kind: EventKind::Access(notify::event::AccessKind::Any),
                paths: vec![artifact_path.clone()],
                attrs: Default::default(),
            },
            &artifact_path,
        ));
    }
}
