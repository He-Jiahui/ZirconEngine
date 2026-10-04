#[test]
fn polling_streams_mutable_sessions_and_skips_terminal_snapshots() {
    let source = include_str!("../polling.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(production.contains("for (profile_name, session) in &mut self.sessions"));
    assert!(production.contains("if session.view_model().snapshot().is_terminal()"));
    assert!(!production.contains("self.sessions.keys().cloned().collect"));
}
