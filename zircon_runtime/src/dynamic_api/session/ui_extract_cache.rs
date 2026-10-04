use std::sync::Arc;

use crate::core::math::UVec2;
use crate::scene::World;
use crate::text::{TextRuntimeContext, TextRuntimeContextAccessError};
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
    pub(super) fn new_with_text_context(
        text_context: &TextRuntimeContext,
    ) -> Result<Self, TextRuntimeContextAccessError> {
        Ok(Self {
            entry: None,
            text_measure_cache: UiTextMeasureCache::new_with_text_context(text_context)?,
            #[cfg(test)]
            rebuild_count: 0,
        })
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
#[path = "tests/ui_extract_cache.rs"]
mod tests;
