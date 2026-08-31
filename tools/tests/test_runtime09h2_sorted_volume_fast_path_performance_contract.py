from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EVALUATOR_RS = ROOT / (
    "zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs"
)
SCENE_PRODUCER_RS = ROOT / (
    "zircon_runtime/src/scene/world/render_post_process.rs"
)


def evaluate_body() -> str:
    source = EVALUATOR_RS.read_text(encoding="utf-8")
    return source.split("pub fn evaluate(", 1)[1].split("fn apply_volume", 1)[0]


class Runtime09H2SortedVolumeFastPathContract(unittest.TestCase):
    def test_sorted_branch_precedes_materialized_fallback(self) -> None:
        body = evaluate_body()

        branch = body.index("if volumes_are_priority_sorted(request.volumes)")
        fallback = body.index("let mut applicable")
        self.assertLess(branch, fallback)

    def test_sorted_branch_applies_without_collecting_candidates(self) -> None:
        body = evaluate_body()
        sorted_branch = body.split("let mut applicable", 1)[0]

        self.assertIn("for volume in request.volumes", sorted_branch)
        self.assertIn("applicable_volume_influence(", sorted_branch)
        self.assertIn("self.apply_volume(&mut settings, volume, influence)?", sorted_branch)
        self.assertNotIn(".collect", sorted_branch)

    def test_unsorted_fallback_retains_stable_priority_sort(self) -> None:
        body = evaluate_body().split("let mut applicable", 1)[1]

        self.assertIn(".enumerate()", body)
        self.assertIn(".collect::<Vec<_>>()", body)
        self.assertIn("applicable.sort_by(compare_volume_priority)", body)

    def test_scene_producer_publishes_priority_order(self) -> None:
        source = SCENE_PRODUCER_RS.read_text(encoding="utf-8")
        producer = source.split("extracts.sort_by", 1)[1].split(
            "CollectedPostProcessVolumes", 1
        )[0]

        self.assertIn("left.priority", producer)
        self.assertIn("partial_cmp(&right.priority)", producer)
        self.assertIn("extracts.into_iter().map", source)


if __name__ == "__main__":
    unittest.main()
