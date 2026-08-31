import unittest
from pathlib import Path
import re

from tools.runtime_domain_dependency_audit import (
    _rust_code_view,
    _rust_use_paths,
    audit_runtime_domain_dependencies,
)


REPO_ROOT = Path(__file__).resolve().parents[2]


def _exports_animation_clip_event(source: str) -> bool:
    code = _rust_code_view(source)
    visibility = r"\bpub(?:\s*\([^)]*\))?"
    return bool(
        re.search(
            visibility + r"\s+use\b[^;]*\bAnimationClipEvent\b",
            code,
            flags=re.DOTALL,
        )
        or re.search(
            visibility
            + r"\s+(?:type|struct|enum)\s+AnimationClipEvent\b",
            code,
        )
    )


def _references_scene_implementation(source: str) -> bool:
    code = _rust_code_view(source)
    if re.search(r"\bcrate\s*::\s*(?:r#)?scene\b", code):
        return True
    return any(
        len(path) >= 2 and path[:2] == ("crate", "scene")
        for path, _alias, _line in _rust_use_paths(code)
    )


class Frameworks01SceneAnimationBoundaryTests(unittest.TestCase):
    def test_scene_does_not_depend_on_optional_animation_implementation(self) -> None:
        report = audit_runtime_domain_dependencies(REPO_ROOT)
        violations = [
            reference
            for reference in report["references"]
            if reference["source_domain"] == "scene"
            and reference["target_domain"] == "animation"
        ]

        self.assertEqual(
            [],
            violations,
            "scene must consume neutral animation contracts; optional animation "
            f"implementations depend on scene, not the reverse: {violations}",
        )

    def test_animation_manager_uses_neutral_world_identity_without_global_ik_inbox(self) -> None:
        manager_source = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/manager.rs"
        ).read_text(encoding="utf-8")
        manager_code = _rust_code_view(manager_source)

        self.assertIn(
            "use crate::core::framework::scene::WorldHandle;",
            manager_code,
        )
        for retired_symbol in (
            "queue_ik_command",
            "drain_ik_commands",
            "drain_ik_commands_excluding",
        ):
            with self.subTest(retired_symbol=retired_symbol):
                self.assertNotIn(retired_symbol, manager_code)
        self.assertFalse(_references_scene_implementation(manager_source))

    def test_neutral_entity_identity_guard_rejects_scene_path_variants(self) -> None:
        for source in (
            "use crate::scene::{EntityId as ImplementationEntityId};",
            "use crate::{\nscene::{EntityId as ImplementationEntityId},\n};",
            "type ImplementationEntityId = crate :: scene :: EntityId;",
            "use crate::r#scene::EntityId as ImplementationEntityId;",
        ):
            with self.subTest(source=source):
                self.assertTrue(_references_scene_implementation(source))

        self.assertFalse(
            _references_scene_implementation(
                "use crate::core::framework::scene::{EntityId, WorldHandle};\n"
                'const EXAMPLE: &str = "crate::scene::EntityId";\n'
                "/* use crate::{scene::EntityId}; */"
            )
        )


    def test_clip_event_contract_has_no_animation_compatibility_export(self) -> None:
        animation_root = (REPO_ROOT / "zircon_runtime/src/animation/mod.rs").read_text(
            encoding="utf-8"
        )
        scene_level = (REPO_ROOT / "zircon_runtime/src/scene/level_system.rs").read_text(
            encoding="utf-8"
        )
        scene_animation_runtime = (
            REPO_ROOT
            / "zircon_runtime/src/scene/level_system/animation_runtime.rs"
        ).read_text(encoding="utf-8")
        plugin_root = (
            REPO_ROOT / "zircon_plugins/animation/runtime/src/lib.rs"
        ).read_text(encoding="utf-8")
        scene_root = (REPO_ROOT / "zircon_runtime/src/scene/mod.rs").read_text(
            encoding="utf-8"
        )
        sampling_contract = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/clip_event_sampling.rs"
        ).read_text(encoding="utf-8")

        self.assertFalse(_exports_animation_clip_event(animation_root))
        self.assertIn("pub use clip_event::ProjectAnimationClipEventSampler", animation_root)
        self.assertIn("mod animation_runtime;", scene_level)
        self.assertIn("AnimationClipEventSampler", scene_animation_runtime)
        self.assertIn(
            "zircon_runtime::core::framework::animation::{", plugin_root
        )
        self.assertIn("AnimationClipEvent, AnimationSequenceApplyReport", plugin_root)
        self.assertIn("pub enum AnimationClipEventBatchAdmission", sampling_contract)
        self.assertIn("pub enum AnimationClipEventQueueAdmission", sampling_contract)
        self.assertNotIn("AnimationClipEventBatchAdmission", scene_root)
        self.assertNotIn("AnimationClipEventQueueAdmission", scene_root)

    def test_clip_event_compatibility_guard_covers_multiline_and_alias_exports(self) -> None:
        for source in (
            "pub use clip_event::AnimationClipEvent;",
            "pub use crate::core::framework::animation::{\nAnimationClipEvent,\n};",
            "pub(crate) use crate::core::framework::animation::AnimationClipEvent as Legacy;",
            "pub type AnimationClipEvent = framework::AnimationClipEvent;",
        ):
            with self.subTest(source=source):
                self.assertTrue(_exports_animation_clip_event(source))

        self.assertFalse(
            _exports_animation_clip_event(
                "pub use clip_event::ProjectAnimationClipEventSampler;"
            )
        )
        self.assertFalse(
            _exports_animation_clip_event(
                'const EXAMPLE: &str = "pub use clip_event::AnimationClipEvent;";\n'
                "/* pub type AnimationClipEvent = Legacy; */"
            )
        )

    def test_animation_pipeline_world_writes_are_replacement_epoch_guarded(self) -> None:
        pipeline_root = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/evaluation/pipeline"
        )
        for relative_path in (
            "events.rs",
            "parameter_apply.rs",
            "pose_apply.rs",
            "sequences.rs",
            "tick.rs",
        ):
            source = (pipeline_root / relative_path).read_text(encoding="utf-8")
            with self.subTest(path=relative_path):
                self.assertNotIn(".with_world_mut(", source)
                self.assertIn("replacement_epoch", source)

        tick_source = (pipeline_root / "tick.rs").read_text(encoding="utf-8")
        self.assertIn("capture_world_replacement_epoch", tick_source)

    def test_clip_event_backpressure_precedes_player_time_commit(self) -> None:
        pipeline_root = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/evaluation/pipeline"
        )
        tick_code = _rust_code_view(
            (pipeline_root / "tick.rs").read_text(encoding="utf-8")
        )
        parameter_code = _rust_code_view(
            (pipeline_root / "parameter_apply.rs").read_text(encoding="utf-8")
        )
        enqueue_positions = [
            match.start()
            for match in re.finditer(r"\benqueue_clip_event_samples\s*\(", tick_code)
        ]
        apply_positions = [
            match.start()
            for match in re.finditer(r"\bapply_clip_player_updates\s*\(", tick_code)
        ]

        self.assertEqual(2, len(enqueue_positions))
        self.assertEqual(2, len(apply_positions))
        self.assertTrue(
            all(enqueue < apply for enqueue, apply in zip(enqueue_positions, apply_positions))
        )
        self.assertIn("Vec<(EntityId, AnimationPlayerComponent)>", parameter_code)
        self.assertIn("pub(super) fn apply_clip_player_updates", parameter_code)
        scan_clip_players = parameter_code.split("fn scan_clip_players", 1)[1].split(
            "fn scan_sequence_players", 1
        )[0]
        self.assertNotIn("set_animation_player", scan_clip_players)

    def test_clip_event_admission_rotates_and_rolls_back_deferred_entities(self) -> None:
        pipeline_root = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/evaluation/pipeline"
        )
        events_code = _rust_code_view(
            (pipeline_root / "events.rs").read_text(encoding="utf-8")
        )
        tick_code = _rust_code_view(
            (pipeline_root / "tick.rs").read_text(encoding="utf-8")
        )
        pipeline_code = _rust_code_view(
            (pipeline_root / "animation_evaluation_pipeline.rs").read_text(
                encoding="utf-8"
            )
        )
        parameter_code = _rust_code_view(
            (pipeline_root / "parameter_apply.rs").read_text(encoding="utf-8")
        )

        self.assertIn("partition_point", events_code)
        self.assertIn("rotate_left", events_code)
        self.assertIn("enqueue_animation_clip_event_range_batches", events_code)
        transaction_begin = tick_code.index("begin_state_machine_runtime_transaction")
        evaluate = tick_code.rindex("resolve_state_machine_pose_requests")
        journal_finish = tick_code.index("finish_state_machine_runtime_transaction")
        admission = tick_code.rindex("enqueue_clip_event_samples")
        rollback = tick_code.rindex("finish_clip_event_admission")
        playback_commit = tick_code.rindex("record_animation_playback_times")
        self.assertLess(transaction_begin, evaluate)
        self.assertLess(evaluate, journal_finish)
        self.assertLess(journal_finish, admission)
        self.assertLess(admission, rollback)
        self.assertLess(rollback, playback_commit)
        self.assertIn("restore_deferred_state_machine_entities", pipeline_code)
        self.assertNotIn("state_machine_runtime_checkpoint", tick_code)
        self.assertNotIn("StateMachineRuntimeCheckpoint", pipeline_code)
        self.assertNotIn("checkpointed_entities", pipeline_code)
        self.assertIn("restore_deferred_entity_map", tick_code)
        self.assertIn("commit_revision_stage", tick_code)
        self.assertNotIn("revision_checkpoint", tick_code)
        self.assertNotIn("restore_deferred_revisions", tick_code)
        self.assertIn("RejectedOversized", events_code)
        self.assertIn("clip_event_batch_capacity_diagnostic", events_code)
        self.assertIn("AnimationClipEventBatchAdmission::Deferred", events_code)
        self.assertIn("deferred_entities.insert(entity)", events_code)
        self.assertNotIn(
            "AnimationClipEventBatchAdmission::RejectedOversized {\n"
            "range_count,\ncapacity,\n} => {\n"
            "deferred_entities.insert(entity)",
            events_code,
        )
        self.assertIn("apply_sequence_player_updates", tick_code)
        self.assertIn("drain_clip_evaluation_diagnostics_excluding", tick_code)
        self.assertNotIn("drain_ik_commands_excluding", tick_code)
        self.assertIn("replacement_epoch,", tick_code)
        self.assertIn("reset_diagnostics", pipeline_code)
        self.assertIn("retain_non_deferred_entity_updates", tick_code)
        self.assertNotIn("retain_admitted_entity_updates", tick_code)
        self.assertIn("commit_revision_stage(revision_stage", tick_code)
        self.assertIn("sequence_player_updates", parameter_code)
        self.assertIn("sequence_revisions", parameter_code)
        self.assertNotIn("AnimationProjectionRevisionCheckpoint", parameter_code)
        self.assertIn("previous_state_machine_transitions", tick_code)
        self.assertIn("deferred_entities", tick_code)

    def test_pipeline_reset_preserves_reusable_compiled_asset_caches(self) -> None:
        pipeline_source = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs"
        ).read_text(encoding="utf-8")
        pipeline_code = _rust_code_view(pipeline_source)

        self.assertNotIn("direct_clip_worker_evaluators.clear()", pipeline_code)
        self.assertIn(
            "for evaluator in &mut self.direct_clip_worker_evaluators",
            pipeline_code,
        )
        self.assertIn("evaluator.reset_diagnostics()", pipeline_code)
        reset_body = pipeline_code.split("fn reset_evaluation_state", 1)[1].split(
            "pub(super) fn presentation_poses", 1
        )[0]
        self.assertIn("if retire_world_bound_caches", reset_body)
        self.assertEqual(1, reset_body.count("self.sequence_cache.clear()"))

    def test_runtime_animation_parameters_have_content_owned_revision(self) -> None:
        parameter_set_path = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/parameter_set.rs"
        )
        self.assertTrue(
            parameter_set_path.exists(),
            "runtime animation instances require a COW parameter owner independent "
            "from the ECS player component change tick",
        )
        if not parameter_set_path.exists():
            return

        parameter_set = _rust_code_view(
            parameter_set_path.read_text(encoding="utf-8")
        )
        self.assertIn("pub struct AnimationParameterRevision", parameter_set)
        self.assertIn("pub struct AnimationParameterSet", parameter_set)
        self.assertIn("values: Arc<AnimationParameterMap>", parameter_set)
        self.assertIn("revision: AnimationParameterRevision", parameter_set)
        self.assertIn("Arc::make_mut", parameter_set)
        self.assertIn("impl Serialize for AnimationParameterSet", parameter_set)
        self.assertIn("impl<'de> Deserialize<'de> for AnimationParameterSet", parameter_set)
        self.assertNotIn("pub fn synchronize(", parameter_set)
        self.assertNotIn("DerefMut", parameter_set)

    def test_animation_player_schema_and_components_use_revisioned_parameter_set(
        self,
    ) -> None:
        parameter_owners = {
            "scene_schema": REPO_ROOT
            / "zircon_runtime/src/asset/assets/scene/animation.rs",
            "ecs_components": REPO_ROOT
            / "zircon_runtime/src/scene/components/scene/animation.rs",
        }

        for owner, path in parameter_owners.items():
            source = _rust_code_view(path.read_text(encoding="utf-8"))
            with self.subTest(owner=owner):
                self.assertEqual(
                    2,
                    source.count("pub parameters: AnimationParameterSet"),
                )
                self.assertNotIn(
                    "pub parameters: BTreeMap<String, AnimationParameterValue>",
                    source,
                )

    def test_animation_pose_publication_uses_sealed_rows_and_partial_target_delta(self) -> None:
        pose_snapshot_path = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/pose_snapshot.rs"
        )
        self.assertTrue(
            pose_snapshot_path.exists(),
            "the neutral animation boundary must own one named sealed pose snapshot",
        )
        if not pose_snapshot_path.exists():
            return

        pose_snapshot = _rust_code_view(
            pose_snapshot_path.read_text(encoding="utf-8")
        )
        pipeline = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs"
            ).read_text(encoding="utf-8")
        )
        tick = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs"
            ).read_text(encoding="utf-8")
        )
        level_frame = _rust_code_view(
            (
                REPO_ROOT / "zircon_runtime/src/scene/level_system/frame_state.rs"
            ).read_text(encoding="utf-8")
        )
        level_runtime = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_runtime/src/scene/level_system/animation_runtime.rs"
            ).read_text(encoding="utf-8")
        )
        render_pose = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_runtime/src/core/framework/render/frame_extract/skeletal_pose.rs"
            ).read_text(encoding="utf-8")
        )
        history = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_runtime/src/graphics/runtime/history/validation_key.rs"
            ).read_text(encoding="utf-8")
        )
        skeletal_targets = _rust_code_view(
            (
                REPO_ROOT
                / "zircon_runtime/src/core/framework/physics/skeletal_pose.rs"
            ).read_text(encoding="utf-8")
        )

        self.assertIn("pub type AnimationPoseHandle = Arc<AnimationPoseOutput>", pose_snapshot)
        self.assertIn(
            "pub type AnimationPoseMap = BTreeMap<EntityId, AnimationPoseHandle>",
            pose_snapshot,
        )
        self.assertIn("pub type AnimationPoseSnapshot = Arc<AnimationPoseMap>", pose_snapshot)
        self.assertIn("enum PresentationPoseChange", pipeline)
        self.assertIn("Full", pipeline)
        self.assertIn("Partial", pipeline)
        self.assertIn("changed_entities", pipeline)
        self.assertIn("Arc::new(pose)", pipeline)
        self.assertIn("AnimationPoseSnapshot", level_frame)
        self.assertIn("AnimationPoseSnapshot", level_runtime)
        self.assertIn(
            "Arc::ptr_eq(published.animation_poses(), &animation_poses)",
            level_runtime,
        )
        self.assertNotIn(
            "published.animation_poses().as_ref() == animation_poses.as_ref()",
            level_runtime,
        )
        self.assertIn("pub pose: AnimationPoseHandle", render_pose)
        self.assertIn("pose: AnimationPoseHandle", history)
        self.assertIn(
            "impl PartialEq for FrameHistoryAnimationPoseValidationKey",
            history,
        )
        self.assertIn("Arc::ptr_eq(&self.pose, &other.pose)", history)
        self.assertIn("PresentationPoseChange::Partial", tick)
        self.assertIn("targets.remove", tick)
        self.assertIn("pub fn remove", skeletal_targets)
        self.assertNotIn("pub pose: AnimationPoseOutput", render_pose)
        self.assertNotIn(
            "Arc<BTreeMap<EntityId, AnimationPoseOutput>>",
            pipeline + level_frame + level_runtime,
        )

    def test_animation_runtime_lowers_shared_compiler_ir_without_legacy_compilers(self) -> None:
        runtime_root = REPO_ROOT / "zircon_plugins/animation/runtime"
        source_root = runtime_root / "src"
        graph_compile = (source_root / "evaluation/compiled_graph/compile.rs").read_text(
            encoding="utf-8"
        )
        graph_evaluate = (
            source_root / "evaluation/compiled_graph/evaluate.rs"
        ).read_text(encoding="utf-8")
        graph_types = (source_root / "evaluation/compiled_graph/types.rs").read_text(
            encoding="utf-8"
        )
        graph_contract = (
            runtime_root / "tests/animation_compiled_graph_contract.rs"
        ).read_text(encoding="utf-8")
        shared_graph_compile = (
            REPO_ROOT / "zircon_runtime/src/core/framework/animation/compiler/graph.rs"
        ).read_text(encoding="utf-8")
        shared_graph_node = shared_graph_compile.split(
            "pub enum AnimationCompiledGraphNode", 1
        )[1].split("pub struct AnimationCompiledGraph", 1)[0]
        graph_cache = (source_root / "evaluation/pipeline/graph_cache.rs").read_text(
            encoding="utf-8"
        )
        parameter_set = (
            REPO_ROOT / "zircon_runtime/src/core/framework/animation/parameter_set.rs"
        ).read_text(encoding="utf-8")
        state_compile = (source_root / "state_machine/compiled/compile.rs").read_text(
            encoding="utf-8"
        )
        state_cache = (
            source_root / "evaluation/pipeline/state_machine_cache.rs"
        ).read_text(encoding="utf-8")
        pipeline = (
            source_root / "evaluation/pipeline/animation_evaluation_pipeline.rs"
        ).read_text(encoding="utf-8")
        nested_machine_resolve = (
            source_root / "evaluation/pipeline/nested_machine_resolve.rs"
        ).read_text(encoding="utf-8")
        state_graph_sample = (
            source_root / "evaluation/pipeline/state_graph_sample.rs"
        ).read_text(encoding="utf-8")
        state_machine_step = (
            source_root / "evaluation/pipeline/state_machine_step.rs"
        ).read_text(encoding="utf-8")
        state_machine_layers = (
            source_root / "evaluation/pipeline/state_machine_layers.rs"
        ).read_text(encoding="utf-8")
        parameter_apply = (
            source_root / "evaluation/pipeline/parameter_apply.rs"
        ).read_text(encoding="utf-8")
        requests = (
            source_root / "evaluation/pipeline/requests.rs"
        ).read_text(encoding="utf-8")
        compiled_state = (
            source_root / "state_machine/compiled/compiled_state.rs"
        ).read_text(encoding="utf-8")
        compiled_machine = (
            source_root
            / "state_machine/compiled/compiled_animation_state_machine.rs"
        ).read_text(encoding="utf-8")
        compiled_evaluate = (
            source_root / "state_machine/compiled/evaluate.rs"
        ).read_text(encoding="utf-8")
        runtime_journal = pipeline.split(
            "pub(super) struct StateMachineRuntimeJournal", 1
        )[1].split("pub struct AnimationEvaluationProjectionStats", 1)[0]
        instance_cache = state_cache.split(
            "impl StateMachineInstanceCache", 1
        )[1].split("impl AnimationEvaluationPipeline", 1)[0]
        blend_compile = (
            source_root / "state_machine/blend_space/blend_space_2d.rs"
        ).read_text(encoding="utf-8")
        blend_geometry = (
            source_root / "state_machine/blend_space/geometry.rs"
        ).read_text(encoding="utf-8")
        manifest = (runtime_root / "Cargo.toml").read_text(encoding="utf-8")
        all_runtime_source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in source_root.rglob("*.rs")
        )

        self.assertIn("compile_animation_graph(source)", graph_compile)
        self.assertIn("weight_parameter: Option<usize>", shared_graph_compile)
        self.assertNotIn("id: String", shared_graph_node)
        self.assertNotIn("BTreeMap", graph_compile)
        self.assertNotIn("fn resolve_parameter", graph_compile)
        self.assertIn("compile_animation_graph_runtime(&graph, targets)", graph_cache)
        self.assertIn("AnimationParameterContentFingerprint", parameter_set)
        self.assertIn("content_fingerprint: AnimationParameterContentFingerprint", parameter_set)
        self.assertIn("pub fn content_fingerprint", parameter_set)
        self.assertIn("self.content_fingerprint != other.content_fingerprint", parameter_set)
        self.assertIn("graph_evaluation_cache: BTreeMap", pipeline)
        self.assertNotIn("VecDeque<CachedGraphEvaluation>", pipeline)
        self.assertIn("parameters.content_fingerprint()", graph_cache)
        self.assertIn(".get(&cache_key)", graph_cache)
        self.assertIn(".entry(cache_key)", pipeline)
        self.assertNotIn("graph_evaluation_cache.iter().find", graph_cache)
        self.assertNotIn("graph_evaluation_cache.pop_front", pipeline)
        self.assertIn(
            "graph_evaluation_frame_cache_reuses_equal_content_and_separates_distinct_content",
            pipeline,
        )
        self.assertIn("evaluation_order: Box<[GraphNodeSlot]>", graph_types)
        self.assertIn("GraphContextWeights", graph_evaluate)
        self.assertIn("self.evaluation_order.iter().rev()", graph_evaluate)
        self.assertNotIn("fn collect_clips", graph_evaluate)
        self.assertNotIn("collect_clips(", graph_evaluate)
        self.assertIn(
            "compiled_graph_diamond_aggregates_shared_clip_node_once",
            graph_contract,
        )
        self.assertIn(
            "compiled_graph_evaluation_is_non_recursive_for_deep_chain",
            graph_contract,
        )
        self.assertIn("compile_animation_state_machine(source)", state_compile)
        self.assertIn(
            "compile_animation_state_machine_runtime_bundle(&source)",
            state_cache,
        )
        self.assertNotIn("CompiledAnimationGraph::compile", all_runtime_source)
        self.assertNotIn("CompiledAnimationStateMachine::compile", all_runtime_source)
        self.assertNotIn("BlendSpacePoint1D", all_runtime_source)
        self.assertNotIn("BlendSpacePoint2D", all_runtime_source)
        self.assertFalse(
            (source_root / "state_machine/blend_space/blend_space_point.rs").exists()
        )
        self.assertIn('spade = "2.15.1"', manifest)
        self.assertIn("bulk_load_stable", blend_compile)
        self.assertIn("neighbors: Box<[[Option<usize>; 3]]>", blend_compile)
        self.assertIn("hull_edges: Box<[[usize; 2]]>", blend_compile)
        self.assertIn("StateMachineBlendSamplingState", compiled_state)
        self.assertIn("triangle_hints: Box<[Option<usize>]>", compiled_state)
        self.assertIn("evaluate_with_blend_sampling", compiled_evaluate)
        self.assertIn("STATE_MACHINE_INSTANCE_CACHE_LIMIT", state_cache)
        self.assertIn("AnimationParameterRevision", requests)
        self.assertEqual(2, requests.count("parameters: AnimationParameterSet"))
        self.assertNotIn("StateMachineParameterProjectionRevision", requests)
        self.assertNotIn("parameter_projection_revision", requests)
        state_machine_scan = parameter_apply.split(
            "fn scan_state_machine_players", 1
        )[1]
        self.assertNotIn("StateMachineParameterProjectionRevision", state_machine_scan)
        self.assertEqual(
            2,
            parameter_apply.count("parameters: player.parameters.clone()"),
        )
        self.assertNotIn("parameters.synchronize(&player.parameters)", parameter_apply)
        self.assertNotIn("graph_parameter_snapshots", parameter_apply)
        self.assertNotIn("state_machine_parameter_snapshots", parameter_apply)
        self.assertIn("parameter_names: Arc<[String]>", compiled_machine)
        self.assertIn("StateMachineParameterValues", compiled_evaluate)
        self.assertNotIn("fn parameter_values", compiled_evaluate)
        self.assertIn("parameter_layout: Arc<[String]>", state_cache)
        self.assertIn("parameter_revision: AnimationParameterRevision", state_cache)
        self.assertIn("parameter_values: Box<[Option<AnimationParameterValue>]>", state_cache)
        self.assertIn("Arc::ptr_eq", state_cache)
        self.assertIn("machine.project_parameters", state_cache)
        self.assertIn("StateMachineEvaluationResult", state_cache)
        self.assertNotIn("AnimationStateMachineEvaluation", state_cache)
        self.assertNotIn("parameters.values.clone()", state_cache)
        self.assertIn("retain_entities", state_cache)
        self.assertIn("eviction_clock: VecDeque", state_cache)
        self.assertIn(".pop_front()", instance_cache)
        self.assertNotIn(".min_by_key", instance_cache)
        self.assertNotIn("eviction_order", instance_cache)
        self.assertIn("state_machine_instance_cache", pipeline)
        self.assertIn("retain_entities(active_entities)", pipeline)
        self.assertIn("previous_by_instance: BTreeMap", runtime_journal)
        self.assertIn("previous_interrupted_transition_source", runtime_journal)
        self.assertIn("previous_nested_machine_state", runtime_journal)
        self.assertIn("previous_nested_machine_transition", runtime_journal)
        self.assertNotIn("checkpointed_entities", runtime_journal)
        self.assertNotIn("sampling_cache", runtime_journal)
        self.assertNotIn("instance_cache", runtime_journal)
        state_machine_writers = state_machine_step + "\n" + state_machine_layers
        for direct_write in (
            r"\.nested_machine_states\s*\.\s*insert",
            r"\.nested_machine_transitions\s*\.\s*insert",
            r"\.nested_machine_transitions\s*\.\s*remove",
            r"pipeline\s*\.\s*record_interrupted_transition_source\s*\(",
            r"pipeline\s*\.\s*clear_interrupted_transition_source\s*\(",
        ):
            self.assertNotRegex(state_machine_writers, direct_write)
        self.assertIn("set_nested_machine_state", state_machine_writers)
        self.assertIn("set_nested_machine_transition", state_machine_writers)
        self.assertIn("clear_nested_machine_transition", state_machine_writers)
        self.assertIn(
            "record_state_machine_interrupted_transition_source",
            state_machine_writers,
        )
        self.assertIn(
            "clear_state_machine_interrupted_transition_source",
            state_machine_writers,
        )
        self.assertIn("state_machine_instance_cache.clear()", pipeline)
        self.assertIn("parameters: AnimationParameterSet", pipeline)
        self.assertNotIn("AnimationParameterMap", pipeline)
        self.assertIn("parameters: &AnimationParameterSet", graph_cache)
        self.assertIn("cached.parameters == *parameters", graph_cache)
        self.assertIn("&instance,", nested_machine_resolve)
        self.assertIn("instance: &MachineInstanceKey", state_graph_sample)
        self.assertIn("graph_samples_for_state_with_sampling", state_graph_sample)
        self.assertNotIn(".graph_samples_for_state(", state_graph_sample)
        self.assertIn("pub(super) fn project_to_segment", blend_geometry)
        self.assertIn("-> (f64, Real)", blend_geometry)
        self.assertIn("(denominator != 0.0).then", blend_geometry)
        self.assertNotIn("denominator.abs() > Real::EPSILON", blend_geometry)

    def test_shared_compiler_blend_space_validation_is_bounded_and_scale_independent(
        self,
    ) -> None:
        compiler = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/compiler/state_machine/compile.rs"
        ).read_text(encoding="utf-8")
        manifest = (REPO_ROOT / "zircon_runtime/Cargo.toml").read_text(
            encoding="utf-8"
        )
        collinearity = compiler.split(
            "fn contains_non_collinear_points", 1
        )[1].split("fn compile_transitions", 1)[0]

        self.assertIn('robust = "1.2.0"', manifest)
        self.assertIn("BTreeSet::new()", compiler)
        self.assertIn("canonical_real_bits", compiler)
        self.assertIn("orient2d(coord(first), coord(second), coord(third))", collinearity)
        self.assertNotIn("positions.contains", compiler)
        self.assertNotIn("for first in", collinearity)
        self.assertNotIn("for second in", collinearity)
        self.assertNotIn("for third in", collinearity)
        self.assertNotIn("Real::EPSILON", collinearity)

    def test_runtime_blend_space_retains_location_across_hull_sampling(self) -> None:
        blend_space = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_2d.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("enum TriangleWalk", blend_space)
        self.assertIn("TriangleWalk::OutsideHull", blend_space)
        self.assertIn("self.triangles.len() / 2", blend_space)
        self.assertNotIn(
            "self.sample_hull(point).map(|weights| (weights, None))", blend_space
        )


if __name__ == "__main__":
    unittest.main()
