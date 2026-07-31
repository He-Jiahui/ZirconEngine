#[test]
fn gpu_timer_readback_recovers_from_absent_pending_state_without_panic() {
    let source = include_str!(
        "../../../../graphics/backend/render_backend/gpu_pass_timer/gpu_pass_timer.rs"
    );
    let production = source.split("\n#[cfg(test)]").next().unwrap_or_default();

    assert!(
        !production.contains(".expect("),
        "GPU timestamp readback must recover from an absent pending slot instead of panicking"
    );
    assert!(
        production.contains("let Some(pending) = slot.pending.take() else {"),
        "completed timestamp readback must take its pending state through a fail-closed branch"
    );
}
