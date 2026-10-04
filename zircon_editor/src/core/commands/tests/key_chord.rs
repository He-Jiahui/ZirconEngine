use std::str::FromStr;

use super::EditorKeyChord;

#[test]
fn chord_format_and_alias_normalization_preserve_canonical_text() {
    assert_eq!(
        EditorKeyChord::from_str("command+option+del")
            .unwrap()
            .to_string(),
        "Alt+Meta+Delete"
    );
    assert_eq!(EditorKeyChord::new("escape").to_string(), "Escape");
    assert_eq!(EditorKeyChord::new("f12").to_string(), "F12");
}

#[test]
fn chord_validity_requires_one_non_modifier_key() {
    assert!(EditorKeyChord::from_str("Ctrl+S").unwrap().is_valid());
    assert!(!EditorKeyChord::new("").is_valid());
    assert!(!EditorKeyChord::new("Ctrl").is_valid());
    assert!(!EditorKeyChord::new("DeadAcute").is_valid());
    assert!(!EditorKeyChord::new("Unidentified").is_valid());
}

#[test]
fn hot_chord_normalization_and_display_do_not_build_temporary_lowercase_or_parts_lists() {
    let source = include_str!("../key_chord.rs");
    let lowercase_temporary = ["to_ascii_lowercase()", ".as_str()"].concat();
    let parts_list = ["let mut parts = ", "Vec::new()"].concat();

    assert!(!source.contains(&lowercase_temporary));
    assert!(!source.contains(&parts_list));
}
