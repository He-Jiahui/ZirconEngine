from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMANDS = ROOT / "zircon_editor/src/core/commands"
HOST_ACTIONS = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/command_palette_actions.rs"
)


class Editor08CommandPaletteQueryContractTests(unittest.TestCase):
    def test_registry_only_publishes_the_immutable_catalog_generation(self) -> None:
        registry = (COMMANDS / "registry.rs").read_text(encoding="utf-8")

        self.assertIn("pub fn command_palette_catalog", registry)
        self.assertNotIn("pub fn command_palette_query_window(", registry)
        self.assertNotIn("pub fn command_palette_query_window_with_mru(", registry)

    def test_retained_host_releases_registry_lock_before_querying(self) -> None:
        actions = HOST_ACTIONS.read_text(encoding="utf-8")

        self.assertEqual(actions.count("command_eval().shared_snapshot()"), 3)
        self.assertNotIn("command_eval().snapshot()", actions)
        self.assertIn("let catalog = {", actions)
        self.assertIn("commands.command_palette_catalog()", actions)
        self.assertIn("catalog.query_window_with_mru(", actions)
        self.assertNotIn("commands.command_palette_query_window", actions)

    def test_fuzzy_matcher_scans_each_document_once(self) -> None:
        palette = (COMMANDS / "palette.rs").read_text(encoding="utf-8")
        fuzzy = palette[palette.index("fn fuzzy_score(") :]

        self.assertNotIn("document.windows(", fuzzy)
        self.assertEqual(fuzzy.count("document.iter()"), 1)
        self.assertIn("metrics.document_byte_visits += 1;", fuzzy)

    def test_command_context_generation_publishes_a_shared_snapshot(self) -> None:
        snapshot = (COMMANDS / "eval_snapshot_handle.rs").read_text(encoding="utf-8")

        self.assertIn("context: Arc<CommandEvalCtx>", snapshot)
        self.assertIn("pub fn shared_snapshot(&self) -> Arc<CommandEvalCtx>", snapshot)
        self.assertIn("Arc::clone(&snapshot.context)", snapshot)


if __name__ == "__main__":
    unittest.main()
