use super::*;
use crate::asset::{Handle, TextureAsset};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator};

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).expect("valid resource locator")
}

#[test]
fn typed_asset_events_roundtrip_for_tooling_snapshots() {
    let handle =
        Handle::<TextureAsset>::new(ResourceId::from_stable_label("typed asset event snapshot"));
    let added = AssetEvent::Added {
        handle,
        locator: Some(locator("res://textures/event-snapshot.png")),
        revision: 1,
    };

    let added_json = serde_json::to_string(&added).expect("serialize added event");
    let decoded_added: AssetEvent<TextureAsset> =
        serde_json::from_str(&added_json).expect("deserialize added event");
    assert_eq!(decoded_added, added);
    assert!(added_json.contains("\"added\""));
    assert!(added_json.contains("\"revision\":1"));
    assert_eq!(added.event_kind(), AssetEventKind::Added);
    assert_eq!(
        added.locator(),
        Some(&locator("res://textures/event-snapshot.png"))
    );
    assert_eq!(added.previous_locator(), None);
    assert_eq!(added.revision(), 1);

    let renamed = AssetEvent::Renamed {
        handle,
        locator: Some(locator("res://textures/event-snapshot-renamed.png")),
        previous_locator: Some(locator("res://textures/event-snapshot.png")),
        revision: 2,
    };
    let renamed_json = serde_json::to_string(&renamed).expect("serialize renamed event");
    let decoded_renamed: AssetEvent<TextureAsset> =
        serde_json::from_str(&renamed_json).expect("deserialize renamed event");
    assert_eq!(decoded_renamed, renamed);
    assert!(renamed_json.contains("\"renamed\""));
    assert!(renamed_json.contains("event-snapshot-renamed.png"));
    assert!(renamed_json.contains("event-snapshot.png"));
    assert_eq!(decoded_renamed.handle().id(), handle.id());
    assert_eq!(decoded_renamed.event_kind(), AssetEventKind::Renamed);
    assert_eq!(
        decoded_renamed.locator(),
        Some(&locator("res://textures/event-snapshot-renamed.png"))
    );
    assert_eq!(
        decoded_renamed.previous_locator(),
        Some(&locator("res://textures/event-snapshot.png"))
    );
    assert_eq!(decoded_renamed.revision(), 2);
    assert_eq!(
        serde_json::to_string(&AssetEventKind::ReloadFailed).expect("serialize event kind"),
        "\"reload_failed\""
    );
}

#[test]
fn typed_asset_receiver_skips_other_resource_kinds_without_a_filter_thread() {
    let resources = crate::core::resource::ResourceManager::new();
    let typed = typed_event_receiver::<TextureAsset>(resources.subscribe());
    let shader_id = ResourceId::from_stable_label("typed event unrelated shader");
    let texture_id = ResourceId::from_stable_label("typed event target texture");
    resources
        .register_record(crate::core::resource::ResourceRecord::new(
            shader_id,
            ResourceKind::Shader,
            locator("res://shaders/unrelated.wgsl"),
        ))
        .unwrap();
    resources
        .register_record(crate::core::resource::ResourceRecord::new(
            texture_id,
            ResourceKind::Texture,
            locator("res://textures/target.png"),
        ))
        .unwrap();

    let event = typed.try_recv().expect("typed texture event");

    assert_eq!(event.handle().id(), texture_id);
    assert_eq!(event.revision(), 2);
}
