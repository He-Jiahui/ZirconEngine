// 静态选项基础签名与 sound_options 的构造字段一一对应，附加字段由后续阶段接入。
use super::super::super::super::storage::PendingOptionManifest;
use super::super::record::OptionManifestSignature;
use super::required;

pub(super) fn option_manifest_signature_from_pending(
    key: String,
    pending: &mut PendingOptionManifest,
) -> OptionManifestSignature {
    OptionManifestSignature {
        key,
        display_name: required::take_required_option_display_name(&mut pending.display_name),
        value_type: required::take_required_option_value_type(&mut pending.value_type),
        default_value: required::take_required_option_default_value(&mut pending.default_value),
    }
}
