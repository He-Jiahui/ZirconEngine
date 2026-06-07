use super::super::signature::OptionManifestSignature;

pub(super) fn option_manifest_from_signature(
    signature: OptionManifestSignature,
) -> zircon_runtime::plugin::PluginOptionManifest {
    zircon_runtime::plugin::PluginOptionManifest::new(
        signature.key,
        signature.display_name,
        signature.value_type,
        signature.default_value,
    )
}
