use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_submit_context_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "submit context child",
        sources.submit_context,
        &[
            "fn runtime_07_submit_context_shares_large_extract_payloads",
            "#[path = \"submit_context/camera_loop_sharing.rs\"]",
            "#[path = \"submit_context/source_extract_payloads.rs\"]",
            "#[path = \"submit_context/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "submit error paths child",
        sources.submit_error_paths,
        &["fn runtime_07_submit_paths_return_errors_for_checked_viewport_records"],
    );

    let submit_support_children = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        sources.submit_context_camera_loop,
        sources.submit_context_feedback_sidebands,
        sources.submit_context_source_extract_payloads,
        sources.submit_context_sources,
        sources.submit_context_split_layout,
        sources.submit_context_split_layout_route,
        sources.submit_context_split_layout_source_inventory,
        sources.submit_context_split_layout_sources,
        sources.submit_context_split_layout_status_docs,
        sources.submit_context_status_docs
    );
    assert_contains_all(
        "submit context support children",
        &submit_support_children,
        &[
            "assert_camera_loop_uses_shared_sources",
            "assert_feedback_sidebands_move_owned_payloads",
            "assert_source_extract_payloads_are_shared",
            "pub(super) struct SubmitContextSources",
            "runtime_15_runtime_07_submit_context_guard_child_owner_split",
            "runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split",
            "submit_context/split_layout/source_inventory.rs",
            "assert_submit_context_status_docs",
        ],
    );
}
