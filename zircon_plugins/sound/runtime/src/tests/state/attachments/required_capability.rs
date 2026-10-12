pub(super) fn apply_required_capability(
    option: zircon_runtime::plugin::PluginOptionManifest,
    required_capability: Option<String>,
) -> zircon_runtime::plugin::PluginOptionManifest {
    if let Some(capability) = required_capability {
        option.with_required_capability(capability)
    } else {
        option
    }
}
