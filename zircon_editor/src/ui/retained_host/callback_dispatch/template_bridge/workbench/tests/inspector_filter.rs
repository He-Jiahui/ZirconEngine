use super::*;

#[test]
fn term_matching_is_ascii_case_insensitive_and_empty_queries_match() {
    assert!(contains_ascii_case_insensitive("Mesh Renderer", "renderer"));
    assert!(contains_ascii_case_insensitive("Render Layer", "LAYER"));
    assert!(contains_ascii_case_insensitive("Transform", ""));
    assert!(!contains_ascii_case_insensitive("Position", "material"));
}
