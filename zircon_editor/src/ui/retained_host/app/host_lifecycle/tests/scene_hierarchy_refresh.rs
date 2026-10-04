#[test]
fn fragment_failures_request_an_authoritative_reflow() {
    let source = include_str!("../scene_hierarchy_refresh.rs");
    for failure in [
        "Hierarchy fragment apply failed: {error}",
        "Hierarchy selection resync failed: {error}",
    ] {
        let failure = source
            .find(failure)
            .unwrap_or_else(|| panic!("missing hierarchy failure branch: {failure}"));
        let reflow = source[failure..]
            .find("self.resync_scene_hierarchy_from_message(message);")
            .unwrap_or_else(|| panic!("{failure} must request an authoritative reflow"));
        assert!(
            reflow < 256,
            "failure recovery must stay in the same match arm"
        );
    }
}

#[test]
fn selection_gap_retries_the_same_sparse_fragment_before_publishing() {
    let production = include_str!("../scene_hierarchy_refresh.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("hierarchy refresh production section");
    let selection_recovery = production
        .split("if apply.selection_resync_required()")
        .nth(1)
        .expect("selection-gap recovery branch");
    let recovery_end = selection_recovery
        .find("if let Some(entries) = reflow_entries")
        .expect("selection-gap recovery must finish before ordinary reflow handling");
    let selection_recovery = &selection_recovery[..recovery_end];
    let selection_resync = selection_recovery
        .find(".resync_scene_hierarchy_selection(")
        .expect("selection-gap recovery must repair the selection overlay");
    let fragment_retry = selection_recovery[selection_resync..]
        .find(".apply_scene_hierarchy_fragment(&fragment)")
        .expect("selection-gap recovery must retry the interrupted sparse fragment");
    let sparse_publish = selection_recovery[selection_resync..]
        .find("self.publish_sparse_hierarchy_host_nodes")
        .expect("selection-gap recovery must publish the repaired sparse projection");
    let revision_mismatch = selection_recovery[selection_resync..]
        .find("selection_revision != message.selection().revision()")
        .expect("selection-gap recovery must reject an advanced snapshot");
    let mismatch_recovery = &selection_recovery[selection_resync + revision_mismatch..];
    let mismatch_reflow = mismatch_recovery
        .find("self.resync_scene_hierarchy_from_message(message);")
        .expect("advanced snapshot must use authoritative reflow");
    let mismatch_return = mismatch_recovery[mismatch_reflow..]
        .find("return;")
        .expect("advanced snapshot reflow must terminate the branch");
    let merged_controls = selection_recovery[selection_resync..]
        .find(".extend(fragment_apply.changed_control_ids().iter().cloned())")
        .expect("selection and fragment control ids must merge before publication");

    assert!(revision_mismatch < fragment_retry);
    assert!(mismatch_reflow < mismatch_return);
    assert!(merged_controls < sparse_publish);
    assert_eq!(
        selection_recovery
            .matches("self.publish_sparse_hierarchy_host_nodes")
            .count(),
        1,
        "selection-gap recovery must publish only the merged sparse patch"
    );
    assert!(
        fragment_retry < sparse_publish,
        "the row patch must commit before its sparse host publication"
    );
}

#[test]
fn authoritative_reflow_commits_host_and_pointer_state_after_bridge_success() {
    let source = include_str!("../scene_hierarchy_refresh.rs");
    let reflow = source
        .split("fn commit_scene_hierarchy_reflow")
        .nth(1)
        .expect("authoritative reflow function");
    let bridge = reflow
        .find(".resync_scene_hierarchy_at_selection(")
        .expect("bridge resync must be attempted first");
    let presentation = reflow
        .find("self.ui.set_host_presentation(presentation);")
        .expect("host presentation must be committed");
    let pointer = reflow
        .find("self.sync_hierarchy_pointer_layout(entries.hierarchy_rows_arc());")
        .expect("pointer routing must be committed");
    assert!(bridge < presentation);
    assert!(presentation < pointer);
}

#[test]
fn failed_authoritative_reflow_marks_the_layout_for_the_next_recovery_attempt() {
    let source = include_str!("../scene_hierarchy_refresh.rs");
    let reflow = source
        .split("fn commit_scene_hierarchy_reflow")
        .nth(1)
        .expect("authoritative reflow function");
    let failure = reflow
        .find("Hierarchy authoritative reflow failed: {error}")
        .expect("authoritative reflow failure branch");
    let mark_dirty = reflow[failure..]
        .find("self.mark_layout_dirty();")
        .expect("failed authoritative reflow must schedule recovery");
    let return_statement = reflow[failure..]
        .find("return;")
        .expect("authoritative reflow failure return");
    assert!(mark_dirty < return_statement);
}

#[test]
fn active_hierarchy_filters_force_one_filtered_authoritative_projection() {
    let source = include_str!("../scene_hierarchy_refresh.rs");
    let apply = source
        .split("fn apply_scene_hierarchy_fragment")
        .nth(1)
        .expect("fragment apply function");
    let filter_gate = apply
        .find("self.hierarchy_filter_query().trim().is_empty()")
        .expect("active hierarchy filters must gate sparse patches");
    let sparse_publish = apply
        .find("self.publish_sparse_hierarchy_host_nodes")
        .expect("sparse publication path");
    assert!(filter_gate < sparse_publish);

    let reflow = source
        .split("fn commit_scene_hierarchy_reflow")
        .nth(1)
        .expect("authoritative reflow function");
    let filtered_entries = reflow
        .find("self.filtered_hierarchy_entries(entries)")
        .expect("authoritative projection must apply the active filter once");
    let bridge = reflow
        .find(".resync_scene_hierarchy_at_selection(")
        .expect("filtered bridge resync");
    let pointer = reflow
        .find("self.sync_hierarchy_pointer_layout(entries.hierarchy_rows_arc());")
        .expect("filtered pointer commit");
    assert!(filtered_entries < bridge);
    assert!(bridge < pointer);
}

#[test]
fn play_world_replacement_retires_derived_state_before_querying_the_new_hierarchy() {
    let source = include_str!("../scene_hierarchy_refresh.rs");
    let sync = source
        .split("fn sync_active_hierarchy_world")
        .nth(1)
        .expect("active hierarchy sync function");
    let edge = sync
        .find("advanced_world_replacement_epoch")
        .expect("world replacement edge");
    let pick = sync
        .find("play_viewport_pick.cancel")
        .expect("pending pick retirement");
    let retire = sync
        .find("retire_replaced_play_world")
        .expect("central Play retirement");
    let acknowledge = sync
        .find("acknowledge_world_replacement")
        .expect("replacement acknowledgement");
    let query = sync
        .find("query_play_hierarchy_fragment")
        .expect("new hierarchy query");

    assert!(edge < pick);
    assert!(pick < retire);
    assert!(retire < acknowledge);
    assert!(acknowledge < query);
}
