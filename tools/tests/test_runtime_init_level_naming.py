import unittest
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.runtime_naming_boundary import (  # noqa: E402
    _classify_editor_reference,
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
            "zircon_runtime/src/scene/components/scene.rs",
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
        self.assertEqual(
            "unclassified-runtime-naming-reference",
            _classify_editor_reference(
                "zircon_runtime/src/scene/components/scene.rs",
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


if __name__ == "__main__":
    unittest.main()
