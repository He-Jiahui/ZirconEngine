use std::sync::Arc;

use crate::core::math::UVec2;
use crate::scene::World;
use crate::text::font::FontCollectionService;
use crate::ui::surface::UiTextMeasureCache;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::hud::{runtime_session_hud_extract, HUD_COMPONENT_IDS};
use super::menu::{runtime_session_menu_extract, GAMEPLAY_MENU_COMPONENT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeUiExtractCacheKey {
    menu_generation: u64,
    hud_generations: [u64; HUD_COMPONENT_IDS.len()],
    viewport_size: UVec2,
    // Resolved glyph IDs and raster-face handles are valid for one font publication only.
    font_generation: u64,
}

impl RuntimeUiExtractCacheKey {
    fn from_world(world: &World, viewport_size: UVec2, font_generation: u64) -> Self {
        Self {
            menu_generation: world.dynamic_component_generation(GAMEPLAY_MENU_COMPONENT),
            hud_generations: HUD_COMPONENT_IDS
                .map(|component_id| world.dynamic_component_generation(component_id)),
            viewport_size,
            font_generation,
        }
    }
}

struct RuntimeUiExtractCacheEntry {
    key: RuntimeUiExtractCacheKey,
    extract: Option<Arc<UiRenderExtract>>,
}

pub(super) struct RuntimeUiExtractCache {
    entry: Option<RuntimeUiExtractCacheEntry>,
    text_measure_cache: UiTextMeasureCache,
    #[cfg(test)]
    rebuild_count: u64,
}

impl RuntimeUiExtractCache {
    pub(super) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        Self {
            entry: None,
            text_measure_cache: UiTextMeasureCache::new_with_font_collection(font_collection),
            #[cfg(test)]
            rebuild_count: 0,
        }
    }

    pub(super) fn current_extract(
        &mut self,
        world: &World,
        viewport_size: UVec2,
    ) -> Option<Arc<UiRenderExtract>> {
        self.text_measure_cache.begin_frame();
        let key = RuntimeUiExtractCacheKey::from_world(
            world,
            viewport_size,
            self.text_measure_cache.font_database_generation(),
        );
        let extract = if let Some(entry) = self.entry.as_ref().filter(|entry| entry.key == key) {
            crate::profile_counter!("runtime", "ui.fallback_extract.cache_hit", 1);
            crate::profile_counter!("runtime", "ui.fallback_extract.rebuild_count", 0);
            entry.extract.as_ref().map(Arc::clone)
        } else {
            let extract = match runtime_session_menu_extract(
                world,
                viewport_size,
                &mut self.text_measure_cache,
            ) {
                Some(extract) => Some(extract),
                None => {
                    runtime_session_hud_extract(world, viewport_size, &mut self.text_measure_cache)
                }
            }
            .map(Arc::new);
            #[cfg(test)]
            {
                self.rebuild_count = self.rebuild_count.saturating_add(1);
            }
            crate::profile_counter!("runtime", "ui.fallback_extract.cache_hit", 0);
            crate::profile_counter!("runtime", "ui.fallback_extract.rebuild_count", 1);
            crate::profile_counter!(
                "runtime",
                "ui.fallback_extract.command_count",
                extract
                    .as_ref()
                    .map_or(0, |extract| extract.list.commands.len())
            );
            self.entry = Some(RuntimeUiExtractCacheEntry { key, extract });
            self.entry
                .as_ref()
                .expect("runtime UI cache entry was just published")
                .extract
                .as_ref()
                .map(Arc::clone)
        };
        self.text_measure_cache.finish_frame();
        extract
    }

    #[cfg(test)]
    fn rebuild_count(&self) -> u64 {
        self.rebuild_count
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::scene::components::NodeKind;

    fn extract_cache() -> RuntimeUiExtractCache {
        RuntimeUiExtractCache::new_with_font_collection(FontCollectionService::new())
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
    fn injected_font_generation_change_rebuilds_the_fallback_extract() {
        let world = menu_world("Start");
        let font_collection = FontCollectionService::new();
        let mut cache =
            RuntimeUiExtractCache::new_with_font_collection(Arc::clone(&font_collection));
        let viewport = UVec2::new(640, 360);
        let first = cache.current_extract(&world, viewport).unwrap();
        let generation_before = font_collection.generation();

        let (generation_after, _, changed) = font_collection.mutate(|database| {
            database.set_default_ui_family("RuntimeUiExtractCacheGenerationTest")
        });
        let second = cache.current_extract(&world, viewport).unwrap();

        assert!(changed);
        assert!(generation_after > generation_before);
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(cache.rebuild_count(), 2);
    }
}
