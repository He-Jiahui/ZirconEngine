use std::sync::Arc;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, v2::UiV2CompiledDocument};

use super::store_cache;
use crate::ui::v2_design_tokens::active_editor_v2_design_tokens_snapshot;

/// Immutable template inputs retained by outer projection caches.
///
/// The `Arc`s keep a source/token generation alive while it is cached, so pointer
/// identity remains a reliable generation comparison without hashing template data.
#[derive(Clone)]
pub(crate) struct ViewTemplateResourceGeneration {
    compiled: Arc<UiV2CompiledDocument>,
    design_tokens: Arc<EditorDesignTokens>,
    font_database_generation: u64,
}

impl PartialEq for ViewTemplateResourceGeneration {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.compiled, &other.compiled)
            && Arc::ptr_eq(&self.design_tokens, &other.design_tokens)
            && self.font_database_generation == other.font_database_generation
    }
}

impl Eq for ViewTemplateResourceGeneration {}

pub(crate) fn view_template_resource_generation(
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
) -> Option<ViewTemplateResourceGeneration> {
    if !layout_asset_path.ends_with(".zui") {
        return None;
    }

    let outcome = store_cache::load_view_v2_store(layout_asset_path, style_imports).ok()?;
    Some(ViewTemplateResourceGeneration {
        compiled: outcome.compiled,
        design_tokens: active_editor_v2_design_tokens_snapshot(),
        font_database_generation: UiSurface::shared_font_database_generation(),
    })
}
