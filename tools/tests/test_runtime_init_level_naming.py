import unittest
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.runtime_naming_boundary import (  # noqa: E402
    LEGACY_TOKEN_RE,
    _classify_editor_reference,
    _classify_legacy_reference,
    _decisions_for_term,
    _term_report,
    runtime_naming_boundary_audit,
)


class RuntimeInitLevelNamingTests(unittest.TestCase):
    def test_service_layer_does_not_use_network_server_vocabulary(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        lifecycle = (
            repo_root / "zircon_runtime/src/core/runtime/lifecycle.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("Services,", lifecycle)
        self.assertIn('Self::Services => "Services"', lifecycle)
        self.assertNotIn("Servers", lifecycle)

        for relative_path in (
            "zircon_runtime/src/asset/module.rs",
            "zircon_runtime/src/input/module/descriptor.rs",
            "zircon_runtime/src/platform/module.rs",
        ):
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn("InitLevel::Services", source, relative_path)
            self.assertNotIn("InitLevel::Servers", source, relative_path)

        for relative_path in (
            "zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs",
            "zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs",
        ):
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("ServersModule", source, relative_path)
            self.assertNotIn('"servers"', source, relative_path)

    def test_editor_init_level_is_classified_as_an_editor_host_target(self) -> None:
        report = runtime_naming_boundary_audit(REPO_ROOT)

        lifecycle_debt = [
            row
            for row in report["editor"]["unclassified_locations"]
            if row["path"] == "zircon_runtime/src/core/runtime/lifecycle.rs"
        ]
        self.assertEqual([], lifecycle_debt)

    def test_runtime_editor_metadata_owners_are_explicitly_classified(self) -> None:
        report = runtime_naming_boundary_audit(REPO_ROOT)
        unclassified_paths = {
            row["path"] for row in report["editor"]["unclassified_locations"]
        }

        for relative_path in (
            "zircon_runtime/src/diagnostic_log/level.rs",
            "zircon_runtime/src/scene/components/scene/activation.rs",
            "zircon_runtime/src/scene/components/scene/hierarchy.rs",
            "zircon_runtime/src/scene/components/scene/identity.rs",
            "zircon_runtime/src/scene/components/scene/mesh_renderer.rs",
            "zircon_runtime/src/scene/components/scene/physics.rs",
            "zircon_runtime/src/scene/components/scene/transform.rs",
            "zircon_runtime/src/script/vm/host_interface/descriptor.rs",
            "zircon_runtime/src/script/vm/host_interface/registry.rs",
            "zircon_runtime/src/text/cache/shaped_cache.rs",
            "zircon_runtime/src/text/parallel/shape_pool.rs",
        ):
            self.assertNotIn(relative_path, unclassified_paths)

        self.assertEqual(
            0,
            report["editor"]["unclassified_location_count"],
            report["editor"]["unclassified_locations"],
        )

    def test_editor_owner_classifications_do_not_hide_broader_production_paths(
        self,
    ) -> None:
        scene_component_path = (
            "zircon_runtime/src/scene/components/scene/activation.rs"
        )
        self.assertEqual(
            "scene-reflection-editor-visible-metadata",
            _classify_editor_reference(scene_component_path, ("editor_hint",)),
        )
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(
                scene_component_path,
                ("editor_authoring_state",),
            ),
        )
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(
                "zircon_runtime/src/scene/components/scene_metadata.rs",
                ("editor_hint",),
            ),
        )
        diagnostic_level_path = "zircon_runtime/src/diagnostic_log/level.rs"
        self.assertEqual(
            "curated-runtime-facade-editor-reference",
            _classify_editor_reference(
                diagnostic_level_path,
                ("editor",),
                in_cfg_test_item=True,
            ),
        )
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(
                diagnostic_level_path,
                ("editor_authoring_state",),
            ),
        )
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(
                "zircon_runtime/src/script/vm/unrelated_behavior.rs",
                ("editor",),
            ),
        )
        text_path = "zircon_runtime/src/text/cache/shaped_cache.rs"
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(text_path, ("editor",)),
        )
        self.assertEqual(
            "runtime-text-editor-product-fixture",
            _classify_editor_reference(
                text_path,
                ("editor",),
                in_cfg_test_item=True,
            ),
        )

    def test_legacy_cfg_test_items_do_not_hide_production_naming_debt(self) -> None:
        graphics_path = "zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs"
        scene_path = "zircon_runtime/src/scene/dynamic_scene/document/migration/project_world.rs"

        for relative_path in (graphics_path, scene_path):
            self.assertEqual(
                "test-fixture",
                _classify_legacy_reference(
                    relative_path,
                    in_cfg_test_item=True,
                ),
            )

        self.assertEqual(
            "legacy-runtime-graphics-debt",
            _classify_legacy_reference(graphics_path),
        )
        self.assertEqual(
            "legacy-scene-schema-render-debt",
            _classify_legacy_reference(scene_path),
        )

    def test_legacy_scan_applies_cfg_test_boundary_per_item(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_path = (
                root
                / "zircon_runtime/src/graphics/visibility/cfg_test_boundary.rs"
            )
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """#[cfg(test)]
fn brace_literal_fixture() { let marker = "{"; let legacy_brace_literal = 1; }

fn production_owner() { let legacy_production = 1; }

#[cfg(test)]
fn multiline_fixture()
{
    let legacy_multiline = 1;
}

#[cfg(test)]
fn comment_raw_and_char_fixture() {
    let _raw = r#"{"#;
    let _char = '{';
    /* { */ let legacy_lexical_fixture = 1;
}

#[cfg(test)]
fn array_signature_fixture(
    bytes: [u8; 4],
) {
    let legacy_array_signature = bytes;
}

#[cfg(test)]
fn comparison_signature_fixture(bytes: [u8; (1 < 2) as usize]) {
    let legacy_comparison_signature = bytes;
}

fn production_after_comparison() { let legacy_after_comparison = 1; }

#[cfg(test)]
const LEGACY_TEST: bool = 1 < 2;

fn production_after_const() { let legacy_after_const = 1; }
""",
                encoding="utf-8",
            )

            decisions = _decisions_for_term(root, LEGACY_TOKEN_RE, "legacy")
            report = _term_report(decisions)

        classifications = {
            row.tokens[0]: row.classification for row in decisions
        }
        self.assertEqual(9, report["location_count"])
        self.assertEqual("test-fixture", classifications["legacy_brace_literal"])
        self.assertEqual(
            "legacy-runtime-graphics-debt",
            classifications["legacy_production"],
        )
        self.assertEqual("test-fixture", classifications["legacy_multiline"])
        self.assertEqual("test-fixture", classifications["legacy_lexical_fixture"])
        self.assertEqual("test-fixture", classifications["legacy_array_signature"])
        self.assertEqual(
            "test-fixture",
            classifications["legacy_comparison_signature"],
        )
        self.assertEqual(
            "legacy-runtime-graphics-debt",
            classifications["legacy_after_comparison"],
        )
        self.assertEqual("test-fixture", classifications["LEGACY_TEST"])
        self.assertEqual(
            "legacy-runtime-graphics-debt",
            classifications["legacy_after_const"],
        )
        self.assertEqual(1, report["migration_debt_count"])


if __name__ == "__main__":
    unittest.main()
