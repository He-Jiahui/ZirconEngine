#[test]
fn editor_startup_leaves_play_backend_ownership_to_the_app_composition() {
    let source = include_str!("../editor_host_startup.rs");
    let product_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("product startup source should precede its tests");

    assert!(!product_source.contains("ProcessPlayBackend"));
    assert!(!product_source.contains("set_play_backend"));
}
