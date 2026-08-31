from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.workflows.plan_import import TopologyImporter


class TopologyActivationExecutemanyPerformanceContractTests(unittest.TestCase):
    def test_activation_batches_node_and_edge_inserts(self) -> None:
        source = inspect.getsource(TopologyImporter._activate)

        self.assertGreaterEqual(source.count("connection.executemany("), 4)
        self.assertNotIn("connection.execute(\n                \"\"\"INSERT INTO workflow_nodes", source)
        self.assertNotIn("connection.execute(\n                    \"\"\"INSERT INTO workflow_edges", source)

    def test_append_only_activation_batches_new_tail(self) -> None:
        source = inspect.getsource(TopologyImporter._activate_append_only)

        self.assertGreaterEqual(source.count("connection.executemany("), 2)
        self.assertNotIn("connection.execute(\n                \"\"\"INSERT INTO workflow_nodes", source)
        self.assertNotIn("connection.execute(\n                    \"\"\"INSERT INTO workflow_edges", source)


if __name__ == "__main__":
    unittest.main()
