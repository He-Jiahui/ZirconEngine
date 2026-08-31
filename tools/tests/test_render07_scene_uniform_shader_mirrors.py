import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
GRAPHICS_ROOT = REPOSITORY_ROOT / "zircon_runtime" / "src" / "graphics"
CANONICAL_SCENE_UNIFORM = GRAPHICS_ROOT / "shader" / "wgsl" / "zr_scene_runtime.wgsl"
SSAO_DESCRIPTOR = (
    GRAPHICS_ROOT
    / "feature"
    / "builtin_render_feature_descriptor"
    / "feature_descriptors"
    / "screen_space_ambient_occlusion.rs"
)
SCENE_UNIFORM_PATTERN = re.compile(
    r"struct\s+SceneUniform\s*\{(?P<body>.*?)\}", re.DOTALL
)
FIELD_PATTERN = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:", re.MULTILINE)


def scene_uniform_fields(path: Path) -> list[str] | None:
    match = SCENE_UNIFORM_PATTERN.search(path.read_text(encoding="utf-8"))
    if match is None:
        return None
    return FIELD_PATTERN.findall(match.group("body"))


class Render07SceneUniformShaderMirrorsTests(unittest.TestCase):
    maxDiff = None

    def test_production_scene_uniform_mirrors_preserve_canonical_prefix(self) -> None:
        canonical_fields = scene_uniform_fields(CANONICAL_SCENE_UNIFORM)
        self.assertIsNotNone(canonical_fields)
        assert canonical_fields is not None

        history_anchor = "previous_view_proj_unjittered"
        anchor_index = canonical_fields.index(history_anchor)
        fields_at_or_after_anchor = set(canonical_fields[anchor_index:])
        violations: dict[str, list[str]] = {}

        for shader_path in sorted(GRAPHICS_ROOT.rglob("*.wgsl")):
            if shader_path == CANONICAL_SCENE_UNIFORM:
                continue
            fields = scene_uniform_fields(shader_path)
            if fields is None or fields_at_or_after_anchor.isdisjoint(fields):
                continue
            if fields != canonical_fields[: len(fields)]:
                relative_path = shader_path.relative_to(REPOSITORY_ROOT).as_posix()
                violations[relative_path] = fields

        self.assertEqual({}, violations)

    def test_ssao_descriptor_embeds_both_scene_uniform_mirrors(self) -> None:
        descriptor = SSAO_DESCRIPTOR.read_text(encoding="utf-8")
        self.assertIn("ssao_spatial_denoise.wgsl", descriptor)
        self.assertIn("ssao_bilateral_upsample.wgsl", descriptor)


if __name__ == "__main__":
    unittest.main()
