import unittest
from pathlib import Path


class RuntimeAssetArtifactMaterialShaderOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_04_15_asset_artifact_material_shader_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_material_and_shader_cache_payloads_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/asset/artifact/cache_payload/material_shader.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 12)
        self.assertIn("mod material;", owner)
        self.assertIn("mod shader;", owner)
        self.assertIn("pub(super) use material::ArtifactCacheMaterialAsset;", owner)
        self.assertIn("pub(super) use shader::ArtifactCacheShaderAsset;", owner)
        self.assertNotIn("struct ArtifactCache", owner)

        owner_dir = owner_path.with_suffix("")
        material = (owner_dir / "material.rs").read_text(encoding="utf-8")
        shader = (owner_dir / "shader.rs").read_text(encoding="utf-8")
        self.assertLessEqual(len(material.splitlines()), 220)
        self.assertLessEqual(len(shader.splitlines()), 560)

        for anchor in (
            "pub(in super::super) struct ArtifactCacheMaterialAsset",
            "enum ArtifactCacheAlphaMode",
            "struct ArtifactCacheMaterialTextureSlotValue",
            "impl From<&MaterialAsset> for ArtifactCacheMaterialAsset",
        ):
            self.assertIn(anchor, material)
        self.assertNotIn("ArtifactCacheShaderAsset", material)

        for anchor in (
            "pub(in super::super) struct ArtifactCacheShaderAsset",
            "struct ArtifactCacheShaderRenderStateDescriptor",
            "struct ArtifactCacheShaderResourceDescriptor",
            "enum ArtifactCacheRenderShaderDefinitionValue",
            "impl From<&ShaderAsset> for ArtifactCacheShaderAsset",
        ):
            self.assertIn(anchor, shader)
        self.assertNotIn("ArtifactCacheMaterialAsset", shader)

        parent = (owner_path.parent.parent / "cache_payload.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "use material_shader::{ArtifactCacheMaterialAsset, ArtifactCacheShaderAsset};",
            parent,
        )
        self.assertLess(parent.index("Shader(ArtifactCacheShaderAsset)"), parent.index("Material(ArtifactCacheMaterialAsset)"))

        material_struct = material.split(
            "pub(in super::super) struct ArtifactCacheMaterialAsset", 1
        )[1].split("impl From<&MaterialAsset>", 1)[0]
        self._assert_anchors_are_ordered(
            material_struct,
            (
                "name: Option<String>",
                "shader: AssetReference",
                "parent: Option<AssetReference>",
                "base_color: [f32; 4]",
                "alpha_mode: ArtifactCacheAlphaMode",
                "property_values: BTreeMap",
                "texture_slots: BTreeMap",
                "options: BTreeMap",
                "queue: Option<ZMaterialQueueOverride>",
                "validation_diagnostics: Vec<String>",
            ),
        )
        shader_struct = shader.split(
            "pub(in super::super) struct ArtifactCacheShaderAsset", 1
        )[1].split("impl From<&ShaderAsset>", 1)[0]
        self._assert_anchors_are_ordered(
            shader_struct,
            (
                "uri: AssetUri",
                "kind: ShaderAssetKind",
                "source_language: ShaderSourceLanguage",
                "imports: Vec<ArtifactCacheShaderImportRedirectAsset>",
                "shader_defs: Vec<ArtifactCacheRenderShaderDefinitionValue>",
                "render_state: ArtifactCacheShaderRenderStateDescriptor",
                "resources: Vec<ArtifactCacheShaderResourceDescriptor>",
                "material_property_layout: ArtifactCacheMaterialPropertyLayout",
                "editor: ArtifactCacheTomlTable",
                "pipeline_layout: RenderShaderPipelineLayoutDescriptor",
                "validation_diagnostics: Vec<String>",
            ),
        )
        self.assertIn(
            '#[serde(default = "default_artifact_shader_asset_kind")]',
            shader_struct,
        )

    def test_owner_split_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/asset/artifact/cache_payload/material_shader.rs",
            "zircon_runtime/src/asset/artifact/cache_payload/material_shader/material.rs",
            "zircon_runtime/src/asset/artifact/cache_payload/material_shader/shader.rs",
            "tools/tests/test_runtime_asset_artifact_material_shader_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()
