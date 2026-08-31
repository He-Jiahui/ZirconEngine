from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.command_requests import CommandRequestJournal


class CommandRequestBulkPrunePerformanceContractTests(unittest.TestCase):
    def test_ephemeral_retention_deletes_each_selected_batch_in_one_statement(self) -> None:
        source = inspect.getsource(CommandRequestJournal.prune)

        self.assertNotIn("for request_id in expired_ephemeral", source)
        self.assertNotIn("for request_id in overflow", source)
        self.assertNotIn("DELETE FROM command_requests WHERE request_id=?", source)
        self.assertGreaterEqual(source.count("DELETE FROM command_requests"), 2)
        self.assertGreaterEqual(source.count("SELECT candidate.request_id"), 2)
        self.assertGreaterEqual(source.count("changed += connection.execute("), 2)


if __name__ == "__main__":
    unittest.main()
