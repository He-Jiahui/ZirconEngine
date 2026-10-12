use super::{live_key, NativePluginLiveRegistry};
use crate::plugin::PluginModuleKind;

#[test]
fn replacement_returns_previous_value_without_replacing_key() {
    let mut registry = NativePluginLiveRegistry::default();
    assert_eq!(
        registry.insert(live_key(PluginModuleKind::Runtime, "physics"), 1_u8),
        None
    );
    let original_key = registry
        .plugin_ids(PluginModuleKind::Runtime)
        .next()
        .expect("runtime plugin id")
        .as_ptr();

    let borrowed_id = String::from("physics");
    assert_eq!(
        registry.insert(
            live_key(PluginModuleKind::Runtime, borrowed_id.as_str()),
            2_u8,
        ),
        Some(1)
    );

    assert_eq!(
        registry
            .plugin_ids(PluginModuleKind::Runtime)
            .next()
            .expect("runtime plugin id")
            .as_ptr(),
        original_key
    );
    assert_eq!(
        registry.get(&live_key(PluginModuleKind::Runtime, "physics")),
        Some(&2)
    );
}
