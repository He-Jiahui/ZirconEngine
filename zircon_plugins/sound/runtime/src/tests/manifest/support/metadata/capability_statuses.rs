use super::super::values::{capability_status_from_plugin_toml, string_array_values};

pub(super) fn capability_statuses_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
    let mut parser = CapabilityStatusParserState::default();
    for line in manifest.lines().map(str::trim) {
        parser.parse_manifest_line(line);
    }
    parser.finish()
}

// Keeps capability-status row finalization tied to the static TOML table scanner.
#[derive(Default)]
struct CapabilityStatusParserState {
    statuses: Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
    current_capability: Option<String>,
    current_status: Option<zircon_runtime::plugin::CapabilityStatus>,
    current_bevy_references: Vec<String>,
    inside_status: bool,
}

impl CapabilityStatusParserState {
    fn parse_manifest_line(&mut self, line: &str) {
        if line == "[[capability_statuses]]" {
            self.push_current_status();
            self.inside_status = true;
            return;
        }
        if line.starts_with("[[") {
            self.push_current_status();
            self.inside_status = false;
        }
        if !self.inside_status {
            return;
        }
        if let Some(value) = line
            .strip_prefix("capability = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            self.current_capability = Some(value.to_string());
            return;
        }
        if let Some(value) = line
            .strip_prefix("status = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            self.current_status = Some(capability_status_from_plugin_toml(value));
            return;
        }
        if let Some(value) = line
            .strip_prefix("bevy_references = [")
            .and_then(|value| value.strip_suffix(']'))
        {
            self.current_bevy_references = string_array_values(value);
        }
    }

    fn finish(mut self) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
        self.push_current_status();
        self.statuses
    }

    fn push_current_status(&mut self) {
        let Some(capability) = self.current_capability.take() else {
            return;
        };
        let mut manifest = zircon_runtime::plugin::CapabilityStatusManifest::new(
            capability,
            self.current_status
                .take()
                .expect("sound capability status should declare status"),
        );
        for reference in self.current_bevy_references.drain(..) {
            manifest = manifest.with_bevy_reference(reference);
        }
        self.statuses.push(manifest);
    }
}
