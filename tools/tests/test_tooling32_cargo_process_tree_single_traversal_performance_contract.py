from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator import processes


class CargoProcessTreeSingleTraversalPerformanceContractTests(unittest.TestCase):
    def test_live_cargo_tree_uses_one_specialized_traversal(self) -> None:
        source = inspect.getsource(processes.live_cargo_process_tree_pids)

        self.assertIn(
            "_cargo_descendant_pids(root_pid, parents, executable_names)", source
        )
        self.assertNotIn("for cargo_root in cargo_roots", source)

    def test_specialized_traversal_includes_only_tool_roots_and_descendants(self) -> None:
        parents = {
            1: 0,
            2: 1,
            3: 2,
            4: 1,
            5: 4,
            6: 5,
            7: 1,
            8: 7,
            20: 0,
            21: 20,
        }
        executable_names = {
            1: "coordinator.exe",
            2: "control-client.exe",
            3: "helper.exe",
            4: "cargo.exe",
            5: "rustc.exe",
            6: "link.exe",
            7: "rustc.exe",
            8: "sccache.exe",
            20: "cargo.exe",
            21: "link.exe",
        }

        self.assertEqual(
            processes._cargo_descendant_pids(1, parents, executable_names),
            (4, 5, 6, 7, 8),
        )


if __name__ == "__main__":
    unittest.main()
