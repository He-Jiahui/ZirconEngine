pub(super) fn assert_tokenized_assets(assets: &[(&str, &str, &[&str])]) {
    for &(asset_name, asset_source, required_tokens) in assets {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        for &token in required_tokens {
            assert!(
                asset_source.contains(token),
                "{asset_name} must use {token} instead of a local component value"
            );
        }
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce a naked hex color"
        );
    }
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}
