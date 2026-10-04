use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationParameterSet, AnimationParameterValue, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::scene::AnimationStateTransitionRuntime;

use crate::{AnimationAssetRevision, AnimationEvaluationError, CompiledAnimationGraphEvaluation};

use super::{
    AnimationEvaluationPipeline, MachineInstanceKey, PresentationPoseChange,
    GRAPH_EVALUATION_FRAME_CACHE_LIMIT,
};

fn pose() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: Vec::new(),
    }
}

#[test]
fn replacement_epoch_and_empty_mode_prepare_without_duplicate_resets() {
    let mut pipeline = AnimationEvaluationPipeline::default();
    pipeline
        .presentation_poses
        .make_mut()
        .insert(17, Arc::new(pose()));

    assert!(pipeline.begin_evaluation_frame(1));
    assert!(pipeline.presentation_poses.is_empty());

    pipeline
        .presentation_poses
        .make_mut()
        .insert(18, Arc::new(pose()));
    assert!(!pipeline.begin_evaluation_frame(1));
    assert!(pipeline.presentation_poses.contains_key(&18));

    assert!(pipeline.ensure_empty_evaluation_state(1));
    assert!(pipeline.presentation_poses.is_empty());
    assert!(!pipeline.ensure_empty_evaluation_state(1));
    assert!(!pipeline.begin_evaluation_frame(1));

    assert!(pipeline.begin_evaluation_frame(2));
    assert!(pipeline.presentation_poses.is_empty());
}

#[test]
fn partial_pose_publication_reuses_retained_rows_and_reports_exact_delta() {
    let mut pipeline = AnimationEvaluationPipeline::default();
    let initial = pipeline
        .update_presentation_poses(
            &BTreeSet::from([17, 18]),
            BTreeMap::from([(17, pose()), (18, pose())]),
        )
        .expect("initial full publication changes the snapshot");
    assert_eq!(initial.change, PresentationPoseChange::Full);
    let retained = Arc::clone(initial.snapshot.get(&17).unwrap());

    let changed = pipeline
        .update_presentation_poses(
            &BTreeSet::from([17, 18]),
            BTreeMap::from([(
                18,
                AnimationPoseOutput {
                    active_state: Some("Run".into()),
                    ..pose()
                },
            )]),
        )
        .expect("one changed entity publishes a partial snapshot");
    assert_eq!(
        changed.change,
        PresentationPoseChange::Partial {
            changed_entities: Box::new([18]),
        }
    );
    assert!(Arc::ptr_eq(changed.snapshot.get(&17).unwrap(), &retained));

    let removed = pipeline
        .update_presentation_poses(&BTreeSet::from([17]), BTreeMap::new())
        .expect("retiring a source publishes its removal");
    assert_eq!(
        removed.change,
        PresentationPoseChange::Partial {
            changed_entities: Box::new([18]),
        }
    );
    assert!(Arc::ptr_eq(removed.snapshot.get(&17).unwrap(), &retained));
    assert!(!removed.snapshot.contains_key(&18));
}

#[test]
fn graph_evaluation_frame_cache_stops_admission_at_capacity() {
    let mut pipeline = AnimationEvaluationPipeline::default();
    let parameters = AnimationParameterSet::default();

    for index in 0..=GRAPH_EVALUATION_FRAME_CACHE_LIMIT {
        let label = format!("animation.graph.{index}");
        let graph_id = ResourceId::from_stable_label(&label);
        pipeline.cache_graph_evaluation(
            graph_id,
            graph_id,
            &parameters,
            Arc::new(CompiledAnimationGraphEvaluation::default()),
        );
    }

    assert_eq!(
        pipeline.graph_evaluation_cache.len(),
        GRAPH_EVALUATION_FRAME_CACHE_LIMIT
    );
    let first = ResourceId::from_stable_label("animation.graph.0");
    let rejected = ResourceId::from_stable_label(&format!(
        "animation.graph.{GRAPH_EVALUATION_FRAME_CACHE_LIMIT}"
    ));
    assert!(pipeline.graph_evaluation_cache.contains_key(&(
        first,
        first,
        parameters.content_fingerprint()
    )));
    assert!(!pipeline.graph_evaluation_cache.contains_key(&(
        rejected,
        rejected,
        parameters.content_fingerprint()
    )));
}

