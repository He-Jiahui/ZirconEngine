//! Editor-only preparation of cached V2 documents before style resolution.
//!
//! `UiV2PrototypeStoreFileCache` owns immutable, reusable source documents.
//! Copying at this boundary keeps the editor's token namespace out of generic
//! runtime/game surfaces while still making it available to every editor V2
//! style resolver.

use std::sync::{Arc, OnceLock, RwLock};

use zircon_runtime::ui::v2::UiV2StyleResolver;
use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, v2::UiV2AssetDocument};

use crate::core::settings::SettingsSnapshot;

/// A UI-only projection of the authority's immutable token payload. It never accepts local writes.
#[derive(Clone)]
struct EditorV2DesignTokenProjection {
    tokens: Arc<EditorDesignTokens>,
}

impl EditorV2DesignTokenProjection {
    fn synchronize(&mut self, tokens: Arc<EditorDesignTokens>) -> bool {
        if Arc::ptr_eq(&self.tokens, &tokens) {
            return false;
        }
        self.tokens = tokens;
        true
    }
}

pub(crate) fn prepare_editor_v2_document(source: &UiV2AssetDocument) -> UiV2AssetDocument {
    let tokens = active_editor_v2_design_tokens_snapshot();
    prepare_editor_v2_document_with_tokens(source, tokens.as_ref())
}

pub(crate) fn prepare_editor_v2_document_with_tokens(
    source: &UiV2AssetDocument,
    tokens: &EditorDesignTokens,
) -> UiV2AssetDocument {
    let mut document = source.clone();
    UiV2StyleResolver::register_editor_design_tokens(&mut document, tokens);
    document
}

/// Synchronizes the V2 resolver projection from the sole settings authority.
pub(crate) fn install_editor_v2_design_tokens(snapshot: &SettingsSnapshot) -> bool {
    match active_editor_v2_design_tokens().write() {
        Ok(mut active) => active.synchronize(snapshot.design_tokens_handle()),
        Err(poisoned) => poisoned
            .into_inner()
            .synchronize(snapshot.design_tokens_handle()),
    }
}

pub(crate) fn active_editor_v2_design_tokens_snapshot() -> Arc<EditorDesignTokens> {
    match active_editor_v2_design_tokens().read() {
        Ok(projection) => Arc::clone(&projection.tokens),
        Err(poisoned) => Arc::clone(&poisoned.into_inner().tokens),
    }
}

fn active_editor_v2_design_tokens() -> &'static RwLock<EditorV2DesignTokenProjection> {
    static TOKENS: OnceLock<RwLock<EditorV2DesignTokenProjection>> = OnceLock::new();
    TOKENS.get_or_init(|| {
        RwLock::new(EditorV2DesignTokenProjection {
            tokens: Arc::new(EditorDesignTokens::workbench_dark()),
        })
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

    use super::EditorV2DesignTokenProjection;

    #[test]
    fn projection_replaces_only_a_distinct_authority_token_payload() {
        let first = Arc::new(EditorDesignTokens::workbench_dark());
        let second = Arc::new(first.as_ref().clone());
        let mut projection = EditorV2DesignTokenProjection {
            tokens: Arc::clone(&first),
        };

        assert!(!projection.synchronize(Arc::clone(&first)));
        assert!(projection.synchronize(Arc::clone(&second)));
        assert!(Arc::ptr_eq(&projection.tokens, &second));
    }
}
