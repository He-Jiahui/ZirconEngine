use super::diagnostic_mentions_plugin;

#[test]
fn plugin_diagnostic_matching_preserves_boundaries_and_embedded_prefixes() {
    assert!(diagnostic_mentions_plugin(
        "load failed: native plugin physics: invalid ABI",
        "physics"
    ));
    assert!(diagnostic_mentions_plugin(
        "native plugin physics skipped",
        "physics"
    ));
    assert!(!diagnostic_mentions_plugin(
        "native plugin physics2 skipped",
        "physics"
    ));
}

#[test]
fn plugin_diagnostic_matching_does_not_format_needles_per_message() {
    let source = include_str!("../diagnostics.rs");
    let function = source
        .split_once("fn diagnostic_mentions_plugin")
        .expect("diagnostic matcher should exist")
        .1
        .split_once("#[cfg(test)]")
        .expect("tests should follow the matcher")
        .0;

    assert!(!function.contains("format!"));
}
