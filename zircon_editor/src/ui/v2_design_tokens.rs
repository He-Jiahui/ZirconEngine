//! Editor-only preparation of cached V2 documents before style resolution.
//!
//! `UiV2PrototypeStoreFileCache` owns immutable, reusable source documents.
//! Copying at this boundary keeps the editor's token namespace out of generic
//! runtime/game surfaces while still making it available to every editor V2
//! style resolver.

use std::sync::{OnceLock, RwLock};

use zircon_runtime::ui::v2::UiV2StyleResolver;
use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, v2::UiV2AssetDocument};

pub(crate) fn prepare_editor_v2_document(source: &UiV2AssetDocument) -> UiV2AssetDocument {
    let mut document = source.clone();
    let tokens = match active_editor_v2_design_tokens().read() {
        Ok(tokens) => tokens,
        Err(poisoned) => poisoned.into_inner(),
    };
    UiV2StyleResolver::register_editor_design_tokens(&mut document, &tokens);
    document
}

pub(crate) fn install_editor_v2_design_tokens(tokens: &EditorDesignTokens) {
    match active_editor_v2_design_tokens().write() {
        Ok(mut active) => *active = tokens.clone(),
        Err(poisoned) => *poisoned.into_inner() = tokens.clone(),
    }
}

pub(crate) fn active_editor_v2_design_tokens_snapshot() -> EditorDesignTokens {
    match active_editor_v2_design_tokens().read() {
        Ok(tokens) => tokens.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

fn active_editor_v2_design_tokens() -> &'static RwLock<EditorDesignTokens> {
    static TOKENS: OnceLock<RwLock<EditorDesignTokens>> = OnceLock::new();
    TOKENS.get_or_init(|| RwLock::new(EditorDesignTokens::workbench_dark()))
}
