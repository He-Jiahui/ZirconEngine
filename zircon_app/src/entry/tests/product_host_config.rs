use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, ProjectPluginManifest, ProjectPluginSelection,
    RuntimeProfileId,
};
use zircon_runtime::core::framework::render::{RenderProductProfile, RenderProfileBundle};
use zircon_runtime::core::framework::window::WindowDescriptor;
use zircon_runtime::platform::PlatformTarget;

use super::super::{
    EntryConfig, EntryProfile, EntryRunner, ProductArtifactDeliveryStatus, ProductArtifactKind,
    ProductCapabilityRequirement, ProductConfigSource, ProductConfigSourceSet, ProductEntryKind,
    ProductHostConfigError, ProductPlatformClass, ProductRoleRequest, ProductRunnerKind,
    ProductRuntimeLinkage, ProductShutdownPolicy,
};

#[test]
fn export_profile_runtime_identity_survives_product_host_resolution() {
    let export_profile = ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    );
    let resolved = EntryConfig::new(EntryProfile::Runtime)
        .with_target_mode(RuntimeTargetMode::ClientRuntime)
        .with_project_plugins(ProjectPluginManifest::default())
        .with_export_profile(export_profile.clone())
        .resolve()
        .expect("coherent export intent should resolve once");

    assert_eq!(resolved.role(), ProductRoleRequest::DesktopClient);
    assert_eq!(resolved.profile(), EntryProfile::Runtime);
    assert_eq!(resolved.runtime_profile(), Some(RuntimeProfileId::Client2d));
    assert_eq!(resolved.target_mode(), RuntimeTargetMode::ClientRuntime);
    assert_eq!(resolved.export_profile(), Some(&export_profile));
    assert_eq!(
        resolved.provenance().runtime_profile(),
        ProductConfigSource::ExportProfile
    );
    assert_eq!(
        resolved.provenance().target_mode(),
        ProductConfigSource::ExportProfile
    );
}

#[test]
fn runtime_profile_conflicting_with_product_role_is_rejected() {
    let error = EntryConfig::new(EntryProfile::Runtime)
        .with_runtime_profile(RuntimeProfileId::Server)
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::RuntimeProfileRoleConflict {
            role: ProductRoleRequest::DesktopClient,
            runtime_profile: RuntimeProfileId::Server,
        }
    );
}

#[test]
fn explicit_target_conflicting_with_product_role_is_rejected() {
    let error = EntryConfig::new(EntryProfile::Runtime)
        .with_target_mode(RuntimeTargetMode::ServerRuntime)
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::TargetModeConflict {
            source: ProductConfigSource::EntryRequest,
            role: ProductRoleRequest::DesktopClient,
            expected: RuntimeTargetMode::ClientRuntime,
            actual: RuntimeTargetMode::ServerRuntime,
        }
    );
}

#[test]
fn export_profile_requires_an_explicit_runtime_profile() {
    let mut export_profile = ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    );
    export_profile.runtime_profile_id = None;

    let error = EntryConfig::new(EntryProfile::Runtime)
        .with_export_profile(export_profile)
        .resolve()
        .unwrap_err();

    assert_eq!(error, ProductHostConfigError::ExportRuntimeProfileMissing);
}

#[test]
fn unsupported_product_role_fails_before_runtime_composition() {
    let error = EntryConfig::for_product_role(ProductRoleRequest::Commandlet)
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::UnsupportedProductRole(ProductRoleRequest::Commandlet)
    );
}

#[test]
fn server_role_rejects_a_rendering_product_request() {
    let error = EntryConfig::new(EntryProfile::Headless)
        .with_render_profile(RenderProfileBundle::default_render())
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::RenderCapabilityForbidden(ProductRoleRequest::Server)
    );
}

#[test]
fn server_role_rejects_primary_window_ownership() {
    let error = EntryConfig::new(EntryProfile::Headless)
        .with_window_descriptor(WindowDescriptor::default())
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::WindowCapabilityForbidden(ProductRoleRequest::Server)
    );
}

