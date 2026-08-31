import unittest
from pathlib import Path


class RuntimeOperationServiceStructureTests(unittest.TestCase):
    def test_operation_service_state_owners_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        service_path = repo_root / "zircon_runtime/src/operation/service.rs"
        limits_path = repo_root / "zircon_runtime/src/operation/service/limits.rs"
        task_state_path = (
            repo_root / "zircon_runtime/src/operation/service/task_state.rs"
        )
        json_budget_path = (
            repo_root / "zircon_runtime/src/operation/service/json_budget.rs"
        )
        prepare_completion_path = (
            repo_root
            / "zircon_runtime/src/operation/service/prepare_completion.rs"
        )
        admission_path = (
            repo_root / "zircon_runtime/src/operation/service/admission.rs"
        )

        service = service_path.read_text(encoding="utf-8")
        limits = limits_path.read_text(encoding="utf-8")
        task_state = task_state_path.read_text(encoding="utf-8")
        json_budget = json_budget_path.read_text(encoding="utf-8")
        prepare_completion = prepare_completion_path.read_text(encoding="utf-8")
        admission = admission_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(service.splitlines()), 800)
        self.assertIn("mod limits;", service)
        self.assertIn("mod task_state;", service)
        self.assertIn("mod json_budget;", service)
        self.assertIn("mod prepare_completion;", service)
        self.assertIn("use prepare_completion::{", service)
        self.assertNotIn("pub(super) use prepare_completion", service)
        self.assertNotIn("struct RuntimeOperationLimits", service)
        self.assertNotIn("struct RuntimeOperationTaskState", service)
        self.assertNotIn("struct JsonByteCounter", service)
        self.assertNotIn("enum RuntimeOperationPrepareCompletion", service)
        self.assertNotIn("struct RuntimeOperationAdmissionReservation", service)
        self.assertIn("struct RuntimeOperationLimits", limits)
        self.assertIn("struct RuntimeOperationTaskState", task_state)
        self.assertIn("fn compact_phase_indexes", task_state)
        self.assertIn("fn allocate_handle", task_state)
        self.assertIn("struct JsonByteCounter", json_budget)
        self.assertIn("enum RuntimeOperationPrepareCompletion", prepare_completion)
        self.assertIn("struct RuntimeOperationAdmissionReservation", admission)


if __name__ == "__main__":
    unittest.main()
