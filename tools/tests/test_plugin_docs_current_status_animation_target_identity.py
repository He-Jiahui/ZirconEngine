import unittest

from tools.tests.plugin_status_document import StatusDocumentPath as Path


HISTORICAL_STATUS_ID = "plugins_04_m1_t1_target_identity_and_dense_compile_path_implemented"
HISTORICAL_TEST_STATUS_ID = "plugins_04_m1_t1_target_identity_focused_tests_passed"
REVIEW_STATUS_ID = (
    "plugins_04_m1_t3_production_compiled_evaluator_review_corrections_in_progress"
)
POSE_STATUS_ID = "plugins_04_m1_t2_weighted_pose_formal_cargo_4_of_4_passed"
DOC_PATHS = (
    "docs/plans/zircon_plugins/04-animation.md",
    "docs/plans/zircon_plugins/04/2026-07-10-animation-output-records.md",
    "docs/zircon_plugins/animation-runtime-evaluation.md",
    ".codex/sessions/20260710-0554-plugin12-audit-contract.md",
)
CODE_PATHS = (
    "zircon_runtime/src/core/framework/animation/target_id.rs",
    "zircon_plugins/animation/runtime/src/evaluation/mod.rs",
    "zircon_plugins/animation/runtime/src/evaluation/skeleton_target_table.rs",
    "zircon_plugins/animation/runtime/src/evaluation/target_table.rs",
    "zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/mod.rs",
    "zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compiled_animation_clip.rs",
    "zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs",
    "zircon_plugins/animation/runtime/src/evaluation/pose_buffer/mod.rs",
    "zircon_plugins/animation/runtime/src/evaluation/pose_buffer/pose_buffer.rs",
    "zircon_plugins/animation/runtime/src/evaluation/pose_buffer/blend.rs",
    "zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs",
    "zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/animation_clip_evaluator.rs",
    "zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/channel_validation.rs",
    "zircon_plugins/animation/runtime/src/scene_hook/pose.rs",
    "zircon_runtime/src/core/resource/snapshot.rs",
    "zircon_runtime/tests/resource_snapshot_contract.rs",
    "zircon_plugins/animation/runtime/tests/animation_target_table_contract.rs",
    "zircon_plugins/animation/runtime/tests/animation_pose_buffer_contract.rs",
)


class PluginDocsCurrentStatusAnimationTargetIdentityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.documents = {
            path: (self.repo_root / path).read_text(encoding="utf-8")
            for path in DOC_PATHS
        }

    def test_status_ids_are_mirrored_by_owner_documents(self) -> None:
        combined = "\n".join(self.documents.values())
        for status_id in (
            HISTORICAL_STATUS_ID,
            HISTORICAL_TEST_STATUS_ID,
            REVIEW_STATUS_ID,
            POSE_STATUS_ID,
        ):
            self.assertIn(status_id, combined)

    def test_status_records_current_contract_and_pending_acceptance(self) -> None:
        combined = "\n".join(self.documents.values())
        required_phrases = (
            "AnimationTargetId",
            "SkeletonTargetTable",
            "TargetTable<T>",
            "TargetSlot",
            "CompiledAnimationClip",
            "Arc",
            "DuplicateTrackTarget",
            "weights",
            "10/10",
            "4/4",
            "AnimationClipEvaluator",
            "ResourceSnapshot",
            "payload, revision",
            "remove/re-add",
            "bind-reference",
            "19/19",
            "zero-string",
            "零字符串",
            "零分配",
        )
        missing = [phrase for phrase in required_phrases if phrase not in combined]
        self.assertEqual([], missing)

    def test_status_code_owners_exist(self) -> None:
        missing = [path for path in CODE_PATHS if not (self.repo_root / path).is_file()]
        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
