use super::join_string_parts;

#[test]
fn exact_rendering_identifier_join_preserves_parts() {
    assert_eq!(
        join_string_parts(&["runtime.feature.rendering.", "shader_graph"]),
        "runtime.feature.rendering.shader_graph"
    );
}
