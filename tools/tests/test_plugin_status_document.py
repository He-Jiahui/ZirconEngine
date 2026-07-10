import tempfile
import unittest
from pathlib import Path

from tools.tests.plugin_status_document import (
    StatusDocumentPath,
    strip_resolved_output_archives,
)


class PluginStatusDocumentTests(unittest.TestCase):
    def test_resolved_archive_blocks_can_be_excluded_from_current_text_checks(self):
        text = (
            "CURRENT_TOKEN\n"
            "<!-- resolved plan output archive: 09/2026-07-09-output-records.md -->\n"
            "HISTORICAL_TOKEN\n"
            "<!-- end resolved plan output archive -->\n"
        )

        current_text = strip_resolved_output_archives(text)

        self.assertIn("CURRENT_TOKEN", current_text)
        self.assertNotIn("HISTORICAL_TOKEN", current_text)

    def test_expanded_document_is_cached_until_archive_changes(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive = root / "09" / "2026-07-09-output-records.md"
            archive.parent.mkdir()
            archive.write_text("FIRST_TOKEN\n", encoding="utf-8")
            plan = root / "09-plan.md"
            plan.write_text(
                "[archive](09/2026-07-09-output-records.md)\n",
                encoding="utf-8",
            )
            status_path = StatusDocumentPath(plan)

            first = status_path.read_text(encoding="utf-8")
            second = status_path.read_text(encoding="utf-8")
            archive.write_text("SECOND_LONGER_TOKEN\n", encoding="utf-8")
            third = status_path.read_text(encoding="utf-8")

        self.assertIs(first, second)
        self.assertIsNot(second, third)
        self.assertIn("SECOND_LONGER_TOKEN", third)

    def test_numbered_output_archive_is_expanded_at_link_position(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive = root / "09" / "2026-07-09-output-records.md"
            archive.parent.mkdir()
            archive.write_text("ARCHIVED_STATUS_TOKEN\n", encoding="utf-8")
            plan = root / "09-plan.md"
            plan.write_text(
                """
## 状态与产出记录

- 迁入记录：[archive](09/2026-07-09-output-records.md)

## 5. 里程碑与任务分解
""",
                encoding="utf-8",
            )

            text = StatusDocumentPath(plan).read_text(encoding="utf-8")
            status = text[text.index("## 状态") : text.index("## 5.")]

        self.assertIn("ARCHIVED_STATUS_TOKEN", status)

    def test_non_archive_markdown_links_are_not_expanded(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            reference = root / "architecture.md"
            reference.write_text("REFERENCE_ONLY_TOKEN\n", encoding="utf-8")
            plan = root / "plan.md"
            plan.write_text(
                "See [architecture](architecture.md).\n",
                encoding="utf-8",
            )

            text = StatusDocumentPath(plan).read_text(encoding="utf-8")

        self.assertNotIn("REFERENCE_ONLY_TOKEN", text)


if __name__ == "__main__":
    unittest.main()
