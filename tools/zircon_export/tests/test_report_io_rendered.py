from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.report_io import write_rendered_report_targets


class RenderedReportIoTests(unittest.TestCase):
    def test_successful_multi_target_write_reuses_one_serialization(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            first = root / "first.json"
            second = root / "second.json"
            report = {"diagnostics": [], "payload": list(range(128))}
            real_dumps = json.dumps

            with mock.patch(
                "tools.zircon_export.report_io.json.dumps",
                wraps=real_dumps,
            ) as dumps:
                written, rendered = write_rendered_report_targets(
                    [("first", first), ("second", second)],
                    report,
                )

            self.assertTrue(written)
            self.assertEqual(1, dumps.call_count)
            self.assertEqual(rendered, first.read_text(encoding="utf-8"))
            self.assertEqual(rendered, second.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
