import sys
import unittest
from pathlib import Path
from unittest.mock import patch


TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from audit_plugin_structure import build_report, render_markdown  # noqa: E402


class _AuditResult:
    def __init__(self, payload):
        self._payload = payload

    def to_json(self):
        return self._payload


class AuditPluginStructureReportTests(unittest.TestCase):
    def test_report_exposes_feature_provider_package_projection_count(self):
        report = _build_report()

        self.assertEqual(
            2,
            report["summary"]["feature_provider_package_projection_count"],
        )
        markdown = render_markdown(report)
        self.assertIn("- Feature-provider package projections: 2", markdown)

    def test_report_exposes_generated_manifest_header_status(self):
        report = _build_report()

        self.assertEqual(
            0,
            report["summary"]["generated_manifest_header_violations"],
        )
        markdown = render_markdown(report)
        self.assertIn("- Generated manifest header violations: 0", markdown)

    def test_report_exposes_retired_ui_asset_status(self):
        report = _build_report()

        self.assertEqual(0, report["summary"]["retired_ui_asset_files"])
        self.assertEqual(
            "zui-only-clean",
            report["summary"]["zui_only_layout_status"],
        )
        self.assertEqual("classified-and-clear", report["m1_gate_status"])
        markdown = render_markdown(report)
        self.assertIn("- Retired UI asset files: 0", markdown)
        self.assertIn("- ZUI-only layout status: `zui-only-clean`", markdown)


def _build_report():
    with (
        patch(
            "audit_plugin_structure.audit_plugin_manifest_schema",
            return_value=_AuditResult(_manifest_schema()),
        ),
        patch(
            "audit_plugin_structure.audit_plugin_skeleton_conformance",
            return_value=_AuditResult(_skeleton_conformance()),
        ),
        patch(
            "audit_plugin_structure.audit_plugin_registration_conformance",
            return_value=_AuditResult(_registration_conformance()),
        ),
        patch(
            "audit_plugin_structure.audit_plugin_capability_conformance",
            return_value=_AuditResult(_capability_conformance()),
        ),
        patch(
            "audit_plugin_structure.audit_plugin_dependency_boundary",
            return_value=_AuditResult(_dependency_boundary()),
        ),
        patch(
            "audit_plugin_structure.audit_retired_ui_asset_conformance",
            return_value=_AuditResult(_retired_ui_asset_conformance()),
        ),
    ):
        return build_report(Path("repo"))


def _manifest_schema():
    return {
        "expected_manifest_count": 37,
        "manifest_count": 37,
        "missing_plugin_toml": 0,
        "missing_plugin_toml_paths": [],
        "manifest_schema_violations": 0,
        "manifest_schema_violation_details": [],
        "generated_manifest_header_violations": 0,
        "generated_manifest_header_violation_paths": [],
        "feature_provider_package_projection_count": 2,
    }


def _skeleton_conformance():
    return {
        "sample_conformance_status": "sample-clean",
        "core_workspace_dependency_status": "core-workspace-deps-clean",
        "core_workspace_dependency_count": 117,
        "core_workspace_dependency_violation_count": 0,
        "core_workspace_dependency_violations": [],
        "migration_debt_count": 0,
        "migration_debt_detail_count": 0,
        "migration_debt_roots": [],
        "sample_violations": [],
    }


def _registration_conformance():
    return {
        "asset_importer_family_free_function_registration_sites": 0,
        "asset_importer_family_free_function_registration_site_details": [],
        "split_importer_free_function_registration_sites": 0,
        "split_importer_free_function_registration_site_details": [],
        "importer_free_function_registration_sites": 0,
        "runtime_registration_builder_roots": [],
        "runtime_registration_builder_violation_count": 0,
        "runtime_registration_builder_violations": [],
        "m3_t1_gate_status": "family-single-entry-clean",
        "m3_split_importer_gate_status": "split-importer-single-entry-clean",
        "m3_importer_gate_status": "importer-single-entry-clean",
        "m3_t2_runtime_registration_builder_status": (
            "runtime-registration-builder-clean"
        ),
    }


def _capability_conformance():
    return {
        "audited_runtime_root_count": 15,
        "capability_source_mismatches": 0,
        "capability_source_mismatch_details": [],
        "m4_runtime_capability_gate_status": (
            "runtime-capability-single-source-clean"
        ),
        "sdk_builder_mirror_violations": 0,
        "sdk_builder_mirror_violation_details": [],
        "m4_t2_builder_mirror_gate_status": "sdk-builder-mirror-clean",
        "editor_runtime_mirror_root_count": 3,
        "editor_runtime_mirror_violations": 0,
        "editor_runtime_mirror_violation_details": [],
        "d9_editor_runtime_mirror_gate_status": "editor-runtime-mirror-clean",
    }


def _dependency_boundary():
    return {
        "dist_capable_plugin_count": 39,
        "dist_build_matrix_count": 39,
        "distribution_section_violations": 0,
        "distribution_section_violation_details": [],
        "dist_dependency_boundary_violations": 0,
        "dist_dependency_boundary_violation_details": [],
        "m1_dist_dependency_boundary_gate_status": "dist-boundary-clean",
    }


def _retired_ui_asset_conformance():
    return {
        "retired_ui_asset_files": 0,
        "retired_ui_asset_file_paths": [],
        "zui_only_layout_status": "zui-only-clean",
    }


if __name__ == "__main__":
    unittest.main()
