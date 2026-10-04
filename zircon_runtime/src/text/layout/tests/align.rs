use super::*;

#[test]
fn arabic_kashida_offsets_follow_each_joining_pair() {
    assert_eq!(arabic_kashida_insertion_offsets("سلام"), vec![2, 4]);
}

#[test]
fn arabic_kashida_offsets_keep_combining_marks_with_their_base_grapheme() {
    assert_eq!(arabic_kashida_insertion_offsets("سَلَام"), vec![4, 8]);
}

#[test]
fn arabic_kashida_offsets_keep_an_explicit_joiner_with_its_base_grapheme() {
    assert_eq!(arabic_kashida_insertion_offsets("س\u{200d}لام"), vec![5, 7]);
}

#[test]
fn arabic_kashida_offsets_do_not_cross_an_explicit_non_joiner() {
    assert_eq!(arabic_kashida_insertion_offsets("س\u{200c}لام"), vec![7]);
}

#[test]
fn arabic_kashida_uses_unicode_joining_type_beyond_the_retired_ranges() {
    assert_eq!(arabic_kashida_insertion_offsets("س\u{0870}"), vec![2]);
}

#[test]
fn arabic_kashida_does_not_admit_other_joining_scripts() {
    assert!(arabic_kashida_insertion_offsets("\u{10acd}\u{10acd}").is_empty());
}

#[test]
fn bounded_arabic_kashida_offsets_sample_long_lines_without_collecting_every_pair() {
    assert_eq!(
        arabic_kashida_insertion_offsets_bounded("سلمسلمسلمسلم", 3),
        vec![4, 12, 20]
    );
}

#[test]
fn bounded_arabic_kashida_offsets_allow_no_insertions() {
    assert!(arabic_kashida_insertion_offsets_bounded("سلام", 0).is_empty());
}

#[test]
fn arabic_justify_requires_a_materialized_tatweel_for_extra_advance() {
    assert!(justify_line_advances("سلام", &[10.0; 4], 40.0, 41.0).is_none());
}

#[test]
fn arabic_justify_expands_a_marked_materialized_tatweel() {
    let adjusted = justify_line_advances("سـَلام", &[10.0; 5], 50.0, 55.0)
        .expect("marked tatweel remains a concrete justify target");

    assert_eq!(adjusted[1], 15.0);
}
