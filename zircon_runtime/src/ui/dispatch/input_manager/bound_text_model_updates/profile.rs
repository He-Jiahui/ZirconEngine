//! 只记录固定名称、字节量和状态计数，保持密码输入及一般文本内容不进入性能诊断载荷。

use zircon_runtime_interface::ui::text::{UiTextModelUpdateOrigin, UiTextModelUpdateStatus};

#[inline]
pub(super) fn record_request(payload_bytes: usize, origin: UiTextModelUpdateOrigin, focused: bool) {
    crate::profile_counter!("runtime", "ui_text.model_update.requests", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.request_bytes",
        payload_bytes
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.bound_refresh_requests",
        (origin == UiTextModelUpdateOrigin::BoundRefresh) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.explicit_requests",
        matches!(
            origin,
            UiTextModelUpdateOrigin::ExplicitSetText | UiTextModelUpdateOrigin::ExplicitLoadText
        ) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.focused_requests",
        focused as usize
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = (payload_bytes, origin, focused);
}

#[inline]
pub(super) fn record_security_class(secure: bool) {
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.secure_requests",
        secure as usize
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = secure;
}

#[inline]
pub(super) fn record_receipt(status: UiTextModelUpdateStatus) {
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.applied_receipts",
        (status == UiTextModelUpdateStatus::Applied) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.unchanged_receipts",
        (status == UiTextModelUpdateStatus::Unchanged) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.deferred_receipts",
        (status == UiTextModelUpdateStatus::Deferred) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.conflict_receipts",
        (status == UiTextModelUpdateStatus::Conflict) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.rejected_receipts",
        (status == UiTextModelUpdateStatus::Rejected) as usize
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = status;
}

#[inline]
pub(super) fn record_pending_admission(payload_bytes: usize, superseded: bool) {
    crate::profile_counter!("runtime", "ui_text.model_update.pending_admissions", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.pending_admitted_bytes",
        payload_bytes
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.pending_supersessions",
        superseded as usize
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = (payload_bytes, superseded);
}

#[inline]
pub(super) fn record_pending_release(payload_bytes: usize) {
    crate::profile_counter!("runtime", "ui_text.model_update.pending_releases", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.model_update.pending_released_bytes",
        payload_bytes
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = payload_bytes;
}

#[cfg(test)]
#[path = "tests/profile.rs"]
mod tests;
