use std::sync::Arc;

use serde_json::{json, Value};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::ACTIVE_HOST_PAINT_THEME;

#[derive(Clone)]
pub(super) struct HostPaintTokenInput {
    pub(super) tokens: Arc<EditorDesignTokens>,
    pub(super) settings_generation: Option<u64>,
}

/// Observe only the immutable theme held by the frame being painted. A current
/// settings lookup or the authority's default theme cannot prove that consumption.
pub(crate) fn active_host_paint_token_input() -> Option<Value> {
    ACTIVE_HOST_PAINT_THEME.with_borrow(|active| {
        let snapshot = active.as_ref()?;
        let input = snapshot.token_input.as_ref()?;
        Some(json!({
            "schema": "dev.zircon.editor.host-paint-token-input",
            "version": 1,
            "observed": true,
            "authority": "host-paint-theme-snapshot",
            "themeGeneration": snapshot.generation,
            "settingsGeneration": input.settings_generation,
            "scaleFactor": snapshot.scale_factor,
            "tokens": input.tokens.as_ref(),
        }))
    })
}

#[cfg(test)]
#[path = "tests/normal_tokens.rs"]
mod tests;
