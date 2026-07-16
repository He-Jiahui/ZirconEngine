import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.tech_stack_boundary import (  # noqa: E402
    _is_zr_vm_binding_dependency,
    _is_zr_vm_feature_name,
    _path_is_within,
    tech_stack_boundary_audit,
)
from runtime_structure_audits.plugin_surface_lifecycle_boundary import (  # noqa: E402
    plugin_surface_lifecycle_boundary_audit,
)


class RuntimeTechStackBoundaryTests(unittest.TestCase):
    def test_folder_backed_mirror_guard_is_discovered(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertTrue(report["mirror_docs_guard_present"])
        self.assertNotIn(
            "Runtime 01 tech-stack mirror-doc aggregate guard is missing.",
            report["risks"],
        )

    def test_current_optional_text_and_backend_feature_declarations_are_clean(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_version_anchors"])
        self.assertEqual([], report["dependency_boundary_violations"])
        self.assertEqual(2, report["jolt_feature_slot_count"])
        self.assertEqual(1, report["runtime_jolt_feature_slot_count"])
        self.assertEqual(1, report["physics_jolt_dependency_feature_slot_count"])
        self.assertTrue(report["joltc_sys_optional_dependency_present"])
        self.assertTrue(report["physics_jolt_backend_files_present"])
        self.assertTrue(report["jolt_backend_feature_gated"])
        self.assertTrue(report["runtime_joltc_sys_dependency_absent"])
        self.assertEqual([], report["risks"])

    def test_zr_vm_backend_is_plugin_owned_and_runtime_binding_free(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertTrue(report["zr_vm_plugin_manifest_present"])
        self.assertTrue(report["zr_vm_plugin_backend_feature_present"])
        self.assertGreater(report["zr_vm_plugin_binding_dependency_count"], 0)
        self.assertTrue(report["zr_vm_plugin_binding_dependencies_optional"])
        self.assertTrue(report["zr_vm_plugin_binding_dependencies_external"])
        self.assertTrue(report["zr_vm_plugin_backend_feature_gates_bindings"])
        self.assertTrue(report["runtime_zr_vm_owner_absent"])

    def test_zr_vm_dependency_rename_and_path_traversal_cannot_bypass_owner_guard(
        self,
    ) -> None:
        self.assertTrue(
            _is_zr_vm_binding_dependency(
                "vm_binding",
                {"package": "zr_vm_rust_binding", "optional": True},
            )
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "zr_vm"
            self.assertTrue(_path_is_within(owner / "binding", owner))
            self.assertFalse(_path_is_within(owner / ".." / "other", owner))
        self.assertTrue(_is_zr_vm_feature_name("zr-vm-real-backend"))
        self.assertTrue(_is_zr_vm_feature_name("zr_vm_backend"))
        self.assertTrue(_is_zr_vm_feature_name("backend-zrvm"))
        self.assertFalse(_is_zr_vm_feature_name("backend-jolt"))

    def test_runtime_zr_vm_renamed_dependency_is_reported_as_owner_drift(self) -> None:
        runtime_manifest = REPO_ROOT / "zircon_runtime/Cargo.toml"
        original = runtime_manifest.read_text(encoding="utf-8")
        drifted = original.replace(
            "[dependencies]",
            "[dependencies]\nvm_binding = { package = \"zr_vm_rust_binding\", path = \"../../zr_vm\", optional = true }",
            1,
        )
        real_read_toml = sys.modules[
            "runtime_structure_audits.tech_stack_boundary"
        ]._read_toml

        def read_toml_with_runtime_drift(path: Path) -> dict[str, object]:
            if path == runtime_manifest:
                import tomllib

                return tomllib.loads(drifted)
            return real_read_toml(path)

        with patch(
            "runtime_structure_audits.tech_stack_boundary._read_toml",
            side_effect=read_toml_with_runtime_drift,
        ):
            report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertFalse(report["runtime_zr_vm_owner_absent"])
        self.assertIn(
            "zircon_runtime declares plugin-owned ZrVM backend state",
            report["dependency_boundary_violations"],
        )

    def test_jolt_backend_is_feature_gated_and_plugin_owned(self) -> None:
        backend_root = REPO_ROOT / "zircon_plugins/physics/runtime/src/backend"

        self.assertFalse((backend_root.parent / "backend.rs").exists())
        self.assertIn(
            'JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = "backend-jolt")',
            (backend_root / "selection.rs").read_text(encoding="utf-8"),
        )
        self.assertTrue((backend_root / "jolt/mod.rs").exists())
        self.assertTrue((backend_root / "jolt/native_world.rs").exists())
        self.assertTrue((backend_root / "jolt/runtime.rs").exists())
        self.assertEqual([], tech_stack_boundary_audit(REPO_ROOT)["risks"])

    def test_runtime_06_current_backend_command_and_folder_guard_owners_are_clean(self) -> None:
        report = plugin_surface_lifecycle_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_source_anchors"])
        self.assertEqual([], report["missing_cargo_gate_anchors"])
        self.assertTrue(report["mirror_docs_guard_present"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()
