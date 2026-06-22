use super::{
    DiagnosticStoreSnapshot, FrameDiagnostics, FrameDiagnosticsStatus, RuntimeAnimationDiagnostics,
    RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
};
use zircon_runtime_interface::ProfileSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub render: RuntimeRenderDiagnostics,
    pub physics: RuntimePhysicsDiagnostics,
    pub animation: RuntimeAnimationDiagnostics,
    pub store: DiagnosticStoreSnapshot,
    pub profile: ProfileSnapshot,
}

impl RuntimeDiagnosticsSnapshot {
    pub fn frame_diagnostics_statuses(&self) -> [FrameDiagnosticsStatus<'_>; 3] {
        [
            self.render.frame_diagnostics_status(),
            self.physics.frame_diagnostics_status(),
            self.animation.frame_diagnostics_status(),
        ]
    }
}

#[cfg(test)]
mod tests {
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
}
