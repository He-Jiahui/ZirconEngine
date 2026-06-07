use super::super::super::super::storage::PendingOptionManifest;

pub(super) fn take_signature_key(pending: &mut PendingOptionManifest) -> Option<String> {
    pending.key.take()
}
