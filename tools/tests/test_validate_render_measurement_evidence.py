import json
import binascii
import struct
import unittest
import zlib
from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.validate_render_measurement_evidence import (
    RENDER_MEASUREMENT_EVIDENCE_SCHEMA,
    validate_render_measurement_evidence,
)


class ValidateRenderMeasurementEvidenceTests(unittest.TestCase):
    def test_accepts_a_complete_measured_bindless_sidecar_with_real_artifact_files(self):
        with _controlled_temporary_directory() as temporary_directory:
            root = Path(temporary_directory)
            report_path = root / "stress-bindless.json"
            _write_artifacts(root)
            _write_sidecar(report_path, _valid_sidecar())

            evidence = validate_render_measurement_evidence(
                report_path, require_artifacts=True, artifact_root=root
            )

            self.assertEqual("stress_unique_materials", evidence.workload_name)
            self.assertEqual("bindless", evidence.variant)
            self.assertTrue(evidence.accepted_for_default)

    def test_rejects_material_bind_aggregate_that_does_not_match_its_pass_split(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "bad-split.json"
            sidecar = _valid_sidecar()
            sidecar["material_binds"]["aggregate_set_count"] += 1
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "aggregate_set_count"):
                validate_render_measurement_evidence(report_path)

    def test_rejects_mismatched_renderdoc_calibration_claim(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "bad-calibration.json"
            sidecar = _valid_sidecar()
            sidecar["calibration"]["renderdoc_group2_event_count"] += 1
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "calibration.matched"):
                validate_render_measurement_evidence(report_path)

            sidecar = _valid_sidecar()
            sidecar["calibration"]["renderdoc_group2_event_count"] = 8
            sidecar["calibration"]["counter_set_count"] = 8
            _write_sidecar(report_path, sidecar)
            with self.assertRaisesRegex(RuntimeError, "aggregate material-bind"):
                validate_render_measurement_evidence(report_path)

    def test_rejects_default_acceptance_without_a_measured_power_observation(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "power-unavailable.json"
            sidecar = _valid_sidecar()
            sidecar["observations"]["board_power_w"] = "power_unavailable"
            sidecar["observations"]["power_telemetry"]["probe"] = "unavailable"
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "measured board_power_w"):
                validate_render_measurement_evidence(report_path)

    def test_rejects_missing_or_non_png_artifacts_when_artifact_checks_are_requested(self):
        with _controlled_temporary_directory() as temporary_directory:
            root = Path(temporary_directory)
            report_path = root / "missing-png.json"
            _write_sidecar(report_path, _valid_sidecar())

            with self.assertRaisesRegex(RuntimeError, "artifacts.png_path is unavailable"):
                validate_render_measurement_evidence(
                    report_path, require_artifacts=True, artifact_root=root
                )

            _write_artifacts(root, png_contents=b"not a png")
            with self.assertRaisesRegex(RuntimeError, "invalid signature"):
                validate_render_measurement_evidence(
                    report_path, require_artifacts=True, artifact_root=root
                )

            _write_artifacts(root, png_contents=b"\x89PNG\r\n\x1a\n")
            with self.assertRaisesRegex(RuntimeError, "missing IEND"):
                validate_render_measurement_evidence(
                    report_path, require_artifacts=True, artifact_root=root
                )

    def test_rejects_non_protocol_workloads_and_paths_that_escape_artifact_root(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "invalid-workload.json"
            sidecar = _valid_sidecar()
            sidecar["workload"]["name"] = "hand_authored_scene"
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "not a protocol workload"):
                validate_render_measurement_evidence(report_path)

            sidecar = _valid_sidecar()
            sidecar["artifacts"]["png_path"] = "C:outside.png"
            _write_sidecar(report_path, sidecar)
            with self.assertRaisesRegex(RuntimeError, "relative path"):
                validate_render_measurement_evidence(report_path)

    def test_rejects_incomplete_sample_accounting_and_wrong_measurement_window(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "bad-window.json"
            sidecar = _valid_sidecar()
            sidecar["observations"]["valid_frame_count"] = 119
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "account for all 120"):
                validate_render_measurement_evidence(report_path)

            sidecar = _valid_sidecar()
            sidecar["workload"]["sampled_frames"] = 119
            _write_sidecar(report_path, sidecar)
            with self.assertRaisesRegex(RuntimeError, "30 warm-up and 120"):
                validate_render_measurement_evidence(report_path)

    def test_rejects_non_finite_statistics_and_a_p95_below_the_median(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "bad-statistics.json"
            sidecar = _valid_sidecar()
            sidecar["observations"]["gpu_frame_ns"]["median"] = float("inf")
            _write_sidecar(report_path, sidecar)

            with self.assertRaisesRegex(RuntimeError, "non-negative number"):
                validate_render_measurement_evidence(report_path)

            sidecar = _valid_sidecar()
            sidecar["observations"]["gpu_frame_ns"]["p95"] = 199
            _write_sidecar(report_path, sidecar)
            with self.assertRaisesRegex(RuntimeError, "p95 must not be below"):
                validate_render_measurement_evidence(report_path)


def _valid_sidecar() -> dict:
    return {
        "schema": RENDER_MEASUREMENT_EVIDENCE_SCHEMA,
        "source": {
            "revision": "0123456789abcdef",
            "source_fingerprint": "fixture-source-fingerprint",
            "session_id": "render19-test",
            "validation_ticket": "render19-test-ticket",
        },
        "adapter": {
            "name": "Fixture GPU",
            "backend": "Vulkan",
            "driver": "fixture-driver",
            "requested_features": ["TIMESTAMP_QUERY"],
            "limits": {"max_sampled_textures_per_shader_stage": 512},
            "bindless_gate": "eligible",
            "slot_capacity": 512,
        },
        "workload": {
            "name": "stress_unique_materials",
            "variant": "bindless",
            "resolution": {"width": 1280, "height": 720},
            "quality_profile": "render19-measured",
            "camera_fingerprint": "fixture-camera",
            "warmup_frames": 30,
            "sampled_frames": 120,
        },
        "observations": {
            "valid_frame_count": 118,
            "excluded_pending_timing_count": 1,
            "excluded_unavailable_timing_count": 1,
            "cpu_mesh_encode_ns": {"median": 100, "p95": 120, "mad": 5},
            "gpu_frame_ns": {"median": 200, "p95": 230, "mad": 7},
            "board_power_w": {"median": 80, "p95": 85, "mad": 2},
            "power_telemetry": {
                "probe": "available",
                "sampling_interval_ms": 100,
                "ac_power": True,
            },
        },
        "material_binds": {
            "aggregate_set_count": 7,
            "aggregate_skip_count": 13,
            "main_mesh": {"set_count": 5, "skip_count": 11},
            "shadow": {"set_count": 2, "skip_count": 2},
        },
        "calibration": {
            "captured_frame": 42,
            "renderdoc_group2_event_count": 7,
            "counter_set_count": 7,
            "matched": True,
        },
        "artifacts": {
            "png_path": "stress-bindless.png",
            "png_pixel_comparison": {
                "passed": True,
                "max_channel_error": 0,
                "reason": "opaque workload exact RGBA match",
            },
            "rdc_cold_path": "stress-bindless-cold.rdc",
            "rdc_warm_path": "stress-bindless-warm.rdc",
            "graph_dump_path": "stress-bindless-graph.json",
        },
        "decision": {
            "noise_threshold": {"median": 0.02, "p95": 0.02, "mad": 0.02},
            "control_result": "not_worse",
            "stress_result": "improved",
            "accepted_for_default": True,
            "rationale": "fixture only validates the schema and decision gates",
        },
    }


def _write_sidecar(path: Path, sidecar: dict) -> None:
    path.write_text(json.dumps(sidecar), encoding="utf-8")


def _write_artifacts(root: Path, png_contents: bytes | None = None) -> None:
    (root / "stress-bindless.png").write_bytes(
        png_contents if png_contents is not None else _minimal_png(1, 1)
    )
    (root / "stress-bindless-cold.rdc").write_bytes(b"rdc-fixture")
    (root / "stress-bindless-warm.rdc").write_bytes(b"rdc-fixture")
    (root / "stress-bindless-graph.json").write_text("{}", encoding="utf-8")


def _minimal_png(width: int, height: int) -> bytes:
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    pixels = b"\x00" + (b"\x00\x00\x00\xff" * width * height)
    return b"\x89PNG\r\n\x1a\n" + _png_chunk(b"IHDR", header) + _png_chunk(
        b"IDAT", zlib.compress(pixels)
    ) + _png_chunk(b"IEND", b"")


def _png_chunk(chunk_type: bytes, payload: bytes) -> bytes:
    crc = binascii.crc32(chunk_type + payload) & 0xFFFFFFFF
    return struct.pack(">I", len(payload)) + chunk_type + payload + struct.pack(">I", crc)


@contextmanager
def _controlled_temporary_directory():
    workspace_root = Path(__file__).resolve().parents[2]
    artifact_parent = workspace_root / "docs/tests/runtime/render/.measurement-validator-test-artifacts"
    artifact_parent.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(dir=artifact_parent) as temporary_directory:
        yield temporary_directory
