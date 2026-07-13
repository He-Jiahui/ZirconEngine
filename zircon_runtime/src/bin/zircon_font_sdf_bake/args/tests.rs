use super::*;

#[test]
fn text_sdf_cli_expands_ranges_and_deduplicates_codepoints() {
    let args = FontSdfCliArgs::parse(
        [
            "--font",
            "font.ttf",
            "--cache-root",
            "cache",
            "--asset-guid",
            "12345678-90ab-4cde-8f01-234567890abc",
            "--codepoint-range",
            "U+0041-U+0043",
            "--codepoint",
            "U+0042",
        ]
        .map(OsString::from),
    )
    .expect("range CLI should parse");

    assert_eq!(
        args.request.selection,
        FontSdfGlyphSelection::Codepoints(vec![0x41, 0x42, 0x43])
    );
}
