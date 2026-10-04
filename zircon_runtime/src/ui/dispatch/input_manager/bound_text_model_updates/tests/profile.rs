#[test]
fn model_update_profile_uses_only_fixed_content_free_counter_names() {
    let source = include_str!("../profile.rs");
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
