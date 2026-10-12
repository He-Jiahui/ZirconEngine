use super::{canonical_hash, consumed_tokens_receipt};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[test]
fn consumed_tokens_receipt_contains_the_complete_canonical_editor_map() {
    let tokens = EditorDesignTokens::workbench_dark();
    let receipt = consumed_tokens_receipt(&tokens).expect("consumed token receipt");
    let expected_tokens = tokens
        .cascade_token_values()
        .into_iter()
        .filter(|(name, _)| name.starts_with("editor."))
        .collect::<std::collections::BTreeMap<_, _>>();
    let expected_tokens = serde_json::to_value(expected_tokens).expect("serialize tokens");
    let expected_hash = canonical_hash(&expected_tokens).expect("hash tokens");

    assert_eq!(receipt["complete"].as_bool(), Some(true));
    assert_eq!(receipt["tokens"], expected_tokens);
    assert_eq!(receipt["sha256"].as_str(), Some(expected_hash.as_str()));
    assert!(receipt["tokens"]
        .get("editor.chrome.top_bar.height")
        .is_some());
    assert!(receipt["tokens"]
        .as_object()
        .expect("canonical token object")
        .keys()
        .all(|name| name.starts_with("editor.")));
}