#[test]
fn editor_role_requires_render_and_primary_window_capabilities() {
    let render_error = EntryConfig::new(EntryProfile::Editor)
        .with_render_profile(RenderProfileBundle::headless())
        .resolve()
        .unwrap_err();
    assert_eq!(
        render_error,
        ProductHostConfigError::RenderCapabilityRequired(ProductRoleRequest::EditorHost)
    );

    let window_error = EntryConfig::new(EntryProfile::Editor)
        .with_window_descriptor(WindowDescriptor::default().without_primary_window())
        .resolve()
        .unwrap_err();
    assert_eq!(
        window_error,
        ProductHostConfigError::WindowCapabilityRequired(ProductRoleRequest::EditorHost)
    );
}

#[test]
fn minimal_desktop_profile_can_omit_window_input_and_render_hosts() {
    let resolved = EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal)
        .with_render_profile(RenderProfileBundle::headless())
        .resolve()
        .expect("minimal runtime profile should remain a valid desktop-client utility mode");

    assert_eq!(resolved.role(), ProductRoleRequest::DesktopClient);
    assert_eq!(resolved.target_mode(), RuntimeTargetMode::ClientRuntime);
    assert_eq!(
        resolved.render_profile().profile(),
        RenderProductProfile::Headless
    );
    assert!(resolved.window_descriptor().primary_window.is_none());
}

#[test]
fn server_export_preserves_native_os_target_while_using_headless_execution() {
    for (name, export_target, platform_target) in [
        (
            "windows-server",
            ExportTargetPlatform::Windows,
            PlatformTarget::Windows,
        ),
        (
            "linux-server",
            ExportTargetPlatform::Linux,
            PlatformTarget::Linux,
        ),
    ] {
        let resolved = EntryConfig::for_product_role(ProductRoleRequest::Server)
            .with_export_profile(ExportProfile::new(
                name,
                RuntimeTargetMode::ServerRuntime,
                export_target,
                RuntimeProfileId::Server,
            ))
            .resolve()
            .expect(
                "native server target should use server execution without losing the target OS",
            );

        assert_eq!(resolved.platform_target(), platform_target);
        assert_eq!(resolved.target_mode(), RuntimeTargetMode::ServerRuntime);
        assert_eq!(
            resolved.render_profile().profile(),
            RenderProductProfile::Headless
        );
        assert!(resolved.window_descriptor().primary_window.is_none());
    }
}

#[test]
fn explicit_project_plugins_overlay_runtime_profile_defaults() {
    let resolved = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d)
        .with_project_plugins(ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                zircon_runtime::builtin::RuntimePluginId::VirtualGeometry,
                true,
                false,
            )],
        })
        .resolve()
        .expect("project plugin selection should overlay the runtime profile baseline");
    let manifest = resolved
        .project_plugin_manifest()
        .expect("profile and project selections should resolve to one manifest");

    for expected in [
        zircon_runtime::builtin::RuntimePluginId::Sound,
        zircon_runtime::builtin::RuntimePluginId::Rendering,
        zircon_runtime::builtin::RuntimePluginId::Texture,
        zircon_runtime::builtin::RuntimePluginId::VirtualGeometry,
    ] {
        assert!(manifest
            .selections
            .iter()
            .any(|selection| selection.id == expected.key()));
    }
    assert_eq!(
        resolved.provenance().project_plugins(),
        ProductConfigSourceSet::single(ProductConfigSource::RuntimeProfile)
            .with(ProductConfigSource::EntryRequest)
    );
}

#[test]
fn required_and_optional_plugin_requests_cannot_overlap() {
    let error = EntryConfig::new(EntryProfile::Runtime)
        .with_runtime_plugins(
            [zircon_runtime::builtin::RuntimePluginId::Sound],
            [zircon_runtime::builtin::RuntimePluginId::Sound],
        )
        .resolve()
        .unwrap_err();

    assert_eq!(
        error,
        ProductHostConfigError::RuntimePluginRequirementConflict(
            zircon_runtime::builtin::RuntimePluginId::Sound
        )
    );
}

