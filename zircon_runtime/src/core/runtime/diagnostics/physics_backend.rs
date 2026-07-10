/// Always-on tooling projection of an optional Physics backend contract.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePhysicsBackendDiagnostics {
    pub requested_backend: String,
    pub active_backend: Option<String>,
    pub state: String,
    pub detail: Option<String>,
    pub simulation_mode: String,
    pub feature_gate: Option<String>,
}
