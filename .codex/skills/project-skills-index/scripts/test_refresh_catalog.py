"""Check catalog discovery, validation, and the exact write scope."""

from pathlib import Path
import tempfile
import unittest

from refresh_catalog import CATALOG, checked_path, refresh


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.skill = self.root / ".codex/skills/sample/SKILL.md"
        self.skill.parent.mkdir(parents=True)
        self.skill.write_text("---\nname: sample\ndescription: Unicode guidance | a second value\n---\n\nSource body.\n", encoding="utf-8")

    def files(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()}

    def test_check_is_read_only_and_write_is_idempotent(self):
        before = self.files()
        self.assertEqual([CATALOG], refresh(self.root))
        self.assertEqual(before, self.files())
        refresh(self.root, True)
        content = (self.root / CATALOG).read_text(encoding="utf-8")
        link = content.split("[SKILL.md](<", 1)[1].split(">)", 1)[0]
        self.assertEqual(self.skill.resolve(), ((self.root / CATALOG).parent / link).resolve())
        self.assertIn("Unicode guidance", content)
        self.assertIn("\\|", content)
        self.assertNotIn("Source body.", content)
        self.assertEqual([], refresh(self.root))
        self.assertEqual([], refresh(self.root, True))

    def test_only_catalog_is_written_and_guides_are_not_discovered(self):
        guide = self.skill.with_name("guide.md")
        guide.write_text("Reference without frontmatter.", encoding="utf-8")
        personal = self.root / "personal.txt"
        personal.write_text("keep", encoding="utf-8")
        before = self.files()
        refresh(self.root, True)
        after = self.files()
        self.assertEqual({CATALOG}, set(after) - set(before))
        self.assertTrue(all(after[path] == data for path, data in before.items()))
        self.assertIn("1 discoverable", (self.root / CATALOG).read_text(encoding="utf-8"))
        self.skill.write_text(self.skill.read_text(encoding="utf-8").replace("Unicode guidance", "Changed guidance"), encoding="utf-8")
        self.assertEqual([CATALOG], refresh(self.root))
        refresh(self.root, True)
        self.assertIn("Changed guidance", (self.root / CATALOG).read_text(encoding="utf-8"))

    def test_invalid_or_duplicate_source_prevents_writes(self):
        second = self.skill.parent.parent / "second/SKILL.md"
        second.parent.mkdir()
        for content in (self.skill.read_text(encoding="utf-8"), "No metadata", "---\nname: Bad_Name\ndescription: Invalid\n---\n"):
            with self.subTest(content=content):
                second.write_text(content, encoding="utf-8")
                before = self.files()
                with self.assertRaises(ValueError):
                    refresh(self.root, True)
                self.assertEqual(before, self.files())

    def test_output_escape_is_rejected(self):
        before = self.files()
        for relative in ("../../outside.md", "C:/outside.md", "/outside.md", "one\\outside.md"):
            with self.subTest(relative=relative), self.assertRaises(ValueError):
                checked_path(self.root, relative)
        self.assertEqual(before, self.files())


if __name__ == "__main__":
    unittest.main()
