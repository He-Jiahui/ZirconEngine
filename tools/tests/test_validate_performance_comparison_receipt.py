from __future__ import annotations

import base64
import hashlib
import json
import struct
import subprocess
import sys
import unittest
import zlib
from pathlib import Path
from tempfile import TemporaryDirectory

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from tools.validate_performance_comparison_receipt import (
    PERFORMANCE_COMPARISON_RECEIPT_SCHEMA,
    sign_performance_comparison_receipt,
    validate_performance_comparison_receipt,
)


class ValidatePerformanceComparisonReceiptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.private_key = Ed25519PrivateKey.generate()
        self.acceptor_id = "independent-performance-review"
        self.trusted_acceptors = {
            self.acceptor_id: self.private_key.public_key().public_bytes(
                serialization.Encoding.Raw,
                serialization.PublicFormat.Raw,
            )
        }

    def test_accepts_an_independently_signed_comparable_improvement_without_authorizing_baseline_update(
        self,
    ):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key, baseline_update=False
            )

            result = validate_performance_comparison_receipt(
                receipt,
                baseline_report=baseline_path,
                candidate_report=candidate_path,
                baseline_artifact_root=root,
                candidate_artifact_root=root,
                trusted_acceptor_public_keys=self.trusted_acceptors,
            )

            self.assertFalse(result.allows_baseline_update)
            self.assertEqual("gpu_frame_ns", result.metric)
            self.assertEqual(200.0, result.baseline_median)
            self.assertEqual(180.0, result.candidate_median)
            self.assertEqual(-0.1, result.median_ratio)
            self.assertEqual(230.0, result.baseline_p95)
            self.assertEqual(210.0, result.candidate_p95)
            self.assertEqual(-20.0 / 230.0, result.p95_ratio)

    def test_rejects_a_self_asserted_baseline_update_without_a_coordinator_promotion_contract(
        self,
    ):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                candidate_samples=([300.0] * 112) + ([330.0] * 6),
                baseline_update=False,
            )
            receipt.pop("signature")
            receipt["comparison"]["budget"] = {
                "max_regression_ratio": 1.0,
                "max_regression_absolute": 1_000.0,
                "max_p95_regression_ratio": 1.0,
            }
            receipt["acceptance"]["baseline_update"] = True
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(RuntimeError, "Coordinator promotion contract"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_candidate_that_exceeds_its_regression_budget(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                candidate_samples=([230.0] * 112) + ([260.0] * 6),
            )

            with self.assertRaisesRegex(RuntimeError, "exceeds the regression budget"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_candidate_with_a_p95_regression_despite_a_median_improvement(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                candidate_samples=([190.0] * 112) + ([300.0] * 6),
            )

            with self.assertRaisesRegex(RuntimeError, "exceeds the regression budget"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_candidate_with_an_inconclusive_p95_confidence_bound(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                baseline_samples=([200.0] * 112) + ([230.0] * 6),
                candidate_samples=([180.0] * 112) + ([225.0] * 6),
            )

            with self.assertRaisesRegex(
                RuntimeError, "P95 confidence upper bound exceeds the regression budget"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_receipt_signed_by_the_candidate_capture_session(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            candidate_key = Ed25519PrivateKey.generate()
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                acceptor_id="candidate-capture-session",
                signer_private_key=candidate_key,
            )
            trusted_acceptors = {
                "candidate-capture-session": candidate_key.public_key().public_bytes(
                    serialization.Encoding.Raw,
                    serialization.PublicFormat.Raw,
                )
            }
            with self.assertRaisesRegex(RuntimeError, "independent of both capture sessions"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=trusted_acceptors,
                )

    def test_rejects_a_tampered_signed_receipt_and_report_hash_mismatch(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            receipt["scenario"]["input_fingerprint"] = "f" * 64

            with self.assertRaisesRegex(RuntimeError, "signature is invalid"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

            modified_candidate = _sidecar(
                "candidate", ([180.0] * 112) + ([210.0] * 6)
            )
            modified_candidate["source"]["revision"] = "candidate-rewritten"
            _write_json(candidate_path, modified_candidate)
            original_receipt = _read_json(root / "comparison-receipt.json")
            with self.assertRaisesRegex(RuntimeError, "report_sha256 does not match"):
                validate_performance_comparison_receipt(
                    original_receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_retains_a_signed_rejection_without_authorizing_baseline_update(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                decision="rejected",
                baseline_update=False,
            )

            result = validate_performance_comparison_receipt(
                receipt,
                baseline_report=baseline_path,
                candidate_report=candidate_path,
                baseline_artifact_root=root,
                candidate_artifact_root=root,
                trusted_acceptor_public_keys=self.trusted_acceptors,
            )

            self.assertFalse(result.allows_baseline_update)

    def test_rejects_environment_drift_and_non_reproducible_sample_effects(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            receipt.pop("signature")
            receipt["environment"]["driver_version"] = "different-driver"
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(RuntimeError, "driver does not match reports"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

            receipt.pop("signature")
            receipt["environment"]["driver_version"] = "fixture-driver"
            receipt["comparison"]["effect"]["candidate_median"] = 181.0
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(RuntimeError, "candidate_median is not reproducible"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_resigned_scenario_input_fingerprint_not_bound_to_workload(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            receipt.pop("signature")
            receipt["scenario"]["input_fingerprint"] = "0" * 64
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(
                RuntimeError, "input fingerprint is not bound to reports"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_resigned_scenario_id_not_bound_to_workload(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            receipt.pop("signature")
            receipt["scenario"]["id"] = "render19.control_shared_material.bindless"
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(RuntimeError, "scenario ID is not bound to reports"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_a_confidence_interval_that_hides_resampled_variance(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                baseline_samples=([200.0] * 60) + ([400.0] * 58),
                candidate_samples=[180.0] * 118,
            )

            with self.assertRaisesRegex(
                RuntimeError, "does not contain bootstrap interval"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_an_accepted_comparison_with_a_valid_but_inconclusive_median_interval(
        self,
    ):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                baseline_samples=([100.0] * 58) + ([200.0] * 60),
                candidate_samples=[200.0] * 118,
            )
            receipt.pop("signature")
            receipt["comparison"]["confidence"] = {
                "method": "bootstrap-percentile-v1",
                "level": 0.95,
                "lower_ratio": -1.0,
                "upper_ratio": 2.0,
            }
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(
                RuntimeError, "median confidence upper bound exceeds the regression budget"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_cli_emits_verified_median_and_p95_comparison_data(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, _ = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            public_key = self.trusted_acceptors[self.acceptor_id]
            completed = subprocess.run(
                [
                    sys.executable,
                    "-B",
                    "tools/validate_performance_comparison_receipt.py",
                    str(root / "comparison-receipt.json"),
                    str(baseline_path),
                    str(candidate_path),
                    "--baseline-artifact-root",
                    str(root),
                    "--candidate-artifact-root",
                    str(root),
                    "--acceptor-public-key",
                    f"{self.acceptor_id}={base64.b64encode(public_key).decode('ascii')}",
                ],
                check=False,
                capture_output=True,
                cwd=Path(__file__).parents[2],
                text=True,
            )

            self.assertEqual(0, completed.returncode, completed.stderr)
            output = json.loads(completed.stdout)
            self.assertEqual(200.0, output["baseline_median"])
            self.assertEqual(180.0, output["candidate_median"])
            self.assertEqual(230.0, output["baseline_p95"])
            self.assertEqual(210.0, output["candidate_p95"])
            self.assertEqual(-20.0 / 230.0, output["p95_ratio"])
            self.assertFalse(output["allows_baseline_update"])

    def test_rejects_a_raw_artifact_mutated_after_receipt_signing(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            (root / "candidate-warm.rdc").write_bytes(b"replacement-capture")

            with self.assertRaisesRegex(
                RuntimeError, "candidate.artifact_sha256.rdc_warm_path does not match"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_reusing_one_report_as_baseline_and_candidate(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            shared_samples = ([200.0] * 112) + ([230.0] * 6)
            baseline_path, _, receipt = _write_receipt_bundle(
                root,
                signer_private_key=self.private_key,
                candidate_samples=shared_samples,
            )
            receipt.pop("signature")
            receipt["candidate"] = dict(receipt["baseline"])
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(RuntimeError, "reports must be distinct"):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=baseline_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )

    def test_rejects_reusing_a_capture_validation_ticket(self):
        with TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            baseline_path, candidate_path, receipt = _write_receipt_bundle(
                root, signer_private_key=self.private_key
            )
            candidate = _read_json(candidate_path)
            candidate["source"]["validation_ticket"] = "baseline-ticket"
            _write_json(candidate_path, candidate)
            receipt.pop("signature")
            receipt["candidate"]["report_sha256"] = _sha256_file(candidate_path)
            receipt = sign_performance_comparison_receipt(receipt, self.private_key)

            with self.assertRaisesRegex(
                RuntimeError, "capture validation tickets must be distinct"
            ):
                validate_performance_comparison_receipt(
                    receipt,
                    baseline_report=baseline_path,
                    candidate_report=candidate_path,
                    baseline_artifact_root=root,
                    candidate_artifact_root=root,
                    trusted_acceptor_public_keys=self.trusted_acceptors,
                )


def _write_receipt_bundle(
    root: Path,
    *,
    signer_private_key: Ed25519PrivateKey,
    baseline_samples: list[float] | None = None,
    candidate_samples: list[float] | None = None,
    acceptor_id: str = "independent-performance-review",
    decision: str = "accepted",
    baseline_update: bool = False,
) -> tuple[Path, Path, dict]:
    baseline_samples = baseline_samples or (([200.0] * 112) + ([230.0] * 6))
    candidate_samples = candidate_samples or (([180.0] * 112) + ([210.0] * 6))
    baseline_path = root / "baseline.json"
    candidate_path = root / "candidate.json"
    baseline = _sidecar("baseline", baseline_samples)
    candidate = _sidecar("candidate", candidate_samples)
    _write_artifacts(root, baseline)
    _write_artifacts(root, candidate)
    _write_json(baseline_path, baseline)
    _write_json(candidate_path, candidate)
    receipt = _receipt(
        baseline_path,
        candidate_path,
        baseline,
        candidate,
        baseline_samples,
        candidate_samples,
        acceptor_id=acceptor_id,
        decision=decision,
        baseline_update=baseline_update,
    )
    signed = sign_performance_comparison_receipt(receipt, signer_private_key)
    _write_json(root / "comparison-receipt.json", signed)
    return baseline_path, candidate_path, signed


def _receipt(
    baseline_path: Path,
    candidate_path: Path,
    baseline: dict,
    candidate: dict,
    baseline_samples: list[float],
    candidate_samples: list[float],
    *,
    acceptor_id: str,
    decision: str,
    baseline_update: bool,
) -> dict:
    baseline_median, baseline_p95, baseline_mad = _statistics(baseline_samples)
    candidate_median, candidate_p95, candidate_mad = _statistics(candidate_samples)
    median_ratio = (candidate_median - baseline_median) / baseline_median
    p95_ratio = (candidate_p95 - baseline_p95) / baseline_p95
    adapter_fingerprint = _sha256_json(baseline["adapter"])
    return {
        "schema": PERFORMANCE_COMPARISON_RECEIPT_SCHEMA,
        "scenario": {
            "id": "render19.stress_unique_materials.bindless",
            "input_fingerprint": _scenario_input_fingerprint(baseline["workload"]),
            "workload_fingerprint": _sha256_json(baseline["workload"]),
        },
        "baseline": {
            "report_sha256": _sha256_file(baseline_path),
            "artifact_sha256": _artifact_hashes(baseline_path.parent, baseline),
            "build_set": _build_set(baseline["source"]),
        },
        "candidate": {
            "report_sha256": _sha256_file(candidate_path),
            "artifact_sha256": _artifact_hashes(candidate_path.parent, candidate),
            "build_set": _build_set(candidate["source"]),
        },
        "environment": {
            "hardware_fingerprint": "b" * 64,
            "adapter_fingerprint": adapter_fingerprint,
            "os_version": "Windows 11 24H2",
            "driver_version": "fixture-driver",
            "power_policy": "ac-high-performance",
        },
        "comparison": {
            "metric": "gpu_frame_ns",
            "unit": "ns",
            "direction": "lower_is_better",
            "method": "raw-sample-distribution-v1",
            "baseline_samples": baseline_samples,
            "candidate_samples": candidate_samples,
            "effect": {
                "baseline_median": baseline_median,
                "candidate_median": candidate_median,
                "median_ratio": median_ratio,
                "baseline_p95": baseline_p95,
                "candidate_p95": candidate_p95,
                "p95_ratio": p95_ratio,
                "baseline_mad": baseline_mad,
                "candidate_mad": candidate_mad,
            },
            "confidence": {
                "method": "bootstrap-percentile-v1",
                "level": 0.95,
                "lower_ratio": min(median_ratio, -0.12),
                "upper_ratio": max(median_ratio, -0.08),
            },
            "budget": {
                "max_regression_ratio": 0.02,
                "max_regression_absolute": 4.0,
                "max_p95_regression_ratio": 0.10,
            },
            "decision": decision,
        },
        "acceptance": {
            "acceptor_id": acceptor_id,
            "accepted_at": "2026-08-22T13:30:00+00:00",
            "baseline_update": baseline_update,
        },
    }


def _build_set(source: dict) -> dict:
    return {
        "revision": source["revision"],
        "source_fingerprint": source["source_fingerprint"],
        "target_profile": "profiling",
        "toolchain": "rustc 1.90.0",
        "binary_sha256": "c" * 64,
        "symbol_sha256": "d" * 64,
    }


def _sidecar(label: str, samples: list[float]) -> dict:
    median, p95, mad = _statistics(samples)
    return {
        "schema": "zircon_render_measurement_evidence_v1",
        "source": {
            "revision": ("1" if label == "baseline" else "2") * 40,
            "source_fingerprint": ("e" if label == "baseline" else "f") * 64,
            "session_id": f"{label}-capture-session",
            "validation_ticket": f"{label}-ticket",
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
            "valid_frame_count": len(samples),
            "excluded_pending_timing_count": 120 - len(samples),
            "excluded_unavailable_timing_count": 0,
            "cpu_mesh_encode_ns": {"median": 100.0, "p95": 120.0, "mad": 5.0},
            "gpu_frame_ns": {"median": median, "p95": p95, "mad": mad},
            "board_power_w": {"median": 80.0, "p95": 85.0, "mad": 2.0},
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
            "png_path": f"{label}.png",
            "png_pixel_comparison": {
                "passed": True,
                "max_channel_error": 0,
                "reason": "fixture exact RGBA match",
            },
            "rdc_cold_path": f"{label}-cold.rdc",
            "rdc_warm_path": f"{label}-warm.rdc",
            "graph_dump_path": f"{label}-graph.json",
        },
        "decision": {
            "noise_threshold": {"median": 0.02, "p95": 0.02, "mad": 0.02},
            "control_result": "not_worse",
            "stress_result": "improved",
            "accepted_for_default": True,
            "rationale": "fixture validates comparison-receipt binding",
        },
    }


def _statistics(samples: list[float]) -> tuple[float, float, float]:
    ordered = sorted(samples)
    midpoint = len(ordered) // 2
    median = (ordered[midpoint - 1] + ordered[midpoint]) / 2.0
    p95 = ordered[(95 * len(ordered) + 99) // 100 - 1]
    deviations = sorted(abs(value - median) for value in ordered)
    mad = (deviations[midpoint - 1] + deviations[midpoint]) / 2.0
    return median, p95, mad


def _sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _sha256_json(value: object) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def _scenario_input_fingerprint(workload: dict) -> str:
    return _sha256_json(
        {
            field: workload[field]
            for field in (
                "name",
                "variant",
                "resolution",
                "quality_profile",
                "camera_fingerprint",
            )
        }
    )


def _artifact_hashes(root: Path, sidecar: dict) -> dict[str, str]:
    return {
        field: _sha256_file(root / sidecar["artifacts"][field])
        for field in ("png_path", "rdc_cold_path", "rdc_warm_path", "graph_dump_path")
    }


def _write_artifacts(root: Path, sidecar: dict) -> None:
    artifacts = sidecar["artifacts"]
    (root / artifacts["png_path"]).write_bytes(_minimal_png())
    (root / artifacts["rdc_cold_path"]).write_bytes(b"cold-rdc-fixture")
    (root / artifacts["rdc_warm_path"]).write_bytes(b"warm-rdc-fixture")
    (root / artifacts["graph_dump_path"]).write_text("{}", encoding="utf-8")


def _minimal_png() -> bytes:
    header = struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0)
    pixels = b"\x00\x00\x00\x00\xff"
    return b"\x89PNG\r\n\x1a\n" + _png_chunk(b"IHDR", header) + _png_chunk(
        b"IDAT", zlib.compress(pixels)
    ) + _png_chunk(b"IEND", b"")


def _png_chunk(chunk_type: bytes, payload: bytes) -> bytes:
    checksum = zlib.crc32(chunk_type + payload) & 0xFFFFFFFF
    return struct.pack(">I", len(payload)) + chunk_type + payload + struct.pack(">I", checksum)


def _write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


def _read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))
