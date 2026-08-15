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

    def test_animation_manager_uses_neutral_scene_entity_identity(self) -> None:
        manager_source = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/manager.rs"
        ).read_text(encoding="utf-8")
        manager_code = _rust_code_view(manager_source)

        self.assertIn(
            "use crate::core::framework::scene::{EntityId, WorldHandle};",
            manager_code,
        )
        self.assertIn("deferred_entities: &[EntityId]", manager_code)
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
        checkpoint = tick_code.index("state_machine_runtime_checkpoint")
        evaluate = tick_code.rindex("resolve_state_machine_pose_requests")
        admission = tick_code.rindex("enqueue_clip_event_samples")
        rollback = tick_code.rindex("finish_clip_event_admission")
        playback_commit = tick_code.rindex("record_animation_playback_times")
        self.assertLess(checkpoint, evaluate)
        self.assertLess(evaluate, admission)
        self.assertLess(admission, rollback)
        self.assertLess(rollback, playback_commit)
        self.assertIn("restore_deferred_state_machine_entities", pipeline_code)
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
        self.assertIn("drain_ik_commands_excluding", tick_code)
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


if __name__ == "__main__":
    unittest.main()
