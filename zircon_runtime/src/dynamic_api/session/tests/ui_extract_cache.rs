use std::sync::Arc;

use super::*;
use crate::scene::components::NodeKind;
use crate::text::font::FontCollectionService;

fn extract_cache() -> RuntimeUiExtractCache {
    RuntimeUiExtractCache::new_with_text_context(
        &TextRuntimeContext::new().expect("runtime text context"),
    )
    .expect("runtime UI extract cache")
}

fn menu_world(label: &str) -> World {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({ "state": "start", "button": label }),
        )
        .expect("test menu component should be stored");
    world
}

#[test]
fn stable_generation_reuses_the_same_ui_extract_allocation() {
    let world = menu_world("Start");
    let mut cache = extract_cache();
    let viewport = UVec2::new(640, 360);

    let first = cache.current_extract(&world, viewport).unwrap();
    let second = cache.current_extract(&world, viewport).unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(cache.rebuild_count(), 1);
}

#[test]
fn unrelated_world_mutation_keeps_the_cached_ui_extract() {
    let mut world = menu_world("Start");
    let mut cache = extract_cache();
    let viewport = UVec2::new(640, 360);
    let first = cache.current_extract(&world, viewport).unwrap();

    world
        .spawn_node(NodeKind::Empty)
        .expect("unrelated test scene spawn should succeed");
    let second = cache.current_extract(&world, viewport).unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(cache.rebuild_count(), 1);
}

#[test]
fn target_component_mutation_rebuilds_the_ui_extract_once() {
    let mut world = menu_world("Start");
    let mut cache = extract_cache();
    let viewport = UVec2::new(640, 360);
    let first = cache.current_extract(&world, viewport).unwrap();
    let mut rows = Vec::new();
    world.dynamic_component_rows(GAMEPLAY_MENU_COMPONENT, &mut rows);
    let entity = rows.first().expect("test menu entity").0;

    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({ "state": "start", "button": "Changed" }),
        )
        .expect("changed test menu component should be stored");
    let second = cache.current_extract(&world, viewport).unwrap();

    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(cache.rebuild_count(), 2);
    assert!(second
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Changed")));
    let layout_report = cache.text_measure_cache.frame_layout_report();
    assert_eq!(layout_report.hit_count, 2);
    assert_eq!(layout_report.miss_count, 1);
}

#[test]
fn viewport_resize_rebuilds_the_ui_extract_once() {
    let world = menu_world("Start");
    let mut cache = extract_cache();
    let first = cache.current_extract(&world, UVec2::new(640, 360)).unwrap();

    let second = cache
        .current_extract(&world, UVec2::new(1280, 720))
        .unwrap();

    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(cache.rebuild_count(), 2);
}

#[test]
fn stable_absent_ui_does_not_revisit_component_rows() {
    let world = World::empty();
    let mut cache = extract_cache();
    let viewport = UVec2::new(640, 360);

    assert!(cache.current_extract(&world, viewport).is_none());
    assert!(cache.current_extract(&world, viewport).is_none());
    assert_eq!(cache.rebuild_count(), 1);
}

#[test]
fn fallback_extract_cache_key_tracks_the_injected_font_generation() {
    let world = menu_world("Start");
    let key = RuntimeUiExtractCacheKey::from_world(&world, UVec2::new(640, 360), 42);

    assert_eq!(key.font_generation, 42);
}

#[test]
fn runtime_text_context_fallback_extract_cache_retains_the_injected_identity() {
    let text_context = TextRuntimeContext::new().expect("runtime text context");
    let cache = RuntimeUiExtractCache::new_with_text_context(&text_context)
        .expect("runtime UI extract cache");

    assert_eq!(
        cache.text_measure_cache.text_runtime_context_id(),
        Some(text_context.id())
    );
}

#[test]
fn injected_font_generation_change_rebuilds_the_fallback_extract() {
    let world = menu_world("Start");
    let font_collection = FontCollectionService::new();
    let text_context = TextRuntimeContext::new_with_font_collection(Arc::clone(&font_collection))
        .expect("runtime text context");
    let mut cache = RuntimeUiExtractCache::new_with_text_context(&text_context)
        .expect("runtime UI extract cache");
    let viewport = UVec2::new(640, 360);
    let first = cache.current_extract(&world, viewport).unwrap();
    let generation_before = font_collection.generation();

    let (generation_after, _, changed) = font_collection
        .mutate(|database| database.set_default_ui_family("RuntimeUiExtractCacheGenerationTest"));
    let second = cache.current_extract(&world, viewport).unwrap();

    assert!(changed);
    assert!(generation_after > generation_before);
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(cache.rebuild_count(), 2);
}
