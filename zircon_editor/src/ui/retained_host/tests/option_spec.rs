use super::*;

#[test]
fn plain_option_uses_its_identity_as_the_label() {
    let option = parse_retained_option("surface");

    assert_eq!(option.id, "surface");
    assert_eq!(option.label, "surface");
    assert!(option.matches_id("surface"));
}

#[test]
fn structured_option_keeps_machine_identity_and_display_label() {
    let option = parse_retained_option("post_process|label=Post Process,focused");

    assert_eq!(option.id, "post_process");
    assert_eq!(option.label, "Post Process");
    assert!(option.has_flag("FOCUSED"));
    assert!(option.matches_id("post_process"));
    assert!(option.matches_id("Post Process"));
}
