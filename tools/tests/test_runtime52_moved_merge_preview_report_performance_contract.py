from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / "zircon_runtime/src/scene/dynamic_scene/session/merge/algorithm/plan.rs"
PREVIEW = ROOT / "zircon_runtime/src/scene/dynamic_scene/session/merge/algorithm/preview.rs"


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class MovedMergePreviewReportPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.plan = PLAN.read_text(encoding="utf-8")
        cls.preview = PREVIEW.read_text(encoding="utf-8")

    def test_merge_plan_exposes_consuming_report_accessor(self) -> None:
        signature = "pub fn into_report(self) -> RuntimeSessionArchiveMergeReport"
        body = _function_body(self.plan, signature)

        self.assertEqual(body.strip(), "self.report")

    def test_merge_preview_moves_report_without_cloning(self) -> None:
        body = _function_body(self.preview, "fn preview_merge_archive(")

        self.assertIn(".into_report()", body)
        self.assertNotIn(".report()", body)
        self.assertNotIn(".clone()", body)

    def test_merge_plan_report_accessors_have_rust_regressions(self) -> None:
        self.assertIn(
            "runtime52_batch_consuming_merge_plan_moves_report_without_cloning",
            self.plan,
        )
        self.assertIn(
            "runtime52_batch_borrowed_merge_plan_report_remains_available",
            self.plan,
        )
        self.assertIn(
            "runtime52_batch_consuming_empty_merge_plan_preserves_empty_report",
            self.plan,
        )


if __name__ == "__main__":
    unittest.main()
