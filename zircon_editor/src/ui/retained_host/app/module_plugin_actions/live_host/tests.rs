use super::*;
use std::sync::Arc;

use zircon_runtime::plugin::native::NativePluginLiveHost;

struct UnavailableModulePluginLiveHostBackend;

impl ModulePluginLiveHostBackend for UnavailableModulePluginLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        Err(unavailable_live_plugin_backend_message(
            request.plugin_id,
            request.command.label(),
        ))
    }
}

fn unavailable_live_plugin_backend_message(plugin_id: &str, action: &str) -> String {
    format!(
        "plugin {plugin_id} {action} is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    )
}

#[test]
fn live_backend_actions_report_unavailable_backend_until_runtime_host_is_connected() {
    let project_root = std::path::Path::new("project");
    assert_eq!(
        dispatch_live_plugin_backend_action(
            &UnavailableModulePluginLiveHostBackend,
            "physics",
            ModulePluginLiveHostCommand::Unload,
            project_root,
        )
        .unwrap_err(),
        "plugin physics unload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    );
    assert_eq!(
        dispatch_live_plugin_backend_action(
            &UnavailableModulePluginLiveHostBackend,
            "physics",
            ModulePluginLiveHostCommand::HotReload,
            project_root,
        )
        .unwrap_err(),
        "plugin physics hot reload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    );
    assert_eq!(
        dispatch_live_plugin_backend_action(
            &UnavailableModulePluginLiveHostBackend,
            "   ",
            ModulePluginLiveHostCommand::Unload,
            project_root,
        )
        .unwrap_err(),
        "plugin id is empty"
    );
    assert_eq!(
        unavailable_live_plugin_backend_message("physics", "unload"),
        "plugin physics unload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    );
    assert_eq!(
        unavailable_live_plugin_backend_message("physics", "hot reload"),
        "plugin physics hot reload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    );
}

#[test]
fn live_backend_dispatch_routes_unload_and_hot_reload_commands() {
    #[derive(Clone, Copy)]
    struct RecordingLiveBackend;

    impl ModulePluginLiveHostBackend for RecordingLiveBackend {
        fn execute(
            &self,
            request: ModulePluginLiveHostRequest<'_>,
        ) -> Result<ModulePluginLiveHostOutcome, String> {
            Ok(ModulePluginLiveHostOutcome {
                plugin_id: request.plugin_id.to_string(),
                command: request.command,
                diagnostics: Vec::new(),
            })
        }
    }

    let project_root = std::path::Path::new("project");
    let unload = dispatch_live_plugin_backend_action(
        &RecordingLiveBackend,
        "physics",
        ModulePluginLiveHostCommand::Unload,
        project_root,
    )
    .expect("unload command should route into live backend");
    assert_eq!(
        unload,
        ModulePluginLiveHostOutcome {
            plugin_id: "physics".to_string(),
            command: ModulePluginLiveHostCommand::Unload,
            diagnostics: Vec::new(),
        }
    );
    assert_eq!(
        live_plugin_backend_success_message(&unload),
        "Plugin physics unloaded"
    );

    let hot_reload = dispatch_live_plugin_backend_action(
        &RecordingLiveBackend,
        "physics",
        ModulePluginLiveHostCommand::HotReload,
        project_root,
    )
    .expect("hot reload command should route into live backend");
    assert_eq!(
        live_plugin_backend_success_message(&ModulePluginLiveHostOutcome {
            diagnostics: vec!["library handle was replaced".to_string()],
            ..hot_reload
        }),
        "Plugin physics hot reloaded: library handle was replaced"
    );
}

#[test]
fn runtime_native_live_backend_reports_missing_editor_package_on_hot_reload() {
    let project_root = std::env::temp_dir().join(format!(
        "zircon-missing-native-live-backend-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let backend =
        NativePluginDevelopmentLiveHostBackend::new(Arc::new(NativePluginLiveHost::default()));
    let error = dispatch_live_plugin_backend_action(
        &backend,
        "physics",
        ModulePluginLiveHostCommand::HotReload,
        &project_root,
    )
    .unwrap_err();
    assert!(error.contains("plugin physics hot reload did not load an editor native package"));
    assert!(error.contains("native plugin root does not exist"));
}
