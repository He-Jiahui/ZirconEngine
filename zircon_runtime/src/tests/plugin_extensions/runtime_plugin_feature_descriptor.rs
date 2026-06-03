use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
    RuntimePluginCatalog, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
};
use crate::RuntimeTargetMode;

#[test]
fn runtime_plugin_feature_registration_report_rejects_invalid_feature_ids() {
    let feature = FeatureManifestFixture::new(PluginFeatureBundleManifest::new(
        "soundfeature",
        "Sound Feature",
        "sound",
    ));
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature id `soundfeature`")
        && diagnostic.contains("dot-separated namespace")));

    let mut catalog = RuntimePluginCatalog::default();
    catalog.register_feature(&feature);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| diagnostic
        .contains("feature id `soundfeature`")
        && diagnostic.contains("dot-separated namespace")));
}

#[test]
fn runtime_plugin_feature_registration_report_rejects_invalid_feature_owners() {
    let uppercase_owner = FeatureManifestFixture::new(PluginFeatureBundleManifest::new(
        "sound.timeline",
        "Sound Timeline",
        "Sound",
    ));
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&uppercase_owner);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("owner_plugin_id `Sound`")
            && diagnostic.contains("lowercase ASCII")));

    let cross_owner = FeatureManifestFixture::new(PluginFeatureBundleManifest::new(
        "animation.timeline",
        "Animation Timeline",
        "sound",
    ));
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&cross_owner);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature id `animation.timeline`")
        && diagnostic.contains("owner_plugin_id `sound`")));
}

#[test]
fn runtime_plugin_feature_registration_report_rejects_malformed_owner_package_tokens() {
    let malformed_owner = FeatureManifestFixture::new(PluginFeatureBundleManifest::new(
        "1sound__.timeline",
        "Sound Timeline",
        "1sound__",
    ));
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&malformed_owner);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("owner_plugin_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("owner_plugin_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
}

