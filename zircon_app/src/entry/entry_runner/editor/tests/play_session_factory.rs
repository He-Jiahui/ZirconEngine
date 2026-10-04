#[test]
fn embedded_play_keeps_startup_cleanup_explicit_and_non_blocking() {
    let source = include_str!("../play_session_factory.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("play session factory production source must precede its tests");
    assert!(production.contains("error.diagnostic_with_recovery()"));
    assert!(!production.contains("error.retry_cleanup()"));
}
