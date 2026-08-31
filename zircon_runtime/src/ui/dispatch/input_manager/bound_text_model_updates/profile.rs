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
mod tests {
    #[test]
    fn model_update_profile_uses_only_fixed_content_free_counter_names() {
        let source = include_str!("profile.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for name in [
            "ui_text.model_update.requests",
            "ui_text.model_update.request_bytes",
            "ui_text.model_update.bound_refresh_requests",
            "ui_text.model_update.explicit_requests",
            "ui_text.model_update.focused_requests",
            "ui_text.model_update.secure_requests",
            "ui_text.model_update.applied_receipts",
            "ui_text.model_update.unchanged_receipts",
            "ui_text.model_update.deferred_receipts",
            "ui_text.model_update.conflict_receipts",
            "ui_text.model_update.rejected_receipts",
            "ui_text.model_update.pending_admissions",
            "ui_text.model_update.pending_admitted_bytes",
            "ui_text.model_update.pending_supersessions",
            "ui_text.model_update.pending_releases",
            "ui_text.model_update.pending_released_bytes",
        ] {
            assert!(production.contains(name), "missing fixed counter {name}");
        }
        for forbidden in ["request_id", "tree_id", "node_id", "source_text"] {
            assert!(
                !production.contains(forbidden),
                "dynamic or content-bearing profile field {forbidden}"
            );
        }
    }
}
