from __future__ import annotations

import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
APP_MANIFEST = REPO_ROOT / "zircon_app" / "Cargo.toml"
PROFILE_PRESETS = REPO_ROOT / "zircon_runtime" / "runtime-feature-presets.toml"
EDITOR_VIEWPORT_DEFAULTS = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "viewport"
    / "editor_viewport_render_defaults.rs"
)
ADVANCED_RENDER_CATALOG_FEATURE = "first-party-advanced-render-runtime-plugins"


class HybridGiEditorProfileTests(unittest.TestCase):
    def test_editor_target_compiles_the_default_hybrid_gi_provider_catalog(self) -> None:
        app_manifest = tomllib.loads(APP_MANIFEST.read_text(encoding="utf-8"))
        editor_target_features = app_manifest["features"]["target-editor-host"]

        self.assertIn(ADVANCED_RENDER_CATALOG_FEATURE, editor_target_features)

    def test_editor_and_dev_presets_match_the_editor_provider_catalog(self) -> None:
        presets = tomllib.loads(PROFILE_PRESETS.read_text(encoding="utf-8"))
        profiles = {profile["id"]: profile for profile in presets["profiles"]}

        for profile_id in ("editor", "dev"):
            self.assertIn(
                ADVANCED_RENDER_CATALOG_FEATURE,
                profiles[profile_id]["app_features"],
                f"{profile_id} must compile the provider requested by its default HGI render profile",
            )

    def test_editor_preview_accepts_named_hybrid_gi_profiles_without_custom_budget_leakage(
        self,
    ) -> None:
        source = EDITOR_VIEWPORT_DEFAULTS.read_text(encoding="utf-8")

        self.assertIn("ZIRCON_EDITOR_HYBRID_GI_PROFILE", source)
        for profile in ("fully-dynamic", "indoor-static", "open-world", "cinematic"):
            self.assertIn(f'"{profile}"', source)
        for budget in ("trace_budget", "card_budget", "voxel_budget"):
            self.assertIn(f"settings.{budget} = 0;", source)


if __name__ == "__main__":
    unittest.main()
