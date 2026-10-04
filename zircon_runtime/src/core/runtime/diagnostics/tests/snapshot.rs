use super::{
    FrameDiagnosticsStatus, RuntimeAnimationDiagnostics, RuntimeDiagnosticsSnapshot,
    RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
};

#[test]
fn runtime_snapshot_frame_diagnostics_statuses_preserve_subdomains() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        render: RuntimeRenderDiagnostics::unavailable("render backend missing"),
        physics: RuntimePhysicsDiagnostics {
            available: true,
            fixed_hz: Some(60),
            ..Default::default()
        },
        animation: RuntimeAnimationDiagnostics::unavailable("animation manager missing"),
        store: Default::default(),
        profile: Default::default(),
    };

    assert_eq!(
        snapshot.frame_diagnostics_statuses(),
        [
            FrameDiagnosticsStatus {
                domain: "render",
                available: false,
                error: Some("render backend missing"),
            },
            FrameDiagnosticsStatus {
                domain: "physics",
                available: true,
                error: None,
            },
            FrameDiagnosticsStatus {
                domain: "animation",
                available: false,
                error: Some("animation manager missing"),
            },
        ]
    );
}
