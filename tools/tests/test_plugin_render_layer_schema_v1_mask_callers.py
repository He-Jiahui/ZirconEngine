import unittest
from pathlib import Path


class PluginRenderLayerSchemaV1MaskCallerTests(unittest.TestCase):
    def test_plugin_callers_use_scene_schema_v1_mask_api(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        plugin_sources = list((repo_root / "zircon_plugins").rglob("*.rs"))

        stale_callers: list[str] = []
        hybrid_gi_schema_v1_callers = 0
        for source_path in plugin_sources:
            source = source_path.read_text(encoding="utf-8")
            relative_path = source_path.relative_to(repo_root).as_posix()
            if "from_extract_mask" in source:
                stale_callers.append(relative_path)
            if relative_path.startswith("zircon_plugins/hybrid_gi/runtime/src/"):
                hybrid_gi_schema_v1_callers += source.count(
                    "RenderLayerSet::from_scene_schema_v1_mask(u32::MAX)"
                )

        if stale_callers:
            self.fail(
                "Plugin render-layer callers still use retired from_extract_mask:\n"
                + "\n".join(stale_callers)
            )
        self.assertGreaterEqual(
            hybrid_gi_schema_v1_callers,
            8,
            "Hybrid GI fixtures and production placeholders should use the schema-v1 mask API",
        )


if __name__ == "__main__":
    unittest.main()
