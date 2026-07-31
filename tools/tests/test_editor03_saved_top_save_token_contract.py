import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


class Editor03SavedTopSaveTokenContractTests(unittest.TestCase):
    def test_public_contract_is_typed_and_old_infallible_mark_is_deleted(self) -> None:
        facade = read("zircon_editor/src/core/editing/engine/mod.rs")
        transaction = read("zircon_editor/src/core/editing/engine/transaction.rs")
        save_token = read(
            "zircon_editor/src/core/editing/engine/transaction/save_token.rs"
        )
        history = read("zircon_editor/src/core/editing/engine/history.rs")

        for symbol in ("HistorySaveToken", "HistorySaveMarkOutcome"):
            self.assertIn(symbol, facade)
        self.assertIn("mod save_token;", transaction)
        self.assertIn("pub fn capture_save_token", save_token)
        self.assertIn("pub fn mark_saved_if_unchanged", save_token)
        self.assertNotIn("pub fn mark_saved(&self", transaction)
        self.assertNotIn("pub fn mark_saved(&mut self", history)

    def test_token_binds_context_transaction_identity_and_branch_generation(self) -> None:
        history = read("zircon_editor/src/core/editing/engine/history.rs")
        transaction = read("zircon_editor/src/core/editing/engine/transaction.rs")
        save_token = read(
            "zircon_editor/src/core/editing/engine/transaction/save_token.rs"
        )

        self.assertIn("history: HistoryContextId", history)
        self.assertIn("transaction: Option<TransactionId>", history)
        self.assertIn("generation: u64", history)
        self.assertIn("history_generations", transaction)
        self.assertIn("save_token_lineage", transaction)
        self.assertIn("belongs_to", history)
        self.assertIn("HistoryChangedDuringSave", save_token)
        self.assertIn("SaveTokenHistoryMismatch", save_token)
        self.assertIn("SaveTokenEngineMismatch", save_token)
        self.assertIn("SaveTokenActiveTransaction", save_token)
        mark_body = save_token.split("pub fn mark_saved_if_unchanged", 1)[1]
        self.assertLess(mark_body.find("belongs_to"), mark_body.find("flush_operation_group"))
        self.assertLess(
            mark_body.find("token.history() != history"),
            mark_body.find("flush_operation_group"),
        )

        for getter in ("history", "transaction", "generation"):
            self.assertNotIn(f"pub const fn {getter}", history)

    def test_every_history_mutation_advances_generation(self) -> None:
        transaction = read("zircon_editor/src/core/editing/engine/transaction.rs")
        save_token = read(
            "zircon_editor/src/core/editing/engine/transaction/save_token.rs"
        )

        self.assertIn("next_history_generation", save_token)
        for mutation in ("commit", "replay", "clear_history_and_context"):
            self.assertIn(f"fn {mutation}", transaction)

    def test_dirty_batch_is_typed_generation_owned_and_delta_bounded(self) -> None:
        facade = read("zircon_editor/src/core/editing/engine/mod.rs")
        transaction = read("zircon_editor/src/core/editing/engine/transaction.rs")
        dirty_batch = read(
            "zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs"
        )
        tests = read(
            "zircon_editor/src/tests/editing/transaction_engine/dirty_batch.rs"
        )

        self.assertIn("mod dirty_batch;", transaction)
        for symbol in (
            "HistoryDirtyBatch",
            "HistoryDirtyBatchKind",
            "HistoryDirtyCursor",
            "HistoryDirtyState",
        ):
            self.assertIn(symbol, facade)
        self.assertIn("pub fn dirty_states_since", dirty_batch)
        self.assertIn("HistoryDirtyJournal", dirty_batch)
        self.assertIn("VecDeque", dirty_batch)
        self.assertIn("HistoryDirtyBatchKind::Unchanged", dirty_batch)
        self.assertIn("HistoryDirtyBatchKind::Delta", dirty_batch)
        self.assertIn("HistoryDirtyBatchKind::Reset", dirty_batch)
        self.assertIn("record_dirty_change", dirty_batch)
        self.assertIn("change_start_after", dirty_batch)
        self.assertIn("changed_histories_after", dirty_batch)
        self.assertIn(".range(start..)", dirty_batch)
        self.assertNotIn("journal_visits += visits", dirty_batch)
        self.assertIn("journal_visits", dirty_batch)
        self.assertIn("HistoryDirtyCursorEngineMismatch", dirty_batch)
        query_body = dirty_batch.split("pub fn dirty_states_since", 1)[1].split(
            "pub(super) fn reserve_dirty_change", 1
        )[0]
        self.assertNotIn("histories:", query_body)
        self.assertLess(
            query_body.find("cursor.generation == current_generation"),
            query_body.find("history_generations"),
        )
        for test_name in (
            "initial_batch_is_sorted_reset_and_stable_cursor_is_allocation_empty",
            "delta_contains_only_histories_changed_after_the_cursor",
            "saved_top_change_publishes_clean_delta_without_breaking_idempotent_completion",
            "cursor_from_another_engine_is_rejected",
            "cursor_older_than_the_bounded_journal_receives_reset",
            "undo_redo_and_history_clear_publish_current_dirty_state",
            "failed_generation_reservation_and_clear_type_mismatch_publish_no_delta",
        ):
            self.assertIn(f"fn {test_name}", tests)

    def test_behavior_matrix_is_present(self) -> None:
        tests = read("zircon_editor/src/tests/editing/transaction_engine/history.rs")
        for test_name in (
            "save_token_rejects_commit_between_capture_and_completion",
            "save_token_rejects_same_top_branch_replacement",
            "save_token_rejects_undo_and_redo_between_capture_and_completion",
            "empty_history_token_is_typed_and_invalidated_by_first_commit",
            "save_token_rejects_cross_document_use",
            "invalid_save_token_does_not_flush_an_open_operation_group",
            "save_token_rejects_cross_engine_use",
            "save_token_capture_and_completion_reject_active_transaction_scopes",
            "repeated_save_completion_is_reported_without_moving_the_baseline",
            "save_token_is_invalidated_by_capacity_eviction_and_history_clear",
            "multi_document_tokens_complete_independently",
        ):
            self.assertIn(f"fn {test_name}", tests)

        operation_group_tests = read(
            "zircon_editor/src/tests/editing/transaction_engine/operation_group.rs"
        )
        for test_name in (
            "operation_group_flush_restores_group_after_generation_exhaustion",
            "operation_group_flush_restores_group_after_concurrent_busy",
            "operation_group_initialization_blocks_concurrent_flush",
            "operation_group_first_push_preserves_rollback_failure",
        ):
            self.assertIn(f"fn {test_name}", operation_group_tests)

        operation_group = read(
            "zircon_editor/src/core/editing/engine/transaction/operation_group.rs"
        )
        transaction = read("zircon_editor/src/core/editing/engine/transaction.rs")
        for phase in ("Initializing", "Open", "Flushing"):
            self.assertIn(phase, operation_group)
        self.assertIn("OperationGroupPhase::Initializing", operation_group)
        self.assertIn("active.phase = OperationGroupPhase::Flushing", operation_group)
        new_group = operation_group.split("self.flush_operation_group()?;", 1)[1]
        reservation = new_group.find("self.reserve_operation_group")
        begin = new_group.find("self.begin_transaction")
        self.assertGreaterEqual(reservation, 0)
        self.assertGreaterEqual(begin, 0)
        self.assertLess(reservation, begin)
        reserve_helper = operation_group.split("fn reserve_operation_group", 1)[1]
        self.assertIn("state.operation_group = Some", reserve_helper)
        self.assertIn("transaction: None", reserve_helper)
        begin_body = transaction.split("fn begin_transaction", 1)[1].split(
            "pub fn undo", 1
        )[0]
        self.assertLess(begin_body.find("operation_group_allows_begin"), begin_body.find("state.active.push"))
        self.assertIn("active.allows_begin(history, operation_group_reservation)", begin_body)
        self.assertIn("None => operation_group_reservation.is_none()", begin_body)
        for test_name in (
            "unowned_begin_cannot_cross_live_operation_group_reservation",
            "stale_operation_group_cleanup_preserves_successor",
        ):
            self.assertIn(f"fn {test_name}", operation_group)
        continuation = operation_group.split(
            "if let Some(transaction) = existing_transaction", 1
        )[1].split("self.flush_operation_group()?;", 1)[0]
        self.assertIn(
            "self.clear_operation_group_for_transaction(transaction)", continuation
        )
        self.assertNotIn("self.lock_state().operation_group = None", continuation)
        scoped_cleanup = operation_group.split(
            "fn clear_operation_group_for_transaction", 1
        )[1].split("fn clear_initializing_operation_group", 1)[0]
        self.assertIn("active.transaction == Some(transaction)", scoped_cleanup)


if __name__ == "__main__":
    unittest.main()
