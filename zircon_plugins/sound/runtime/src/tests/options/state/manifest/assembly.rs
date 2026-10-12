// 先构造基础选项，再附加枚举及能力门槛，以运行时相同的值类型做元数据对照。
use super::super::signature::OptionManifestSignature;
use super::{attachments, base};

pub(in crate::tests::manifest::support::options::state::finalize) fn option_manifest_from_pending(
    signature: OptionManifestSignature,
    enum_values: Vec<String>,
    required_capability: Option<String>,
) -> zircon_runtime::plugin::PluginOptionManifest {
    attachments::apply_option_manifest_attachments(
        base::option_manifest_from_signature(signature),
        enum_values,
        required_capability,
    )
}
