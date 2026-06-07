use super::super::manifest;
use super::super::signature::OptionManifestSignature;

pub(in crate::tests::manifest::support::options::state::finalize) fn push_option_manifest(
    options: &mut Vec<zircon_runtime::plugin::PluginOptionManifest>,
    signature: OptionManifestSignature,
    enum_values: Vec<String>,
    required_capability: Option<String>,
) {
    options.push(manifest::option_manifest_from_pending(
        signature,
        enum_values,
        required_capability,
    ));
}
