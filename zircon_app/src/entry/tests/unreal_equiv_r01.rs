//! Behavioral models: Unreal EditorConfig/Private/Tests/JsonConfigTests.cpp,
//! FJsonConfigTests_LoadHierarchicalBool (:342) and LoadHierarchicalString (:407).
//! Only configuration precedence is mirrored; the window descriptor is an opaque
//! entry-owned JSON value and no platform/window host is started.
//! The activation-time case reads the same keys from a later module's `finish`
//! hook: UE applies FConfigFile::OverrideFromCommandline inside
//! FConfigContext::GenerateDestIniFile (ConfigContext.cpp), so no reader of the
//! loaded branch sees the saved layer above the override (official review R01-R1).

use std::sync::{Arc, Mutex};

use serde_json::json;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::framework::window::{
    WindowDescriptor, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
use zircon_runtime::core::{
    CoreError, CoreResult, ModuleContext, ModuleDescriptor, ModuleLifecycle,
};
use zircon_runtime::plugin::{
    PluginPackageRole, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::super::{EntryConfig, EntryProfile, ProductCompositionRequest};
use super::{product_composition::close_owner::close_composition, IsolatedConfigFile};

const SAMPLER_PLUGIN_ID: &str = "r01_activation_sampler";

#[derive(Default)]
struct ActivationSample {
    finish_calls: usize,
    window_title: Option<String>,
    inherited_string: Option<String>,
}

struct SamplingLifecycle(Arc<Mutex<ActivationSample>>);

impl ModuleLifecycle for SamplingLifecycle {
    fn finish(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        let mut sample = self.0.lock().unwrap();
        sample.finish_calls += 1;
        sample.window_title = core
            .load_config::<WindowDescriptor>(PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY)
            .ok()
            .map(|descriptor| descriptor.title);
        sample.inherited_string = core.load_config::<String>("r01.inherited_string").ok();
        Ok(())
    }
}

#[test]
fn unreal_equiv_r01_entry_overlay_wins_after_persisted_config_load() {
    let config_file = IsolatedConfigFile::new();
    let persisted = WindowDescriptor {
        title: "Foo".to_owned(),
        visible: true,
        ..WindowDescriptor::default()
    };
    let requested = WindowDescriptor::default()
        .without_primary_window()
        .with_title("Bar");
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal)
        .with_window_descriptor(requested);
    let expected = config
        .clone()
        .resolve()
        .expect("minimal entry config should resolve")
        .window_descriptor()
        .clone();
    assert_ne!(expected.title, persisted.title);
    assert_ne!(expected.visible, persisted.visible);

    let disk_values = json!({
        (PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY): persisted,
        "r01.inherited_bool": true,
        "r01.inherited_string": "Foo"
    });
    std::fs::write(
        config_file.path(),
        serde_json::to_vec(&disk_values).expect("persisted fixture should serialize"),
    )
    .expect("isolated config fixture should be written");

    let composition = ProductCompositionRequest::new(config)
        .with_config_file_path(config_file.path())
        .compose()
        .expect("minimal composition should load the isolated Foundation file");
    let observed = composition
        .core()
        .load_config::<WindowDescriptor>(PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY);
    let inherited_bool = composition.core().load_config::<bool>("r01.inherited_bool");
    let inherited_string = composition
        .core()
        .load_config::<String>("r01.inherited_string");
    close_composition(composition);

    // Like UE's child-over-parent reads, the final upper value wins and absent
    // upper keys retain the lower value. This case samples only the composed
    // result; the activation-time read is covered by the case below.
    let observed = observed.expect("entry-owned JSON should remain decodable");
    assert_eq!(observed, expected);
    assert_eq!(observed.title, "Bar");
    assert!(!observed.visible);
    assert!(inherited_bool.expect("a disk-only boolean must survive the entry overlay"));
    assert_eq!(
        inherited_string.expect("a disk-only string must survive the entry overlay"),
        "Foo"
    );
}

#[test]
fn unreal_equiv_r01_entry_overlay_wins_while_later_modules_activate() {
    let config_file = IsolatedConfigFile::new();
    let persisted = WindowDescriptor {
        title: "Foo".to_owned(),
        ..WindowDescriptor::default()
    };
    let disk_values = json!({
        (PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY): persisted,
        "r01.inherited_string": "Foo"
    });
    std::fs::write(
        config_file.path(),
        serde_json::to_vec(&disk_values).expect("persisted fixture should serialize"),
    )
    .expect("isolated config fixture should be written");

    let sample = Arc::new(Mutex::new(ActivationSample::default()));
    let descriptor = RuntimePluginDescriptor::builder(
        SAMPLER_PLUGIN_ID,
        SAMPLER_PLUGIN_ID,
        RuntimePluginId::new(SAMPLER_PLUGIN_ID),
        format!("zircon_plugin_{SAMPLER_PLUGIN_ID}_runtime"),
    )
    .with_target_modes([RuntimeTargetMode::ServerRuntime])
    .with_package_role(PluginPackageRole::Production)
    .with_module_descriptor(
        ModuleDescriptor::new(format!("{SAMPLER_PLUGIN_ID}.runtime"), SAMPLER_PLUGIN_ID)
            .with_lifecycle(Arc::new(SamplingLifecycle(sample.clone()))),
    )
    .build();
    let report = RuntimePluginRegistrationReport::from_plugin(&descriptor);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new(SAMPLER_PLUGIN_ID),
            true,
            true,
        )
        .with_target_modes([RuntimeTargetMode::ServerRuntime])],
    };
    let config = EntryConfig::new(EntryProfile::Headless)
        .with_project_plugins(manifest)
        .with_window_descriptor(
            WindowDescriptor::default()
                .without_primary_window()
                .with_title("Bar"),
        );

    let composition = ProductCompositionRequest::new(config)
        .with_config_file_path(config_file.path())
        .with_runtime_plugin_registrations([report])
        .compose()
        .expect("headless composition should activate the sampling module");
    let composed = composition
        .core()
        .load_config::<WindowDescriptor>(PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY);
    close_composition(composition);

    // The sampling module finishes after Foundation has loaded the disk file,
    // so a disk-only key is already visible and the colliding key must already
    // carry the entry value rather than the persisted "Foo".
    let sample = sample.lock().unwrap();
    assert_eq!(sample.finish_calls, 1);
    assert_eq!(sample.inherited_string.as_deref(), Some("Foo"));
    assert_eq!(sample.window_title.as_deref(), Some("Bar"));
    assert_eq!(
        composed
            .expect("entry-owned JSON should remain decodable")
            .title,
        "Bar"
    );
}
