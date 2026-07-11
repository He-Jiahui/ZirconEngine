from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.assets import StaticAssetService
from tools.session_coordinator.models import CoordinatorError


class StaticAssetServiceTests(unittest.TestCase):
    def test_index_and_hashed_asset_have_distinct_cache_policy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "assets").mkdir()
            (root / "index.html").write_text("<main>console</main>", encoding="utf-8")
            (root / "assets" / "app-Bmsr0ZXN.js").write_text("console.log('ok')", encoding="utf-8")
            service = StaticAssetService(root)

            index = service.resolve("/ui/")
            asset = service.resolve("/ui/assets/app-Bmsr0ZXN.js")

        self.assertEqual("no-store", index.headers["Cache-Control"])
        self.assertEqual("public,max-age=31536000,immutable", asset.headers["Cache-Control"])
        self.assertEqual("text/html; charset=utf-8", index.headers["Content-Type"])

    def test_only_ui_navigation_falls_back_to_index(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            root.mkdir(exist_ok=True)
            (root / "index.html").write_text("index", encoding="utf-8")
            service = StaticAssetService(root)
            self.assertEqual(b"index", service.resolve("/ui/workflows/run-a").body)
            with self.assertRaises(CoordinatorError):
                service.resolve("/ui/missing.js")

    def test_traversal_and_bootstrap_are_never_static_assets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            root.mkdir(exist_ok=True)
            (root / "index.html").write_text("index", encoding="utf-8")
            service = StaticAssetService(root)
            with self.assertRaises(CoordinatorError):
                service.resolve("/ui/%2e%2e/secret")
            self.assertIsNone(service.resolve("/ui/bootstrap/ticket"))


if __name__ == "__main__":
    unittest.main()
