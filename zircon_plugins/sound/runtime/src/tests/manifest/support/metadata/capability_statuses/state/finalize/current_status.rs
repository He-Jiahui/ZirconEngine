use super::super::storage::CapabilityStatusParserState;
use super::{manifest, required};

impl CapabilityStatusParserState {
    pub(in super::super) fn push_current_status(&mut self) {
        let Some(capability) = self.current_capability.take() else {
            return;
        };
        self.statuses.push(manifest::capability_status_manifest(
            capability,
            required::take_required_capability_status(&mut self.current_status),
            self.current_bevy_references.drain(..),
        ));
    }
}
