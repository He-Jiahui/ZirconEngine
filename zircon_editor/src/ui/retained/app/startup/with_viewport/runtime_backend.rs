use super::super::super::super::*;
use crate::core::play::{NativePluginAdmissionResolver, NativePluginArtifactAuthorityResolver};

pub(super) struct StartupRuntimeBackend {
    pub(super) runtime: Arc<EditorHostEventController>,
    pub(super) native_plugin_host: zircon_runtime::plugin::native::host::NativePluginHostHandle,
    pub(super) native_plugin_authority_resolver: NativePluginArtifactAuthorityResolver,
}

pub(super) fn create_startup_runtime_backend(
    runtime: EditorHostEventController,
    native_plugin_authority_resolver: NativePluginArtifactAuthorityResolver,
    native_plugin_admission_resolver: NativePluginAdmissionResolver,
) -> StartupRuntimeBackend {
    let native_plugin_host = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_native_plugin_host");
        zircon_runtime::plugin::native::host::NativePluginHostHandle::default()
    };
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_plugin_bridge_activation");
        runtime.set_plugin_bridge_activation(std::sync::Arc::new(
            NativePluginBridgeActivation::new(native_plugin_host.clone())
                .with_authority_resolver(native_plugin_authority_resolver.clone())
                .with_admission_resolver(native_plugin_admission_resolver),
        ));
    }

    StartupRuntimeBackend {
        runtime: Arc::new(runtime),
        native_plugin_host,
        native_plugin_authority_resolver,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::play::{PlayKind, PlayStartRequest};
    use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn native_play_startup_resolves_package_admission_for_the_selected_project() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let harness = EventRuntimeHarness::new("native-play-admission");
        let controller = &harness.runtime;
        let manager = controller.editor_manager_handle();
        let state = EditorState::with_default_selection_with_context(
            zircon_runtime::scene::DefaultLevelManager::default().create_default_level(),
            UVec2::new(320, 180),
            controller.context().clone(),
        );
        let runtime = EditorHostEventController::new(state, manager).unwrap();
        let called = Arc::new(AtomicBool::new(false));
        let observation = called.clone();
        let project = std::path::Path::new("/tmp/ze-editor-ui-20261022/interact/plugin-admission");
        let backend = create_startup_runtime_backend(
            runtime,
            Arc::new(|_| {
                Ok(zircon_runtime::plugin::native::NativePluginArtifactAuthority::deny_all())
            }),
            Arc::new(move |root, target| {
                assert_eq!(root, project);
                observation.store(true, Ordering::SeqCst);
                Ok(zircon_runtime::plugin::package_service::resolve_project_native_plugin_admission(
                    root, None, target,
                    &zircon_runtime::core::framework::project::ProjectPluginManifest::default(),
                ))
            }),
        );
        backend
            .runtime
            .play_sessions()
            .request_play(PlayStartRequest::immediate(PlayKind::Play, Some(project)))
            .unwrap();
        assert!(called.load(Ordering::SeqCst));
        backend.runtime.play_sessions().request_stop().unwrap();
    }
}