#[test]
fn optional_request_cannot_downgrade_a_profile_required_plugin() {
    let resolved = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d)
        .with_optional_runtime_plugins([zircon_runtime::builtin::RuntimePluginId::Sound])
        .resolve()
        .expect("optional overlay should preserve stronger profile requirements");
    let sound = resolved
        .project_plugin_manifest()
        .expect("client profile should resolve a plugin manifest")
        .selections
        .iter()
        .find(|selection| selection.id == zircon_runtime::builtin::RuntimePluginId::Sound.key())
        .expect("client profile should retain sound");

    assert!(sound.enabled);
    assert!(sound.required);
    assert_eq!(sound.target_modes, [RuntimeTargetMode::ClientRuntime]);
}

#[test]
fn editor_setting_provenance_is_recorded_per_resolved_field() {
    let resolved = EntryConfig::new(EntryProfile::Editor)
        .with_editor_enabled_subsystems(["scene"])
        .resolve()
        .expect("editor subsystem request should resolve");

    assert_eq!(
        resolved.provenance().editor_enabled_subsystems(),
        ProductConfigSource::EntryRequest
    );
    assert_eq!(
        resolved.provenance().editor_runtime_sandbox(),
        ProductConfigSource::ProductRole
    );
}

#[test]
fn every_product_role_has_one_stable_artifact_target_descriptor() {
    let mut artifact_targets = std::collections::BTreeSet::new();

    for role in ProductRoleRequest::ALL {
        let descriptor = role.descriptor();

        assert_eq!(descriptor.role(), role);
        assert!(artifact_targets.insert(descriptor.artifact_manifest().target_name()));
    }

    assert_eq!(artifact_targets.len(), ProductRoleRequest::ALL.len());
}

#[test]
fn product_artifact_delivery_matrix_does_not_promote_missing_products() {
    let cases = [
        (
            ProductRoleRequest::EditorHost,
            ProductArtifactDeliveryStatus::Runnable,
            true,
            true,
        ),
        (
            ProductRoleRequest::DesktopClient,
            ProductArtifactDeliveryStatus::Preview,
            true,
            true,
        ),
        (
            ProductRoleRequest::Server,
            ProductArtifactDeliveryStatus::ConfigurationOnly,
            true,
            false,
        ),
        (
            ProductRoleRequest::WebClient,
            ProductArtifactDeliveryStatus::Unavailable,
            false,
            false,
        ),
        (
            ProductRoleRequest::AndroidClient,
            ProductArtifactDeliveryStatus::Unavailable,
            false,
            false,
        ),
        (
            ProductRoleRequest::EditorPlayChild,
            ProductArtifactDeliveryStatus::Unavailable,
            false,
            false,
        ),
        (
            ProductRoleRequest::Commandlet,
            ProductArtifactDeliveryStatus::Unavailable,
            false,
            false,
        ),
        (
            ProductRoleRequest::Embedded,
            ProductArtifactDeliveryStatus::Unavailable,
            false,
            false,
        ),
    ];

    for (role, expected_status, expected_config_owner, expected_runnable_artifact) in cases {
        let manifest = role.descriptor().artifact_manifest();

        assert_eq!(manifest.delivery_status(), expected_status);
        assert_eq!(manifest.has_configuration_owner(), expected_config_owner);
        assert_eq!(manifest.has_runnable_artifact(), expected_runnable_artifact);
    }
}

