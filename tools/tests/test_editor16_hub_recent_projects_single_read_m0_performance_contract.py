from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RECENT_WRITEBACK = (
    ROOT / "zircon_editor" / "src" / "core" / "hub_link" / "recent_writeback.rs"
)
RECENT_STORE = (
    ROOT
    / "zircon_runtime_interface"
    / "src"
    / "hub_protocol"
    / "recent_projects"
    / "store.rs"
)


class HubRecentProjectsSingleReadM0PerformanceContractTests(unittest.TestCase):
    def test_missing_registry_is_handled_by_the_single_read_attempt(self) -> None:
        writeback = RECENT_WRITEBACK.read_text(encoding="utf-8")
        store = RECENT_STORE.read_text(encoding="utf-8")
        bounded_read = store.split("fn read_bounded(", 1)[1].split(
            "fn write_registry(", 1
        )[0]

        self.assertIn("HubRecentProjectsStore::new(hub_recent_projects_path())", writeback)
        self.assertIn(".load_projection()", writeback)
        self.assertNotIn("registry_path.exists()", writeback)
        self.assertNotIn("path.exists()", bounded_read)
        self.assertEqual(bounded_read.count("File::open(path)"), 1)
        self.assertIn("io::ErrorKind::NotFound", bounded_read)


if __name__ == "__main__":
    unittest.main()
