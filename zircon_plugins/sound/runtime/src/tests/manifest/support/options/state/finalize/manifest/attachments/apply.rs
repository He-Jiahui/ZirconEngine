use super::{enum_values, required_capability};

pub(in crate::tests::manifest::support::options::state::finalize::manifest) fn apply_option_manifest_attachments(
    option: zircon_runtime::plugin::PluginOptionManifest,
    enum_values: Vec<String>,
    required_capability: Option<String>,
) -> zircon_runtime::plugin::PluginOptionManifest {
    required_capability::apply_required_capability(
        enum_values::apply_enum_values(option, enum_values),
        required_capability,
    )
}
