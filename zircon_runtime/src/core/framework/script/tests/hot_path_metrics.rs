use super::ScriptHostHotPathMetrics;

#[test]
fn runtime13_context_construction_counters_increase_without_resetting_global_metrics() {
    let before = ScriptHostHotPathMetrics::snapshot();

    ScriptHostHotPathMetrics::record_script_context_weak_handle();
    ScriptHostHotPathMetrics::record_script_context_level_clone();

    let after = ScriptHostHotPathMetrics::snapshot();
    assert!(
        after.script_context_weak_handles >= before.script_context_weak_handles.saturating_add(1),
        "weak-handle construction counter should increase"
    );
    assert!(
        after.script_context_level_clones >= before.script_context_level_clones.saturating_add(1),
        "level-clone construction counter should increase"
    );
}
