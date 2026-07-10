from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "asset"


def production_rust_sources(root: Path) -> list[Path]:
    sources: list[Path] = []
    for path in root.rglob("*.rs"):
        relative = path.relative_to(root)
        if "tests" in relative.parts:
            continue
        if path.name in {"test.rs", "tests.rs"}:
            continue
        if path.name.startswith("test_") or path.name.endswith("_tests.rs"):
            continue
        sources.append(path)
    return sources


class Frameworks05AssetUiBoundaryTests(unittest.TestCase):
    def test_asset_production_has_no_ui_domain_references(self) -> None:
        offenders: list[str] = []
        for path in production_rust_sources(ASSET_ROOT):
            for line_number, line in enumerate(
                path.read_text(encoding="utf-8").splitlines(), start=1
            ):
                if "crate::ui::" in line:
                    offenders.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: {line.strip()}"
                    )

        self.assertEqual([], offenders)

    def test_zui_loader_registration_is_owned_by_ui_document_plugin(self) -> None:
        asset_importer = (
            ASSET_ROOT / "importer" / "ingest" / "asset_importer.rs"
        ).read_text(encoding="utf-8")
        asset_ingest_mod = (
            ASSET_ROOT / "importer" / "ingest" / "mod.rs"
        ).read_text(encoding="utf-8")
        plugin = (
            REPO_ROOT
            / "zircon_plugins"
            / "ui_document_importer"
            / "runtime"
            / "src"
            / "plugin.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("import_ui_zui_asset", asset_importer)
        self.assertNotIn("import_ui_zui_asset", asset_ingest_mod)
        self.assertNotIn("zircon.builtin.ui_document.zui", asset_importer)
        self.assertIn("registry.register_asset_importer", plugin)
        self.assertIn("import_ui_zui_document", plugin)

    def test_root_module_order_has_no_asset_ui_semantic_comment(self) -> None:
        crate_root = (REPO_ROOT / "zircon_runtime" / "src" / "lib.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("must be declared before", crate_root)


if __name__ == "__main__":
    unittest.main()
