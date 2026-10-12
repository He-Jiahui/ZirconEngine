// 用 key 确认当前表项有效，再提取比较所需的基础签名；缺失必填项让测试失败。
use super::super::super::super::storage::PendingOptionManifest;
use super::super::record::OptionManifestSignature;
use super::{assembly, key};

pub(in super::super::super) fn take_option_manifest_signature(
    pending: &mut PendingOptionManifest,
) -> Option<OptionManifestSignature> {
    let key = key::take_signature_key(pending)?;
    Some(assembly::option_manifest_signature_from_pending(
        key, pending,
    ))
}
