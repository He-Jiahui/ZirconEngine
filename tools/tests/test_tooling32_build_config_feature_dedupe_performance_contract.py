from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_build_config import BuildConfig


SCRIPT = Path(__file__).resolve().parents[1] / "zircon_build_config.py"


class BuildConfigFeatureDedupePerformanceContractTests(unittest.TestCase):
    def test_target_feature_dedupe_uses_a_synchronized_index(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("feature_set = {target_feature}", source)
        self.assertIn("feature not in feature_set", source)
        self.assertIn("feature_set.add(feature)", source)
        self.assertNotIn("feature not in features", source)

    def test_preserves_ordered_feature_rendering(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("features = [target_feature]", source)
        self.assertIn('return " ".join(features)', source)

    def test_preserves_first_seen_non_target_features(self) -> None:
        config = object.__new__(BuildConfig)
        object.__setattr__(
            config,
            "runtime_features",
            (
                "render-pbr",
                "target-server",
                "render-pbr",
                "audio",
                "target-client",
                "audio",
            ),
        )

        self.assertEqual(
            "target-editor-host render-pbr audio",
            config.feature_arg_for_target("target-editor-host"),
        )


if __name__ == "__main__":
    unittest.main()
