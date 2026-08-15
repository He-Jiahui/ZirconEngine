from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DIRTY = ROOT / "zircon_editor" / "src" / "core" / "asset" / "dirty"
ADAPTER = DIRTY / "save_job_adapter.rs"
ADAPTER_TESTS = DIRTY / "save_job_adapter" / "tests.rs"
JOBS_SYSTEM = ROOT / "zircon_editor" / "src" / "core" / "jobs" / "system" / "mod.rs"
JOBS_CONSTRUCTION = (
    ROOT / "zircon_editor" / "src" / "core" / "jobs" / "system" / "construction.rs"
)
JOBS_SUBMISSION = (
    ROOT / "zircon_editor" / "src" / "core" / "jobs" / "system" / "submission.rs"
)
JOBS_ROOT = ROOT / "zircon_editor" / "src" / "core" / "jobs" / "mod.rs"


class InteractiveSaveJobAdapterContractTests(unittest.TestCase):
    def test_adapter_reserves_before_materializing_save_owners(self) -> None:
        source = ADAPTER.read_text(encoding="utf-8")
        schedule = source[source.index("pub fn schedule(") : source.index("pub fn pump_completed(")]

        admission = schedule.index("self.jobs.reserve_batch_admission(")
        mutex_resolution = schedule.index("save_mutex_for(&intent)")
        executor_materialization = schedule.index("let executor = executor_factory();")
        job_materialization = schedule.index("SaveDirtyViewJob {")
        self.assertLess(admission, mutex_resolution)
        self.assertLess(mutex_resolution, executor_materialization)
        self.assertLess(admission, job_materialization)
        self.assertIn("JobCategory::InteractiveSave", schedule)
        self.assertIn("JobPriority::Interactive", schedule)
        self.assertIn("EditorJobAdmissionRequest::new(", schedule)
        self.assertIn("reservation.commit(jobs)?", schedule)
        self.assertNotIn("pending_admission_window", schedule)

    def test_adapter_has_bounded_completion_and_no_ui_or_thread_owner(self) -> None:
        source = ADAPTER.read_text(encoding="utf-8")
        completion_pump = source[
            source.index("pub fn pump_completed_with_budget(") : source.index(
                "pub fn begin_shutdown(&mut self)"
            )
        ]

        self.assertIn("DEFAULT_SAVE_DIRTY_VIEWS_COMPLETION_BUDGET: usize = 64", source)
        self.assertIn("max_tickets.min(self.tickets.len())", completion_pump)
        self.assertIn("self.complete_slot(slot_index, completion)", completion_pump)
        self.assertIn(
            "completions: std::mem::take(&mut self.completions)", completion_pump
        )
        self.assertNotIn("BTreeMap", source)
        self.assertNotIn("self.completions.insert", completion_pump)
        self.assertNotIn("collect::<Vec<_>>()", completion_pump)
        self.assertIn("Err(JobError::failed(failure))", source)
        self.assertIn("downcast_ref::<SaveDirtyViewFailure>()", source)
        self.assertIn("pub fn begin_shutdown(&mut self)", source)
        for forbidden in (
            "thread::spawn",
            "std::thread",
            "UiHostWindow",
            "RetainedEditorHost",
            "serialize(",
        ):
            self.assertNotIn(forbidden, source)

    def test_shared_admission_window_and_behavior_regressions_are_present(self) -> None:
        system = JOBS_SYSTEM.read_text(encoding="utf-8")
        construction = JOBS_CONSTRUCTION.read_text(encoding="utf-8")
        submission = JOBS_SUBMISSION.read_text(encoding="utf-8")
        jobs_root = JOBS_ROOT.read_text(encoding="utf-8")
        tests = ADAPTER_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "pub use construction::{EditorJobAdmissionWindow, EditorJobSystem};", system
        )
        self.assertIn(
            "pub use admission_reservation::EditorJobBatchAdmissionReservation;", system
        )
        self.assertNotIn("pub struct EditorJobAdmissionWindow", system)
        self.assertNotIn("pub fn pending_admission_window(&self)", system)
        self.assertNotIn("pub fn reserve_batch_admission(", system)
        self.assertIn("pub struct EditorJobAdmissionWindow", construction)
        self.assertIn("pub const fn max_pending_entries(self)", construction)
        self.assertIn("pub fn pending_admission_window(&self)", submission)
        self.assertIn("pub fn reserve_batch_admission(", submission)
        self.assertIn("EditorJobAdmissionRequest", jobs_root)
        self.assertIn("EditorJobBatchAdmissionReservation", jobs_root)
        self.assertIn("EditorJobAdmissionWindow", jobs_root)
        for test_name in (
            "interactive_save_batch_rejects_before_mutex_or_executor_materialization",
            "interactive_save_batch_reports_actual_bytes_before_mutex_or_executor_materialization",
            "interactive_save_reservation_blocks_competing_admission_before_materializing_executor",
            "interactive_save_batch_reuses_the_caller_supplied_foreground_save_mutex",
            "interactive_save_batch_preserves_partial_failure_for_generation_safe_apply_and_retry",
            "interactive_save_shutdown_cancels_owned_pending_tickets_and_rejects_new_batches",
            "interactive_save_completion_pump_inspects_at_most_the_explicit_ticket_budget",
        ):
            self.assertIn(test_name, tests)


if __name__ == "__main__":
    unittest.main()
