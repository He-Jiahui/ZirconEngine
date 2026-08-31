import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


class Editor09ImportDiagnosticsSubmissionContractTests(unittest.TestCase):
    def test_asset_diagnostics_follow_the_job_submission_transaction(self) -> None:
        diagnostics = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/diagnostics.rs"
        ).read_text(encoding="utf-8")
        job = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/job.rs"
        ).read_text(encoding="utf-8")
        submit = (
            REPO_ROOT / "zircon_editor/src/core/asset/import_flow/submit.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct DeferredSubmissionDiagnostic", diagnostics)
        self.assertIn("struct EditorAssetImportFlightDiagnostics", diagnostics)
        self.assertIn("diagnostics.arm();", submit)
        self.assertIn("diagnostics.reject_submission", submit)
        self.assertIn("Arc<EditorAssetImportFlightDiagnostics>", job)
        self.assertNotIn("diagnostics: EditorAssetImportDiagnostics", job)

    def test_rejected_asset_job_has_one_typed_rejection_record(self) -> None:
        diagnostics_tests = (
            REPO_ROOT
            / "zircon_editor/src/core/asset/import_flow/tests/diagnostics.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "asset_submission_rejection_overrides_pre_arm_drop_cancellation",
            diagnostics_tests,
        )
        self.assertIn('message().contains("result=rejected")', diagnostics_tests)
        self.assertIn('message().contains("shutting down")', diagnostics_tests)
        self.assertIn("LogJumpTarget::Asset", diagnostics_tests)


if __name__ == "__main__":
    unittest.main()
