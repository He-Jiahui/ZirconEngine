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
