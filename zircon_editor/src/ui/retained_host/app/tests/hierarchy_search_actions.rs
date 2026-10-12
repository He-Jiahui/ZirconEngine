use super::support::*;

#[test]
fn authored_outliner_edit_commit_and_clear_reach_the_host_hierarchy_filter() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_authored_outliner_search");
    harness.activate_workbench_page();
    let entries = harness
        .host
        .borrow()
        .runtime
        .chrome_snapshot()
        .expect("authoring scene should be available")
        .scene_entries;
    let query = entries
        .iter()
        .find(|entry| !entry.display_name.trim().is_empty())
        .expect("the default scene should contain a named object")
        .display_name
        .clone();

    {
        let host = harness.host.borrow();
        for (action, binding) in [
            (
                "workbench.hierarchy.search.edit",
                "Workbench/SceneSearchEditFromShell",
            ),
            (
                "workbench.hierarchy.search.commit",
                "Workbench/SceneSearchCommitFromShell",
            ),
        ] {
            assert_eq!(
                host.workbench_window_bridge
                    .binding_id_for_action_id(action)
                    .as_deref(),
                Some(binding),
                "the current authored search field must expose the actual action"
            );
        }
    }
    let journal_baseline = harness.journal_len();

    for (binding, value) in [
        ("workbench.hierarchy.search.edit", query.as_str()),
        (
            "workbench.hierarchy.search.commit",
            "mvp-no-matching-object-7c93",
        ),
        ("Workbench/SceneSearchEditFromShell", ""),
        ("Workbench/SceneSearchCommitFromShell", query.as_str()),
        ("Workbench/SceneSearchEdit", ""),
        ("Workbench/SceneSearchCommit", query.as_str()),
        ("workbench.hierarchy.search.edit", ""),
    ] {
        pane_surface_host(&harness.root_ui).invoke_surface_control_edited(
            "WorkbenchSceneSearchField".into(),
            binding.into(),
            value.into(),
        );
        let mut host = harness.host.borrow_mut();
        assert_eq!(
            host.hierarchy_filter_query(),
            value,
            "search route {binding}"
        );
        let filtered = host.filtered_hierarchy_entries(&entries);
        if value.is_empty() {
            assert!(
                filtered.is_none(),
                "clearing search must restore the full projection"
            );
        } else {
            let filtered = filtered.expect("a nonempty query must enter the filter");
            let expected = entries.filtered_by_hierarchy_query(value);
            assert_eq!(
                filtered
                    .iter()
                    .map(|entry| entry.entity)
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|entry| entry.entity)
                    .collect::<Vec<_>>(),
                "search route {binding} must filter the authoring objects"
            );
        }
        host.recompute_if_dirty();
    }
    assert!(
        harness.delta_events_since(journal_baseline).is_empty(),
        "search must not record scene edits"
    );
}
