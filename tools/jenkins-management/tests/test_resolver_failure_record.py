"""Pure admission and bounded DTO checks; actual PS collector tests run in Jenkins."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

PATH = Path(__file__).resolve().parents[1] / "resolver_failure_record.py"
SPEC = importlib.util.spec_from_file_location("s80_recipe_tests", PATH)
RECIPE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RECIPE)


class ErrorGraphContractTests(unittest.TestCase):
    def plan(self):
        return {"schemaVersion": 1, "requestId": RECIPE.RUN, "attemptId": RECIPE.RUN,
                "output": str(RECIPE.OUTPUT), "budgetSeconds": 120,
                "continuation": {"build": 213, "originalNativeStarted": False},
                "message": {"sha256": RECIPE.MESSAGE_SHA},
                "support": {"path": str(RECIPE.SUPPORT), "sha256": RECIPE.SUPPORT_SHA},
                "test": {"path": str(RECIPE.TEST), "sha256": RECIPE.TEST_SHA},
                "resolver": {"path": str(RECIPE.ROOT / "tools/maintenance/WindowsPathResolver.psm1"),
                             "sha256": RECIPE.RESOLVER_SHA},
                "pesterManifest": {"path": str(RECIPE.PESTER)},
                "limits": {"maxObjects": 12, "maxDepth": 6, "maxUtf8Bytes": 65536}}

    def test_rejects_identity_output_budget_or_collector_limit_drift(self):
        RECIPE.validate_plan(self.plan())
        for key, value in [("attemptId", "retry"), ("output", "C:/temp"), ("budgetSeconds", 121),
                           ("limits", {"maxObjects": 13, "maxDepth": 6, "maxUtf8Bytes": 65536})]:
            plan = self.plan()
            plan[key] = value
            with self.assertRaises(ValueError):
                RECIPE.validate_plan(plan)

    def test_rejects_replaced_source_or_unrelated_message(self):
        for key in ("test", "resolver", "message"):
            plan = self.plan()
            plan[key] = {**plan[key], "sha256": "0" * 64}
            with self.assertRaises(ValueError):
                RECIPE.validate_plan(plan)

    def test_cycle_and_shared_node_ids_remain_references_in_readonly_dto(self):
        graph = {"status": "partial", "rootCount": 2, "nodeCount": 2,
                 "roots": [{"nodeId": 0}, {"nodeId": 0}],
                 "nodes": [{"nodeId": 0, "depth": 0, "edges": [{"nodeId": 1}]},
                           {"nodeId": 1, "depth": 1, "edges": [{"nodeId": 0}]}]}
        original = copy.deepcopy(graph)
        self.assertEqual(RECIPE.graph_facts(json.dumps(graph).encode()), original)
        self.assertEqual(graph, original)

    def test_missing_roots_and_output_truncation_never_become_complete(self):
        graph = {"status": "failed", "rootCount": 0, "nodeCount": 0, "roots": [], "nodes": []}
        self.assertEqual(RECIPE.graph_facts(json.dumps(graph).encode())["status"], "failed")
        with self.assertRaises(ValueError):
            RECIPE.graph_facts(b" " * 65537)
        graph = {"status": "partial", "nodeCount": 1, "roots": [{"nodeId": 3}],
                 "nodes": [{"nodeId": 0, "depth": 0, "edges": []}]}
        with self.assertRaises(ValueError):
            RECIPE.graph_facts(json.dumps(graph).encode())


if __name__ == "__main__":
    unittest.main()
