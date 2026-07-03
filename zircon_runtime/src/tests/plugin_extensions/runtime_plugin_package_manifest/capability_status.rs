use super::*;

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_capabilities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_capability_status(CapabilityStatusManifest::new(
                "Runtime.Plugin.Weather",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.plugin.storm",
                CapabilityStatus::Stub,
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status capability `Runtime.Plugin.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status `runtime.plugin.storm`")
            && diagnostic.contains("same package")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_capability_status_targets() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_capability_status(
                CapabilityStatusManifest::new("runtime.plugin.weather", CapabilityStatus::Partial)
                    .with_target_modes([
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::EditorHost,
                    ]),
            )
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.plugin.weather",
                CapabilityStatus::Stub,
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status `runtime.plugin.weather`")
            && diagnostic.contains("must be unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("target mode ClientRuntime") && diagnostic.contains("unique")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("target mode EditorHost")
            && diagnostic.contains("supported_targets")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_bevy_metadata() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_capability_status(
                CapabilityStatusManifest::new("runtime.plugin.weather", CapabilityStatus::Partial)
                    .with_bevy_reference("../bevy/crates/bevy_app/src/plugin.rs")
                    .with_bevy_reference("../bevy/crates/bevy_app/src/plugin.rs")
                    .with_note(" partial parity "),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("bevy reference `../bevy/crates/bevy_app/src/plugin.rs`")
            && diagnostic.contains("dev/bevy")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("bevy reference `../bevy/crates/bevy_app/src/plugin.rs`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status note ` partial parity `")
            && diagnostic.contains("non-empty and trimmed")
    }));
}