#[test]
fn runtime_plugin_feature_registration_report_rejects_untrimmed_display_names() {
    let feature = FeatureManifestFixture::new(PluginFeatureBundleManifest::new(
        "sound.timeline",
        " Sound Timeline ",
        "sound",
    ));
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("display_name ` Sound Timeline `")
        && diagnostic.contains("non-empty and trimmed")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_empty_capabilities() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound");
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(
        |diagnostic| diagnostic.contains("capabilities") && diagnostic.contains("at least one")
    ));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_capabilities() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("Runtime.Feature.Sound");
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("capability `Runtime.Feature.Sound`")
        && diagnostic.contains("lowercase ASCII")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_duplicate_capabilities() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_capability("runtime.feature.sound.timeline");
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("capability `runtime.feature.sound.timeline`")
        && diagnostic.contains("unique")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_empty_dependencies() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline");
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(
        |diagnostic| diagnostic.contains("dependencies") && diagnostic.contains("at least one")
    ));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("exactly one primary dependency")
        && diagnostic.contains("found 0")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_dependencies() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "Sound",
            "Runtime.Plugin.Sound",
        ));
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("dependency plugin_id `Sound`")
        && diagnostic.contains("lowercase ASCII")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("dependency capability `Runtime.Plugin.Sound`")
        && diagnostic.contains("lowercase ASCII")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_malformed_dependency_package_tokens() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "1sound__",
            "runtime.plugin.sound",
        ));
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("dependency plugin_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("dependency plugin_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_malformed_provider_package_tokens() {
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        valid_native_feature_manifest(),
        Some("1sound__".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("provider_package_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("provider_package_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_untrimmed_provider_package_tokens() {
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        valid_native_feature_manifest(),
        Some(" sound_timeline ".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("provider_package_id ` sound_timeline `")
        && diagnostic.contains("non-empty and trimmed")));
}

#[test]
fn runtime_plugin_feature_registration_report_rejects_malformed_provider_package_id_overrides() {
    let feature = FeatureManifestFixture::new(valid_native_feature_manifest());
    let registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature)
        .with_provider_package_id("1sound__");

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("provider_package_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("provider_package_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_duplicate_dependencies() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_dependency(PluginFeatureDependency::required(
            "sound",
            "runtime.plugin.sound",
        ));
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("dependency `sound` capability `runtime.plugin.sound`")
        && diagnostic.contains("unique")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_primary_dependencies() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "animation",
            "runtime.plugin.animation",
        ))
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ));
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("primary dependency `animation`")
        && diagnostic.contains("owner_plugin_id `sound`")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("exactly one primary dependency")
        && diagnostic.contains("found 2")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_module_identities() {
    let feature = valid_native_feature_manifest().with_runtime_module(
        PluginModuleManifest::runtime("sound.other.runtime", "zircon-plugin-sound")
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["runtime.feature.sound.timeline"]),
    );
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module name `sound.other.runtime`")
        && diagnostic.contains("feature id `sound.timeline`")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module crate_name `zircon-plugin-sound`")
        && diagnostic.contains("zircon_plugin_")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_malformed_module_crate_tokens() {
    let feature = valid_native_feature_manifest().with_runtime_module(
        PluginModuleManifest::runtime("sound.timeline.runtime", "zircon_plugin_sound__runtime")
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["runtime.feature.sound.timeline"]),
    );
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module crate_name `zircon_plugin_sound__runtime`")
        && diagnostic.contains("repeated underscores")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_module_capabilities() {
    let feature = valid_native_feature_manifest().with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline.runtime",
            "zircon_plugin_sound_timeline_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capabilities([
            "editor.feature.sound.timeline",
            "editor.feature.sound.timeline",
        ]),
    );
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "runtime module `sound.timeline.runtime` capability `editor.feature.sound.timeline`"
        ) && diagnostic.contains("runtime.")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module `sound.timeline.runtime` capability `editor.feature.sound.timeline`")
        && diagnostic.contains("unique")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_invalid_module_target_modes() {
    let feature = valid_native_feature_manifest()
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline.runtime",
                "zircon_plugin_sound_timeline_runtime",
            )
            .with_capabilities(["runtime.feature.sound.timeline"]),
        )
        .with_runtime_module(
            PluginModuleManifest::editor(
                "sound.timeline.editor",
                "zircon_plugin_sound_timeline_editor",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["editor.feature.sound.timeline"]),
        );
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module `sound.timeline.runtime` target_modes")
        && diagnostic.contains("at least one")));
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("editor module `sound.timeline.editor` target mode ClientRuntime")
        && diagnostic.contains("EditorHost")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_duplicate_module_names() {
    let feature = valid_native_feature_manifest()
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline.runtime",
                "zircon_plugin_sound_timeline_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["runtime.feature.sound.timeline"]),
        )
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline.runtime",
                "zircon_plugin_sound_timeline_runtime_debug",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["runtime.feature.sound.timeline.debug"]),
        );
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("module name `sound.timeline.runtime`")
        && diagnostic.contains("unique")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_empty_default_packaging() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_default_packaging(Vec::<ExportPackagingStrategy>::new());
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("default_packaging")
            && diagnostic.contains("at least one")));
}

#[test]
fn native_runtime_plugin_feature_registration_report_rejects_duplicate_default_packaging() {
    let feature = PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_default_packaging([
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::LibraryEmbed,
        ]);
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        feature,
        Some("sound_timeline".to_string()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("default_packaging strategy LibraryEmbed")
        && diagnostic.contains("unique")));
}

struct FeatureManifestFixture {
    manifest: PluginFeatureBundleManifest,
}

impl FeatureManifestFixture {
    fn new(manifest: PluginFeatureBundleManifest) -> Self {
        Self { manifest }
    }
}

impl RuntimePluginFeature for FeatureManifestFixture {
    fn manifest(&self) -> PluginFeatureBundleManifest {
        self.manifest.clone()
    }

    fn register_runtime_extensions(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}

fn valid_native_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_capability("runtime.feature.sound.timeline")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
}