#[test]
fn graph_evaluation_frame_cache_reuses_equal_content_and_separates_distinct_content() {
    let graph = ResourceId::from_stable_label("animation.graph.content-index");
    let skeleton = ResourceId::from_stable_label("animation.skeleton.content-index");
    let parameters =
        AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
    let equal_content = AnimationParameterSet::from(parameters.as_map().clone());
    let distinct_content =
        AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.75))]);
    let first_evaluation = Arc::new(CompiledAnimationGraphEvaluation::default());
    let replacement_evaluation = Arc::new(CompiledAnimationGraphEvaluation::default());
    let distinct_evaluation = Arc::new(CompiledAnimationGraphEvaluation::default());
    let mut pipeline = AnimationEvaluationPipeline::default();

    pipeline.cache_graph_evaluation(graph, skeleton, &parameters, Arc::clone(&first_evaluation));
    pipeline.cache_graph_evaluation(graph, skeleton, &equal_content, replacement_evaluation);

    assert_eq!(pipeline.graph_evaluation_cache.len(), 1);
    let equal_key = (graph, skeleton, equal_content.content_fingerprint());
    let cached = pipeline
        .graph_evaluation_cache
        .get(&equal_key)
        .expect("equal parameter content reuses the indexed entry");
    assert_eq!(cached.parameters, equal_content);
    assert!(Arc::ptr_eq(&cached.evaluation, &first_evaluation));

    pipeline.cache_graph_evaluation(graph, skeleton, &distinct_content, distinct_evaluation);
    assert_ne!(
        parameters.content_fingerprint(),
        distinct_content.content_fingerprint()
    );
    assert_eq!(pipeline.graph_evaluation_cache.len(), 2);
}

#[test]
fn graph_evaluation_frame_cache_source_uses_bounded_content_index() {
    let source = include_str!("../animation_evaluation_pipeline.rs");
    let tests = source.find("#[cfg(test)]").expect("test module boundary");
    let production = &source[..tests];
    let start = source
        .find("    pub(super) fn cache_graph_evaluation(")
        .expect("graph evaluation cache insertion owner");
    let end = source[start..]
        .find("    pub(super) fn update_presentation_poses(")
        .map(|offset| start + offset)
        .expect("graph evaluation cache insertion boundary");
    let insertion = &source[start..end];

    assert!(production.contains("BTreeMap<GraphEvaluationCacheKey, CachedGraphEvaluation>"));
    assert!(insertion.contains("parameters.content_fingerprint()"));
    assert!(insertion.contains(".entry(cache_key)"));
    assert!(insertion.contains(".or_insert_with(|| CachedGraphEvaluation"));
    assert!(!insertion.contains("pop_front()"));
    assert!(!insertion.contains("remove(0)"));
}

#[test]
fn replacement_epoch_retires_pending_diagnostics_from_all_evaluators() {
    let skeleton = AnimationAssetRevision::new(
        ResourceId::from_stable_label("animation.skeleton.replacement-diagnostic"),
        1,
    );
    let clip = AnimationAssetRevision::new(
        ResourceId::from_stable_label("animation.clip.replacement-diagnostic"),
        1,
    );
    let mut pipeline = AnimationEvaluationPipeline::default();
    assert!(pipeline.begin_evaluation_frame(1));
    pipeline.clip_evaluator_mut().record_diagnostic(
        17,
        skeleton,
        clip,
        AnimationEvaluationError::MissingPreparedClip {
            skeleton: skeleton.id(),
            clip: clip.id(),
        },
    );
    let mut worker = pipeline.take_direct_clip_worker_evaluator(1);
    worker.record_diagnostic(
        18,
        skeleton,
        clip,
        AnimationEvaluationError::MissingPreparedClip {
            skeleton: skeleton.id(),
            clip: clip.id(),
        },
    );
    pipeline.restore_direct_clip_worker_evaluator(1, worker);

    assert!(pipeline.begin_evaluation_frame(2));
    assert_eq!(pipeline.direct_clip_worker_evaluators.len(), 1);
    assert!(pipeline
        .drain_clip_evaluation_diagnostics_excluding(&BTreeSet::new())
        .is_empty());

    pipeline.clip_evaluator_mut().record_diagnostic(
        19,
        skeleton,
        clip,
        AnimationEvaluationError::MissingPreparedClip {
            skeleton: skeleton.id(),
            clip: clip.id(),
        },
    );
    let current = pipeline.drain_clip_evaluation_diagnostics_excluding(&BTreeSet::new());
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].entity, 19);
}

