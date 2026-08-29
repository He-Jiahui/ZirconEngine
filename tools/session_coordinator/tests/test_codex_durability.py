from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.codex_sync.durability import flush_directory


class CodexDirectoryDurabilityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_flush_directory_accepts_an_existing_directory(self) -> None:
        (self.root / "entry.json").write_text("{}", encoding="utf-8")

        flush_directory(self.root)

    def test_flush_directory_rejects_a_regular_file(self) -> None:
        target = self.root / "entry.json"
        target.write_text("{}", encoding="utf-8")

        with self.assertRaises(NotADirectoryError):
            flush_directory(target)


if __name__ == "__main__":
    unittest.main()
