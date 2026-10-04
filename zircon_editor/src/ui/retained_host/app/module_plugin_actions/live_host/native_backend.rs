#[cfg(debug_assertions)]
use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use std::sync::Mutex;

use zircon_runtime::core::framework::channel::ChannelWakeCallback;
#[cfg(debug_assertions)]
use zircon_runtime::plugin::native::discovery::discover_native_plugins;
use zircon_runtime::plugin::native::host::NativePluginHostHandle;
use zircon_runtime::plugin::PluginModuleKind;

use crate::core::jobs::EditorJobSystem;
use crate::core::play::NativePluginArtifactAuthorityResolver;
use crate::core::plugin::project_native_plugin_directory;

#[cfg(debug_assertions)]
use super::development_watch::{DevelopmentPluginWatch, DevelopmentPluginWatchKey};
use super::types::{
    ModulePluginDevelopmentWatchPoll, ModulePluginLiveHostBackend, ModulePluginLiveHostCommand,
    ModulePluginLiveHostCompletion, ModulePluginLiveHostOutcome, ModulePluginLiveHostProject,
    ModulePluginLiveHostRequest,
};

pub(in crate::ui::retained_host::app) struct NativePluginDevelopmentLiveHostBackend {
    live_host: NativePluginHostHandle,
    authority_resolver: Option<NativePluginArtifactAuthorityResolver>,
    #[cfg(debug_assertions)]
    editor_jobs: EditorJobSystem,
    #[cfg(debug_assertions)]
    wake_host: ChannelWakeCallback,
    #[cfg(debug_assertions)]
    development_watches: Mutex<BTreeMap<DevelopmentPluginWatchKey, DevelopmentPluginWatch>>,
}

impl NativePluginDevelopmentLiveHostBackend {
    pub(in crate::ui::retained_host::app) fn new(
        live_host: NativePluginHostHandle,
        editor_jobs: EditorJobSystem,
        wake_host: ChannelWakeCallback,
    ) -> Self {
        #[cfg(not(debug_assertions))]
        let _ = (editor_jobs, wake_host);
        Self {
            live_host,
            authority_resolver: None,
            #[cfg(debug_assertions)]
            editor_jobs,
            #[cfg(debug_assertions)]
            wake_host,
            #[cfg(debug_assertions)]
            development_watches: Mutex::new(BTreeMap::new()),
        }
    }

    pub(in crate::ui::retained_host::app) fn with_authority_resolver(
        mut self,
        authority_resolver: NativePluginArtifactAuthorityResolver,
    ) -> Self {
        self.authority_resolver = Some(authority_resolver);
        self
    }

    #[cfg(debug_assertions)]
    fn ensure_development_watch(
        &self,
        project: &ModulePluginLiveHostProject,
        plugin_id: &str,
    ) -> Result<bool, String> {
        let native_plugin_root = project_native_plugin_directory(project.root());
        let artifact_path = development_artifact_path(&native_plugin_root, plugin_id)?;
        let key = DevelopmentPluginWatchKey::new(project.clone(), plugin_id, &artifact_path)?;
        let mut watches = self
            .development_watches
            .lock()
            .map_err(|_| "native plugin development watch registry is poisoned".to_string())?;
        if watches.contains_key(&key) {
            return Ok(false);
        }
        let watch = DevelopmentPluginWatch::start(
            &self.live_host,
            self.editor_jobs.clone(),
            self.wake_host.clone(),
            key.clone(),
            self.authority_resolver.clone(),
        )?;
        replace_development_watch(&mut watches, key, watch);
        Ok(true)
    }

    #[cfg(debug_assertions)]
    fn remove_development_watch(&self, plugin_id: &str) -> Result<(), String> {
        self.development_watches
            .lock()
            .map_err(|_| "native plugin development watch registry is poisoned".to_string())?
            .retain(|key, _| key.plugin_id() != plugin_id);
        Ok(())
    }
}

#[cfg(debug_assertions)]
fn development_artifact_path(
    native_plugin_root: &std::path::Path,
    plugin_id: &str,
) -> Result<std::path::PathBuf, String> {
    let report = discover_native_plugins(native_plugin_root);
    let candidates = report
        .discovered()
        .iter()
        .filter(|candidate| candidate.plugin_id == plugin_id)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [candidate] => Ok(candidate.library_path.clone()),
        [] => {
            let diagnostics = report.diagnostics().join("; ");
            let detail = if diagnostics.is_empty() {
                String::new()
            } else {
                format!(": {diagnostics}")
            };
            Err(format!(
                "native plugin `{plugin_id}` has no discovered development artifact{detail}"
            ))
        }
        _ => Err(format!(
            "native plugin `{plugin_id}` has {} discovered development artifacts",
            candidates.len()
        )),
    }
}

