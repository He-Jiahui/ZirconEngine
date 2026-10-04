use super::*;

#[test]
fn editor_hybrid_gi_profile_parser_accepts_product_profile_labels() {
    assert_eq!(
        parse_editor_hybrid_gi_profile("fully-dynamic"),
        Some(RenderHybridGiProfile::FullyDynamic)
    );
    assert_eq!(
        parse_editor_hybrid_gi_profile("indoor-static"),
        Some(RenderHybridGiProfile::IndoorStatic)
    );
    assert_eq!(
        parse_editor_hybrid_gi_profile("open-world"),
        Some(RenderHybridGiProfile::OpenWorld)
    );
    assert_eq!(
        parse_editor_hybrid_gi_profile("cinematic"),
        Some(RenderHybridGiProfile::Cinematic)
    );
    assert_eq!(parse_editor_hybrid_gi_profile("unknown"), None);
}

#[test]
fn editor_hybrid_gi_environment_override_uses_a_process_cache() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/retained_host/viewport/editor_viewport_render_defaults.rs"),
    )
    .expect("render defaults source should read");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("render defaults production source should exist");

    assert!(production.contains("static PROFILE: OnceLock<Option<RenderHybridGiProfile>>"));
    assert_eq!(production.matches("std::env::var(").count(), 1);
}
