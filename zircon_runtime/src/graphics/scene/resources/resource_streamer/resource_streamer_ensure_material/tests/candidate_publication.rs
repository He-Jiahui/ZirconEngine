#[test]
fn dependency_execution_failures_keep_last_good_without_suppressing_retry() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");
    let retention = source
        .split("fn retain_last_good_material_after_candidate_failure")
        .nth(1)
        .expect("dependency failure retention");

    assert!(retention.contains("retain_last_good_material_candidate(id, None"));
    assert!(!retention.contains("cache_material_candidate_failure"));
}

#[test]
fn terminal_pipeline_rejection_keeps_the_exact_staged_identity_for_cache_suppression() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");
    let rejection = source
        .split("fn reject_staged_material_pipeline_candidate(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn retain_last_good_material_candidate(")
                .next()
        })
        .expect("pipeline rejection function");

    assert!(rejection.contains("prepared.staged_candidate.as_ref()"));
    assert!(!rejection.contains("prepared.staged_candidate.take()"));
    assert!(rejection.contains("prepared.staged_pipeline_failed = true"));
    assert!(rejection.contains("self.active_staged_material_ids.remove(&id)"));
    assert!(source.contains("self.active_staged_material_ids.insert(id)"));
}

#[test]
fn cold_materials_remain_staged_until_pipeline_publication() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");
    let cold_insert = source
        .split("self.materials.insert(")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn publish_staged").next())
        .expect("cold candidate insertion");

    assert!(cold_insert.contains("published: None"));
    assert!(cold_insert.contains("staged_candidate: Some(candidate)"));
    // BUG: [CR-R02-runtime_wave12_graphics_resource_streamer-0001] 冷材质测试只取插入调用之后的片段；活动登记在片段之前，此处断言必为 false，见本文件冷启动分支的顺序。
    assert!(cold_insert.contains("self.active_staged_material_ids.insert(id)"));
    assert!(source.contains("prepared.published = Some(candidate)"));
    assert!(source.contains("prepared.previous_published = prepared.published.take()"));
}

#[test]
fn candidate_publication_tracks_the_complete_viewport_cycle() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");

    assert!(source.contains("record_staged_material_pipeline_admission"));
    assert!(source.contains("reset_staged_material_pipeline_admission_cycle"));
    assert!(source.contains("finish_staged_material_pipeline_admission_cycle"));
    assert!(source.contains("park_unobserved_staged_material_candidate"));
    assert!(source.contains("staged_pipeline_admission_cycle.record(deferred)"));
    assert!(source.contains("staged_pipeline_admission_cycle.finish()"));
}

#[test]
fn parked_candidate_reactivates_only_when_material_preparation_touches_it_again() {
    let publication_source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");
    let ensure_source = include_str!("../../resource_streamer_ensure_material.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material preparation test boundary");

    assert!(publication_source.contains("park_unobserved_staged_material_candidate"));
    assert!(publication_source.contains("self.active_staged_material_ids.remove(&id)"));
    // BUG: [CR-R02-runtime_wave12_graphics_resource_streamer-0003] 父文件先声明测试子模块，此处取首个测试标记前的片段尚未含准备实现；下面三项检查均为 false，测试首先在此失败。
    assert!(ensure_source.contains("current_slot == PreparedMaterialCacheSlot::Staged"));
    assert!(ensure_source.contains("self.active_staged_material_ids.insert(id)"));
    assert!(ensure_source.contains("material_candidate_reactivated"));
}

#[test]
fn stale_candidate_is_rechecked_before_draw_visible_publication() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");
    let publication = source
        .split("fn publish_staged_material_candidate")
        .nth(1)
        .expect("candidate publication function");

    assert!(publication.contains("material_candidate_publication_stale"));
    assert!(publication.contains("prepared_material_bundle_cache_is_current"));
    assert!(publication.contains("staged_candidate.as_ref()"));
    assert!(
        publication
            .find("prepared_material_bundle_cache_is_current")
            .unwrap()
            < publication.find("staged_candidate.take()").unwrap()
    );
}

#[test]
fn publication_does_not_create_a_permanent_context_admission_owner() {
    let source = include_str!("../candidate_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("candidate publication test boundary");

    assert!(!source.contains("context_admission_material_ids"));
}

#[test]
fn unchanged_non_pipeline_rejection_is_suppressed_before_material_rebuild() {
    let prepared_source = include_str!("../../../prepared/prepared_material.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("prepared material test boundary");
    let ensure_source = include_str!("../../resource_streamer_ensure_material.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material preparation test boundary");
    // BUG: [CR-R02-runtime_wave12_graphics_resource_streamer-0003] 同一父文件片段也缺少缓存探测实现；下方 split 无第二段，expect 必然 panic，尚未进入后续身份断言。
    let cache_probe = ensure_source
        .split("let current_slot =")
        .nth(1)
        .and_then(|source| source.split("if let Some(current_slot)").next())
        .expect("material cache probe");
    let cache_hit = ensure_source
        .split("if let Some(current_slot)")
        .nth(1)
        .and_then(|source| source.split("material_prepare_rebuild").next())
        .expect("material cache-hit handling");

    assert!(prepared_source.contains("PreparedMaterialCandidateIdentity"));
    assert!(prepared_source.contains("identity: Option<PreparedMaterialCandidateIdentity>"));
    assert!(cache_probe.contains("PreparedMaterialCacheSlot::RejectedCandidate"));
    assert!(cache_probe.contains("prepared_material_candidate_cache_is_current"));
    assert!(cache_hit.contains("PreparedMaterialCacheSlot::RejectedCandidate"));
    assert!(cache_hit.contains("return Ok(())"));
}
