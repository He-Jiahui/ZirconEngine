use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_text_bound_model_updates_are_bounded_child_owner() {
    let manager = read_runtime_src("ui/dispatch/input_manager/manager.rs");
    let owner = read_runtime_src("ui/dispatch/input_manager/bound_text_model_updates.rs");
    let transaction =
        read_runtime_src("ui/dispatch/input_manager/bound_text_model_updates/transaction.rs");
    let profile = read_runtime_src("ui/dispatch/input_manager/bound_text_model_updates/profile.rs");
    let tests = read_runtime_src("ui/dispatch/input_manager/bound_text_model_updates/tests.rs");
    let secure_store = read_runtime_src("ui/surface/secure_text_values.rs");
    let runtime_manifest = read_repo("zircon_runtime/Cargo.toml");
    let interface = read_repo("zircon_runtime_interface/src/ui/text/model_update.rs");
    let structure = read_repo("docs/plans/engine-code-structure-convention.md");
    let findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let text_plan = read_repo("docs/plans/zircon_runtime/text/08-ime-and-text-input.md");

    for public_method in [
        "pub fn text_document_key(",
        "pub fn update_text_model(",
        "pub fn drain_text_model_update_receipts(",
    ] {
        assert!(
            !manager.contains(public_method),
            "input-manager root must delegate `{public_method}` to the bound-model child"
        );
        assert!(
            owner.contains(public_method),
            "bound-model child must own `{public_method}`"
        );
    }

    assert_contains_all(
        "bound-model update child keeps one bounded document/session transaction owner",
        &owner,
        &[
            "MVP_MAX_PENDING_TEXT_MODEL_UPDATES: usize = 256",
            "MVP_MAX_PENDING_TEXT_MODEL_UPDATE_BYTES: usize = 16 * 1024 * 1024",
            "MVP_MAX_TEXT_MODEL_UPDATE_VALUE_BYTES: usize = 4 * 1024 * 1024",
            "pending: BTreeMap<UiNodeId, UiPendingTextModelUpdate>",
            "store_pending_secure_text_model_update",
            "take_pending_secure_text_model_update",
            "impl Drop for UiTextModelUpdateState",
            "active_secure_store",
        ],
    );
    assert_contains_all(
        "secure deferred values remain Surface-owned behind a clear-only manager handle",
        &secure_store,
        &[
            "pub(in crate::ui) struct UiPendingSecureTextModelUpdateStoreHandle",
            "values: Arc<Mutex<BTreeMap<UiNodeId, UiPendingSecureTextModelValue>>>",
            "value: Zeroizing<String>",
            "fn into_string(mut self) -> String",
            "std::mem::take(&mut *self.value)",
            "pub(in crate::ui) fn clear(&self)",
            "pub(crate) fn store_pending_secure_text_model_update(",
            "pub(crate) fn take_pending_secure_text_model_update(",
            "pub(in crate::ui) fn pending_secure_text_model_update_store_handle(",
        ],
    );
    assert_contains_all(
        "pending secure text uses the audited zeroization dependency directly",
        &runtime_manifest,
        &["zeroize = \"1.8.2\""],
    );
    assert_contains_all(
        "bound-model transaction child owns committed-base projection and dual commit",
        &transaction,
        &[
            "PreparedUiEditableTextDocumentTransaction::new(properties, document).commit()",
            "UiTextEditSource::Programmatic",
            "fn committed_document_state(",
            "UiTextEditAction::CancelComposition",
            "UiTextHistoryCommit::Barrier",
        ],
    );
    assert_contains_all(
        "versioned interface distinguishes bound and explicit replacement",
        &interface,
        &[
            "pub enum UiTextModelUpdateOrigin",
            "BoundRefresh",
            "ExplicitSetText",
            "ExplicitLoadText",
            "pub expected_document: UiTextDocumentKey",
            "pub struct UiTextModelUpdateReceipt",
            ".field(\"value_byte_len\", &self.value.len())",
        ],
    );
    assert_contains_all(
        "behavior owner covers focused, conflict, supersession, preedit, secure and admission paths",
        &tests,
        &[
            "malformed_request_returns_a_valid_content_free_rejection",
            "focused_bound_refresh_defers_without_overwriting_the_edit_buffer",
            "latest_unchanged_refresh_supersedes_an_older_deferred_value",
            "user_edit_after_defer_wins_and_blur_emits_content_free_conflict",
            "explicit_load_text_uses_the_same_focused_force_review_boundary",
            "explicit_set_text_during_preedit_replaces_the_committed_document_once",
            "secure_deferred_value_lives_in_the_surface_secure_store",
            "secure_policy_change_rejects_and_discards_the_pending_value",
            "detached_secure_owner_rejects_and_discards_the_pending_value",
            "surface_switch_revokes_the_previous_surface_pending_secure_value",
            "manager_drop_revokes_pending_secure_values_from_the_surface_store",
            "oversized_focused_refresh_is_rejected_without_retention_or_mutation",
        ],
    );

    for (label, source) in [
        ("queue owner", owner.as_str()),
        ("transaction owner", transaction.as_str()),
    ] {
        for forbidden in [".unwrap()", ".expect(", "panic!(", "unreachable!("] {
            assert!(
                !source.contains(forbidden),
                "bound-model {label} must not contain `{forbidden}`"
            );
        }
    }
    for forbidden_profile_field in ["request_id", "tree_id", "node_id", "source_text"] {
        assert!(
            !profile
                .split("#[cfg(test)]")
                .next()
                .unwrap_or(&profile)
                .contains(forbidden_profile_field),
            "profile owner must not publish `{forbidden_profile_field}`"
        );
    }

    for (path, source) in [
        ("ui/dispatch/input_manager/manager.rs", manager.as_str()),
        (
            "ui/dispatch/input_manager/bound_text_model_updates.rs",
            owner.as_str(),
        ),
        (
            "ui/dispatch/input_manager/bound_text_model_updates/profile.rs",
            profile.as_str(),
        ),
        (
            "ui/dispatch/input_manager/bound_text_model_updates/transaction.rs",
            transaction.as_str(),
        ),
        (
            "ui/dispatch/input_manager/bound_text_model_updates/tests.rs",
            tests.as_str(),
        ),
        ("ui/surface/secure_text_values.rs", secure_store.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime production/test owner soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("structure convention", structure.as_str()),
        ("review findings", findings.as_str()),
        ("Text 08 plan", text_plan.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "focused_bound_model_update_gateway_implemented_unvalidated",
                "secure_pending_surface_owned_unvalidated",
            ],
        );
    }
}
