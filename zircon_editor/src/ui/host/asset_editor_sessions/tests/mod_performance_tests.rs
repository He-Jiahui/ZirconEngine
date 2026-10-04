#[test]
fn zui_suffix_check_does_not_lowercase_the_whole_asset_id() {
    let source = include_str!("../mod.rs");
    let allocating_fold = ["to_ascii_", "lowercase()"].concat();

    assert!(!source.contains(&allocating_fold));
}
