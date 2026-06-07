// Keeps capability-status row finalization tied to the static TOML table scanner.
#[derive(Default)]
pub(in super::super) struct CapabilityStatusParserState {
    pub(in super::super) statuses: Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
    pub(in super::super) current_capability: Option<String>,
    pub(in super::super) current_status: Option<zircon_runtime::plugin::CapabilityStatus>,
    pub(in super::super) current_bevy_references: Vec<String>,
    pub(in super::super) inside_status: bool,
}
