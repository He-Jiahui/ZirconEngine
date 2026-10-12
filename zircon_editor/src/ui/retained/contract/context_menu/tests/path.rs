use super::*;

#[test]
fn path_segment_writes_normalized_text_without_an_intermediate_string() {
    assert_eq!(normalized(" --Scene Tree / Props-- "), "scene-tree--props");
    assert_eq!(normalized("A_B.C"), "a_b.c");
    assert_eq!(normalized("---"), "");
}

fn normalized(value: &str) -> String {
    let mut path = String::from("workbench://test/");
    let prefix_len = path.len();
    push_path_segment(&mut path, value);
    path[prefix_len..].to_string()
}
