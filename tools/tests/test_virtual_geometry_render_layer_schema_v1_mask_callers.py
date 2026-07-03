import unittest
from pathlib import Path


class VirtualGeometryRenderLayerSchemaV1MaskCallerTests(unittest.TestCase):
    def test_virtual_geometry_callers_use_scene_schema_v1_mask_api(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source_root = repo_root / "zircon_plugins/virtual_geometry/runtime/src"

        stale_callers: list[str] = []
        schema_v1_callers = 0
        for source_path in source_root.rglob("*.rs"):
            source = source_path.read_text(encoding="utf-8")
            relative_path = source_path.relative_to(repo_root).as_posix()
            if "RenderLayerSet::from_legacy_mask" in source:
                stale_callers.append(relative_path)
            schema_v1_callers += source.count("RenderLayerSet::from_scene_schema_v1_mask")

        if stale_callers:
            self.fail(
                "Virtual Geometry render-layer callers still use retired from_legacy_mask:\n"
                + "\n".join(stale_callers)
            )
        self.assertGreaterEqual(
            schema_v1_callers,
            4,
            "Virtual Geometry provider/tests should use the schema-v1 render-layer mask API",
        )


if __name__ == "__main__":
    unittest.main()
