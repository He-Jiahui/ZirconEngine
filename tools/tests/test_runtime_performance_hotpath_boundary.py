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

from runtime_structure_audits.performance_hotpath_boundary import (  # noqa: E402
    performance_hotpath_boundary_audit,
)
from runtime_structure_audits import performance_hotpath_boundary  # noqa: E402

# 调用热路径边界审计器，核对现行后端命令与归档记录对应，测试计数由清单派生。
class RuntimePerformanceHotpathBoundaryTests(unittest.TestCase):
    def test_numbered_archive_and_current_backend_command_supply_doc_evidence(self) -> None:
        report = performance_hotpath_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["missing_cargo_gate_anchors"])

    def test_test_inventory_count_is_derived_from_the_manifest(self) -> None:
        manifest = (
            "zircon_runtime/src/tests/contracts/performance.rs",
        )
        with patch.object(performance_hotpath_boundary, "RUNTIME_07_TEST_FILES", manifest):
            report = performance_hotpath_boundary_audit(REPO_ROOT)

        self.assertEqual(len(manifest), report["expected_test_file_count"])
        self.assertEqual(len(manifest), len(report["test_files"]))

    def test_fixed_runtime_07_history_reads_verified_archive_identity(self) -> None:
        report = performance_hotpath_boundary_audit(REPO_ROOT)
        records = report["historical_archive_readback"]
        self.assertEqual(1, len(records))
        record = records[0]
        path = performance_hotpath_boundary.RUNTIME_07_OUTPUT_ARCHIVE
        length, sha256 = performance_hotpath_boundary.RUNTIME_07_HISTORICAL_IDENTITIES[path]
        self.assertEqual(path, record["path"])
        self.assertTrue(record["historical_only"])
        self.assertEqual("verified", record["status"])
        self.assertEqual(length, record["actual_length"])
        self.assertEqual(sha256, record["actual_sha256"])

    def test_archive_failure_is_not_masked_by_checkout_or_current_document_anchors(self) -> None:
        boundary = performance_hotpath_boundary
        current_plan = REPO_ROOT / "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
        complete_current_doc = "\n".join(
            (*boundary.RUNTIME_07_DOC_ANCHORS, *boundary.CARGO_GATE_ANCHORS)
        )
        original_read = boundary._read_text

        def read_current(path: Path) -> str:
            return complete_current_doc if path == current_plan else original_read(path)

        length, _ = boundary.RUNTIME_07_HISTORICAL_IDENTITIES[boundary.RUNTIME_07_OUTPUT_ARCHIVE]
        cases = (
            ("missing", None, "archive-read-error"),
            ("wrong-length", b"tampered history", "length-mismatch"),
            ("wrong-sha256", b"x" * length, "sha256-mismatch"),
        )
        with tempfile.TemporaryDirectory() as directory:
            archive_root = Path(directory)
            target = archive_root / boundary.RUNTIME_07_OUTPUT_ARCHIVE
            for name, payload, reason in cases:
                with self.subTest(case=name):
                    if payload is not None:
                        target.parent.mkdir(parents=True, exist_ok=True)
                        target.write_bytes(payload)
                    with patch.object(boundary, "_read_text", side_effect=read_current):
                        report = performance_hotpath_boundary_audit(
                            REPO_ROOT, archive_root=archive_root
                        )
                    self.assertEqual([], report["missing_doc_anchors"])
                    self.assertEqual([], report["missing_cargo_gate_anchors"])
                    self.assertIn(
                        "Runtime 07 fixed historical output archive readback failed.",
                        report["risks"],
                    )
                    record = report["historical_archive_readback"][0]
                    self.assertTrue(record["historical_only"])
                    self.assertEqual("unavailable", record["status"])
                    self.assertEqual(reason, record["reason"])


    def test_current_semantic_owners_and_wiring_are_present(self) -> None:
        gate = performance_hotpath_boundary.large_file_ownership_gate({}, 1000)
        report = performance_hotpath_boundary_audit(REPO_ROOT, gate)
        for key in ("missing_source_files", "missing_frame_span_anchors",
                    "missing_query_counter_anchors", "missing_extract_counter_anchors",
                    "missing_asset_worker_anchors", "missing_animation_scene_anchors",
                    "missing_hotspot_guard_anchors"):
            with self.subTest(key=key):
                self.assertEqual([], report[key])

    def test_split_diagnostic_owner_is_required(self) -> None:
        boundary = performance_hotpath_boundary
        original_exists = Path.exists
        owner = REPO_ROOT / "zircon_runtime/src/asset/pipeline/worker_pool/diagnostics.rs"
        with patch.object(Path, "exists", lambda path: False if path == owner else original_exists(path)):
            report = boundary.performance_hotpath_boundary_audit(
                REPO_ROOT, boundary.large_file_ownership_gate({}, 1000))
        self.assertIn(owner.relative_to(REPO_ROOT).as_posix(), report["missing_source_files"])
        self.assertTrue(report["missing_asset_worker_anchors"])

    def test_current_wiring_and_cache_identity_cannot_be_masked_by_other_sources(self) -> None:
        boundary = performance_hotpath_boundary
        original_read = boundary._read_text
        cases = (
            ("zircon_runtime/src/dynamic_api/session/extract_cache.rs",
             "lifecycle_visibility_revision: world.lifecycle_visibility_revision()", "missing_extract_counter_anchors"),
            ("zircon_runtime/src/scene/level_system.rs",
             "world.reset_ecs_frame_performance_diagnostics();", "missing_query_counter_anchors"),
            ("zircon_runtime/src/scene/module/world_driver.rs",
             ".publish(core, world_time.outer_frame_index());", "missing_query_counter_anchors"),
            ("zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs",
             "publish_events_to_world(world, events);", "missing_animation_scene_anchors"),
            ("zircon_runtime/src/scene/ecs/schedule_runner.rs",
             'SystemStage::RenderExtract => "runtime_frame_schedule_stage.RenderExtract"', "missing_frame_span_anchors"),
        )
        for relative, snippet, key in cases:
            with self.subTest(owner=relative, snippet=snippet):
                owner = REPO_ROOT / relative
                self.assertIn(snippet, original_read(owner))
                def without_wiring(path: Path) -> str:
                    text = original_read(path)
                    return text.replace(snippet, "REMOVED_REQUIRED_WIRING") if path == owner else text
                with patch.object(boundary, "_read_text", side_effect=without_wiring):
                    report = boundary.performance_hotpath_boundary_audit(
                        REPO_ROOT, boundary.large_file_ownership_gate({}, 1000))
                self.assertIn(snippet, report[key])

    def test_split_owner_category_may_have_zero_large_files(self) -> None:
        boundary = performance_hotpath_boundary
        gate = boundary.large_file_ownership_gate(
            {"zircon_runtime": [("zircon_runtime/src/scene/world/world.rs", 1001)]}, 1000)
        self.assertNotIn("runtime-framework-render", gate["classification_counts"])
        report = boundary.performance_hotpath_boundary_audit(REPO_ROOT, gate)
        self.assertEqual([], report["missing_large_file_owner_classes"])
        self.assertEqual(0, report["large_file_unclassified_hotspot_count"])

    def test_actual_hotspot_with_unknown_owner_is_rejected(self) -> None:
        boundary = performance_hotpath_boundary
        from runtime_structure_audits import large_file_ownership
        with patch.object(large_file_ownership, "large_file_owner_class", return_value="unknown-owner"):
            gate = boundary.large_file_ownership_gate(
                {"zircon_runtime": [("zircon_runtime/src/scene/world/world.rs", 1001)]}, 1000)
        self.assertEqual(1, gate["unclassified_hotspot_count"])
        report = boundary.performance_hotpath_boundary_audit(REPO_ROOT, gate)
        self.assertEqual(["unknown-owner"], report["missing_large_file_owner_classes"])
        self.assertIn(
            "Runtime 07 owner-budgeted optimization gate has unclassified large production-file hotspots.",
            report["risks"],
        )


if __name__ == "__main__":
    unittest.main()
