from __future__ import annotations

import os
import subprocess
import sys
import unittest

from tools.session_coordinator.processes import process_is_alive


@unittest.skipUnless(os.name == "nt", "Windows process-handle semantics only")
class ProcessLivenessTests(unittest.TestCase):
    def test_exited_process_with_an_open_parent_handle_is_not_alive(self) -> None:
        child = subprocess.Popen([sys.executable, "-c", "import sys; sys.exit(255)"])
        self.addCleanup(child.wait)
        self.assertEqual(255, child.wait())

        self.assertFalse(process_is_alive(child.pid))


if __name__ == "__main__":
    unittest.main()
