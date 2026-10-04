use std::collections::BTreeMap;

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::RenderMaterialTextureTransform;

use super::{sync_texture_slot, MaterialTextureSlotValue};

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

#[test]
fn texture_slot_sync_preserves_existing_slot_metadata() {
    let mut previous = MaterialTextureSlotValue::new(reference("res://old.texture"));
    previous.fallback = Some("white".to_string());
    previous.transform = Some(RenderMaterialTextureTransform::default());
    previous.uv_channel = 3;
    let mut slots = BTreeMap::from([("base_color".to_string(), previous)]);
    let replacement = reference("res://new.texture");

    sync_texture_slot(&mut slots, "base_color", Some(&replacement));

    let synchronized = slots.get("base_color").unwrap();
    assert_eq!(synchronized.reference.as_ref(), Some(&replacement));
    assert_eq!(synchronized.fallback.as_deref(), Some("white"));
    assert!(synchronized.transform.is_some());
    assert_eq!(synchronized.uv_channel, 3);
}

#[test]
fn texture_slot_sync_inserts_a_new_slot() {
    let mut slots = BTreeMap::new();
    let reference = reference("res://new.texture");

    sync_texture_slot(&mut slots, "normal", Some(&reference));

    let synchronized = slots.get("normal").unwrap();
    assert_eq!(synchronized.reference.as_ref(), Some(&reference));
    assert_eq!(synchronized.fallback, None);
    assert_eq!(synchronized.transform, None);
    assert_eq!(synchronized.uv_channel, 0);
}