#[cfg(debug_assertions)]
fn replace_development_watch<T>(
    watches: &mut BTreeMap<DevelopmentPluginWatchKey, T>,
    key: DevelopmentPluginWatchKey,
    watch: T,
) {
    watches.retain(|existing, _| existing.plugin_id() != key.plugin_id());
    watches.insert(key, watch);
}

#[cfg(debug_assertions)]
fn append_development_watch_cleanup_diagnostic(
    diagnostics: &mut Vec<String>,
    cleanup: Result<(), String>,
) {
    if let Err(error) = cleanup {
        diagnostics.push(format!("native.development_watch.cleanup_failed: {error}"));
    }
}

impl ModulePluginLiveHostBackend for NativePluginDevelopmentLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        let outcome = match request.command {
            ModulePluginLiveHostCommand::Unload => {
                let outcome = self.live_host.unload_editor_plugin(request.plugin_id)?;
                #[cfg(debug_assertions)]
                let outcome = {
                    let mut outcome = outcome;
                    append_development_watch_cleanup_diagnostic(
                        &mut outcome.diagnostics,
                        self.remove_development_watch(request.plugin_id),
                    );
                    outcome
                };
                outcome
            }
            ModulePluginLiveHostCommand::HotReload => {
                let native_plugin_root = project_native_plugin_directory(request.project.root());
                let mut outcome = if let Some(resolver) = self.authority_resolver.as_ref() {
                    let authority = resolver(request.project.root())?;
                    self.live_host.hot_reload_editor_plugin_with_authority(
                        &native_plugin_root,
                        request.plugin_id,
                        &authority,
                    )?
                } else {
                    self.live_host
                        .hot_reload_editor_plugin(&native_plugin_root, request.plugin_id)?
                };
                #[cfg(debug_assertions)]
                match self.ensure_development_watch(request.project, request.plugin_id) {
                    Ok(true) => outcome.diagnostics.push(format!(
                        "native.development_watch.active: plugin `{}` will hot reload after native artifact changes",
                        request.plugin_id
                    )),
                    Ok(false) => {}
                    Err(error) => outcome.diagnostics.push(format!(
                        "native.development_watch.unavailable: {error}"
                    )),
                }
                outcome
            }
        };
        Ok(ModulePluginLiveHostOutcome {
            plugin_id: outcome.plugin_id,
            command: request.command,
            diagnostics: outcome.diagnostics,
        })
    }

    fn loaded_editor_plugin_ids(&self) -> Result<Vec<String>, String> {
        self.live_host.loaded_plugin_ids(PluginModuleKind::Editor)
    }

    fn poll_development_watches(
        &self,
        project: Option<&ModulePluginLiveHostProject>,
    ) -> ModulePluginDevelopmentWatchPoll {
        #[cfg(debug_assertions)]
        {
            let now = std::time::Instant::now();
            let mut watches = match self.development_watches.lock() {
                Ok(watches) => watches,
                Err(_) => {
                    let mut poll = ModulePluginDevelopmentWatchPoll::default();
                    poll.push_diagnostic(
                        "native plugin development watch registry is poisoned".to_string(),
                    );
                    return poll;
                }
            };
            // Dropping a retired watch cancels its pending job before further admission.
            watches.retain(|key, _| project == Some(key.project()));
            let mut aggregate = ModulePluginDevelopmentWatchPoll::default();
            for (key, watch) in watches.iter_mut() {
                let poll = watch.poll(now);
                if let Some(result) = poll.result {
                    aggregate.push_completion(ModulePluginLiveHostCompletion {
                        project: key.project().clone(),
                        plugin_id: key.plugin_id().to_string(),
                        result,
                    });
                }
                aggregate.include_deadline(poll.next_deadline);
            }
            return aggregate;
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = project;
            ModulePluginDevelopmentWatchPoll::default()
        }
    }
}

#[cfg(all(test, debug_assertions))]
#[path = "tests/native_backend.rs"]
mod tests;
