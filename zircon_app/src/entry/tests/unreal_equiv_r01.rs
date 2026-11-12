//! Behavioral models: Unreal EditorConfig/Private/Tests/JsonConfigTests.cpp,
//! FJsonConfigTests_LoadHierarchicalBool (:342) and LoadHierarchicalString (:407).
//! Only configuration precedence is mirrored; the window descriptor is an opaque
//! entry-owned JSON value and no platform/window host is started.

use serde_json::json;
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};

use super::super::{EntryConfig, ProductCompositionRequest};
use super::{product_composition::close_owner::close_composition, IsolatedConfigFile};

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
    // upper keys retain the lower value. This does not sample module activation
    // and therefore does not close the settled R01-R1 intermediate-window risk.
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
