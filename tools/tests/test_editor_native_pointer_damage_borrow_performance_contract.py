from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
NATIVE_POINTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_pointer"
)
OWNERS = (
    "chrome_damage",
    "close_prompt_damage",
    "pane_button_damage",
    "viewport_toolbar_damage",
)


class EditorNativePointerDamageBorrowPerformanceContractTests(unittest.TestCase):
    def owner_sources(self) -> dict[Path, str]:
        sources: dict[Path, str] = {}
        for owner in OWNERS:
            module = NATIVE_POINTER / f"{owner}.rs"
            sources[module] = module.read_text(encoding="utf-8")
            for path in (NATIVE_POINTER / owner).rglob("*.rs"):
                sources[path] = path.read_text(encoding="utf-8")
        return sources

    def test_damage_projection_never_clones_model_rows(self) -> None:
        cloning_sources = [
            str(path.relative_to(ROOT))
            for path, source in self.owner_sources().items()
            if "row_data(" in source
        ]

        self.assertEqual([], cloning_sources)

    def test_scanned_window_tab_and_template_models_use_borrowed_iterators(self) -> None:
        floating = (NATIVE_POINTER / "chrome_damage/floating.rs").read_text(
            encoding="utf-8"
        )
        tabs = (NATIVE_POINTER / "chrome_damage/host_page/tabs.rs").read_text(
            encoding="utf-8"
        )
        nodes = (NATIVE_POINTER / "chrome_damage/host_page/template_nodes.rs").read_text(
            encoding="utf-8"
        )

        self.assertGreaterEqual(floating.count(".iter()"), 3)
        self.assertIn("for tab in page_chrome.tab_frames.iter()", tabs)
        self.assertIn("for node in template_nodes.iter()", nodes)


if __name__ == "__main__":
    unittest.main()
