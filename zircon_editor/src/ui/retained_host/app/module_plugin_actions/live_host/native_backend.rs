#[cfg(debug_assertions)]
use std::collections::BTreeMap;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::sync::Mutex;

use zircon_runtime::plugin::native::NativePluginLiveHost;
#[cfg(debug_assertions)]
use zircon_runtime::plugin::native::NativePluginLoader;

#[cfg(debug_assertions)]
use super::development_watch::{DevelopmentPluginWatch, DevelopmentPluginWatchKey};
use super::types::{
    ModulePluginLiveHostBackend, ModulePluginLiveHostCommand, ModulePluginLiveHostOutcome,
    ModulePluginLiveHostRequest,
};

pub(in crate::ui::retained_host::app) struct NativePluginDevelopmentLiveHostBackend {
    live_host: Arc<NativePluginLiveHost>,
    #[cfg(debug_assertions)]
    development_watches: Mutex<BTreeMap<DevelopmentPluginWatchKey, DevelopmentPluginWatch>>,
}

impl NativePluginDevelopmentLiveHostBackend {
    pub(in crate::ui::retained_host::app) fn new(live_host: Arc<NativePluginLiveHost>) -> Self {
        Self {
            live_host,
            #[cfg(debug_assertions)]
            development_watches: Mutex::new(BTreeMap::new()),
        }
    }

    #[cfg(debug_assertions)]
    fn ensure_development_watch(
        &self,
        project_root: &std::path::Path,
        plugin_id: &str,
    ) -> Result<bool, String> {
        let artifact_path = development_artifact_path(project_root, plugin_id)?;
        let key = DevelopmentPluginWatchKey::new(project_root, plugin_id, &artifact_path)?;
        let mut watches = self
            .development_watches
            .lock()
            .map_err(|_| "native plugin development watch registry is poisoned".to_string())?;
        if watches.contains_key(&key) {
            return Ok(false);
        }
        let watch = DevelopmentPluginWatch::start(&self.live_host, key.clone())?;
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
    project_root: &std::path::Path,
    plugin_id: &str,
) -> Result<std::path::PathBuf, String> {
    let report = NativePluginLoader.discover(project_root);
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
                let mut outcome = self
                    .live_host
                    .hot_reload_editor_plugin(request.project_root, request.plugin_id)?;
                #[cfg(debug_assertions)]
                match self.ensure_development_watch(request.project_root, request.plugin_id) {
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
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;

    #[test]
    fn development_watch_cleanup_failure_is_reported_without_replacing_the_host_outcome() {
        let mut diagnostics = vec!["native plugin unloaded".to_string()];

        append_development_watch_cleanup_diagnostic(
            &mut diagnostics,
            Err("watch registry is poisoned".to_string()),
        );

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0], "native plugin unloaded");
        assert!(diagnostics[1].contains("native.development_watch.cleanup_failed"));
    }

    #[test]
    fn development_watch_replaces_an_old_project_root_for_the_same_plugin() {
        let base = std::env::temp_dir().join(format!(
            "zircon-editor-development-watch-registry-{}",
            std::process::id()
        ));
        let first_root = base.join("first");
        let second_root = base.join("second");
        std::fs::create_dir_all(&first_root).unwrap();
        std::fs::create_dir_all(&second_root).unwrap();
        let first_artifact = first_root.join("demo.dll");
        let second_artifact = second_root.join("demo.dll");
        let other_artifact = first_root.join("other.dll");
        std::fs::write(&first_artifact, []).unwrap();
        std::fs::write(&second_artifact, []).unwrap();
        std::fs::write(&other_artifact, []).unwrap();
        let first = DevelopmentPluginWatchKey::new(&first_root, "demo", &first_artifact).unwrap();
        let second =
            DevelopmentPluginWatchKey::new(&second_root, "demo", &second_artifact).unwrap();
        let other = DevelopmentPluginWatchKey::new(&first_root, "other", &other_artifact).unwrap();
        let mut watches = BTreeMap::from([(first.clone(), 1), (other.clone(), 2)]);

        replace_development_watch(&mut watches, second.clone(), 3);

        assert_eq!(watches.len(), 2);
        assert!(!watches.contains_key(&first));
        assert_eq!(watches.get(&second), Some(&3));
        assert_eq!(watches.get(&other), Some(&2));
        std::fs::remove_dir_all(base).unwrap();
    }
}