#[test]
fn runnable_product_artifacts_name_their_real_cargo_targets() {
    let editor = ProductRoleRequest::EditorHost
        .descriptor()
        .artifact_manifest();
    assert_eq!(editor.target_name(), "zircon_editor");
    assert_eq!(editor.kind(), ProductArtifactKind::NativeExecutable);
    assert_eq!(editor.required_build_feature(), Some("target-editor-host"));

    let desktop = ProductRoleRequest::DesktopClient
        .descriptor()
        .artifact_manifest();
    assert_eq!(desktop.target_name(), "zircon_runtime");
    assert_eq!(desktop.kind(), ProductArtifactKind::NativeExecutable);
    assert_eq!(desktop.required_build_feature(), Some("target-client"));

    let server = ProductRoleRequest::Server.descriptor().artifact_manifest();
    assert_eq!(server.target_name(), "zircon_server");
    assert_eq!(server.required_build_feature(), Some("target-server"));
    assert_eq!(
        server.delivery_status(),
        ProductArtifactDeliveryStatus::ConfigurationOnly
    );
}

#[test]
fn desktop_and_server_role_policies_are_structurally_distinct() {
    let desktop = ProductRoleRequest::DesktopClient.descriptor();
    assert_eq!(desktop.entry_kind(), ProductEntryKind::NativeProcess);
    assert_eq!(desktop.runner_kind(), ProductRunnerKind::DesktopEventLoop);
    assert_eq!(
        desktop.runtime_linkage(),
        ProductRuntimeLinkage::NativeDynamic
    );
    assert_eq!(
        desktop.capabilities().platform(),
        ProductPlatformClass::Desktop
    );
    assert_eq!(
        desktop.capabilities().window(),
        ProductCapabilityRequirement::Optional
    );
    assert_eq!(
        desktop.capabilities().input(),
        ProductCapabilityRequirement::Optional
    );
    assert_eq!(
        desktop.capabilities().render(),
        ProductCapabilityRequirement::Optional
    );
    assert_eq!(
        desktop.shutdown_policy(),
        ProductShutdownPolicy::ProcessCoordinated
    );

    let server = ProductRoleRequest::Server.descriptor();
    assert_eq!(server.runner_kind(), ProductRunnerKind::HeadlessSchedule);
    assert_eq!(server.runtime_linkage(), ProductRuntimeLinkage::Static);
    assert_eq!(
        server.capabilities().platform(),
        ProductPlatformClass::DesktopOrHeadless
    );
    assert_eq!(
        server.capabilities().window(),
        ProductCapabilityRequirement::Forbidden
    );
    assert_eq!(
        server.capabilities().input(),
        ProductCapabilityRequirement::Optional
    );
    assert_eq!(
        server.capabilities().render(),
        ProductCapabilityRequirement::Forbidden
    );
}

#[test]
fn export_platform_is_preserved_in_the_resolved_product_contract() {
    let resolved = EntryConfig::for_product_role(ProductRoleRequest::DesktopClient)
        .with_export_profile(ExportProfile::new(
            "linux-client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Linux,
            RuntimeProfileId::Client2d,
        ))
        .resolve()
        .expect("desktop export target should resolve into the product contract");

    assert_eq!(
        resolved.platform_target(),
        zircon_runtime::platform::PlatformTarget::Linux
    );
    assert_eq!(
        resolved.role_descriptor(),
        ProductRoleRequest::DesktopClient.descriptor()
    );
}

#[test]
fn module_selection_diagnostics_include_product_artifact_and_capability_policy() {
    let diagnostics =
        EntryRunner::module_selection_diagnostics(EntryConfig::new(EntryProfile::Runtime))
            .expect("desktop product diagnostics should resolve");

    for expected in [
        "entry.product_artifact.target=zircon_runtime",
        "entry.product_artifact.delivery=preview",
        "entry.product_runner=desktop_event_loop",
        "entry.product_runtime_linkage=native_dynamic",
        "entry.product_capability.platform=desktop",
        "entry.product_capability.window=optional",
        "entry.product_capability.input=optional",
        "entry.product_capability.render=optional",
        "entry.product_shutdown=process_coordinated",
    ] {
        assert!(diagnostics.contains(expected), "missing `{expected}`");
    }
    assert!(!diagnostics.contains("entry.product_required_runtime_module"));
}
