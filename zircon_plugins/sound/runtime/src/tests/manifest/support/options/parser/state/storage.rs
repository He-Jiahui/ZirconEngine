use super::super::super::state::PendingOptionManifest;

// Preserves the static plugin.toml scanner's table-boundary behavior for option rows.
#[derive(Default)]
pub(in super::super) struct OptionManifestParserState {
    pub(in super::super) options: Vec<zircon_runtime::plugin::PluginOptionManifest>,
    pub(in super::super) pending: PendingOptionManifest,
    pub(in super::super) inside_option: bool,
}
