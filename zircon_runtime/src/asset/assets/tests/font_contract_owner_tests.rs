use std::path::Path;

#[test]
fn composite_font_contract_is_owned_by_the_font_asset_schema() {
    let asset = include_str!("../font.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("font asset production source must precede its tests");
    let cache = include_str!("../../artifact/cache_payload/font.rs");
    let text_family = include_str!("../../../text/model/font/family.rs");
    let text_font = include_str!("../../../text/model/font/mod.rs");

    assert!(asset.contains("pub struct CompositeFontDescriptor"));
    assert!(asset.contains("pub struct FontFamilyName"));
    assert!(!asset.contains("crate::text"));
    assert!(!cache.contains("crate::text"));
    assert!(!text_family.contains("pub struct FontFamilyName"));
    assert!(text_family.contains("use crate::asset::assets::FontFamilyName;"));
    assert!(text_font.contains("pub use crate::asset::assets::{"));
    for contract in [
        "CompositeFontDescriptor",
        "FontCultureTag",
        "FontFamilyName",
        "FontScript",
        "FontScriptTag",
        "SubFontRange",
    ] {
        assert!(text_font.contains(contract));
    }
    assert!(!Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/text/model/font/composite.rs")
        .exists());
}