#[test]
fn deferred_event_entities_restore_state_machine_runtime_first_write_journal() {
    let machine = ResourceId::from_stable_label("animation.machine.journal");
    let admitted = MachineInstanceKey::root(17, machine);
    let deferred = MachineInstanceKey::root(18, machine);
    let unchanged = MachineInstanceKey::root(19, machine);
    let inserted_then_deferred = MachineInstanceKey::root(20, machine);
    let mut pipeline = AnimationEvaluationPipeline::default();
    pipeline
        .nested_machine_states
        .insert(admitted.clone(), "OldA".into());
    pipeline
        .nested_machine_states
        .insert(deferred.clone(), "OldB".into());
    pipeline
        .nested_machine_states
        .insert(unchanged.clone(), "Paused".into());
    pipeline.nested_machine_transitions.insert(
        deferred.clone(),
        AnimationStateTransitionRuntime {
            from_state: "OldB".into(),
            to_state: "OldC".into(),
            duration_seconds: 1.0,
            elapsed_seconds: 0.25,
            from_time_seconds: 0.25,
            to_time_seconds: 0.0,
        },
    );
    pipeline.record_interrupted_transition_source(deferred.clone(), "OldB", "OldC", pose());
    pipeline.begin_state_machine_runtime_transaction(&BTreeSet::from([17, 18, 19, 20]));

    pipeline.set_nested_machine_state(admitted.clone(), "NewA".into());
    pipeline.set_nested_machine_state(deferred.clone(), "NewB".into());
    pipeline.clear_nested_machine_transition(&deferred);
    pipeline.clear_state_machine_interrupted_transition_source(&deferred);
    pipeline.set_nested_machine_state(inserted_then_deferred.clone(), "Transient".into());
    pipeline.set_nested_machine_transition(
        inserted_then_deferred.clone(),
        AnimationStateTransitionRuntime {
            from_state: "Transient".into(),
            to_state: "Discarded".into(),
            duration_seconds: 0.5,
            elapsed_seconds: 0.1,
            from_time_seconds: 0.1,
            to_time_seconds: 0.0,
        },
    );
    pipeline.record_state_machine_interrupted_transition_source(
        inserted_then_deferred.clone(),
        "Transient",
        "Discarded",
        pose(),
    );
    let journal = pipeline.finish_state_machine_runtime_transaction();
    pipeline.restore_deferred_state_machine_entities(
        journal,
        &BTreeSet::from([
            deferred.entity(),
            unchanged.entity(),
            inserted_then_deferred.entity(),
        ]),
    );

    assert_eq!(
        pipeline
            .nested_machine_states
            .get(&admitted)
            .map(String::as_str),
        Some("NewA")
    );
    assert_eq!(
        pipeline
            .nested_machine_states
            .get(&deferred)
            .map(String::as_str),
        Some("OldB")
    );
    assert!(pipeline.nested_machine_transitions.contains_key(&deferred));
    assert!(pipeline
        .interrupted_transition_source(&deferred, "OldB", "OldC")
        .is_some());
    assert_eq!(
        pipeline
            .nested_machine_states
            .get(&unchanged)
            .map(String::as_str),
        Some("Paused")
    );
    assert!(!pipeline
        .nested_machine_states
        .contains_key(&inserted_then_deferred));
    assert!(!pipeline
        .nested_machine_transitions
        .contains_key(&inserted_then_deferred));
    assert!(pipeline
        .interrupted_transition_source(&inserted_then_deferred, "Transient", "Discarded")
        .is_none());
}
