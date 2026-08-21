import hashlib
import unittest
from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.zircon_validate_shader_pbr_gpu_timing_evidence import (
    GPU_TIMING_EVIDENCE_SCHEMA,
    GPU_TIMING_MEASURED_SAMPLE_COUNT,
    GPU_TIMING_WARMUP_SAMPLE_COUNT,
    OPTIONAL_DIRECT_GPU_PASSES,
    REQUIRED_HDRI_DIRECT_GPU_PASSES,
    validate_gpu_timing_evidence,
)


class ZirconValidateShaderPbrGpuTimingEvidenceTests(unittest.TestCase):
    def test_accepts_a_calibrated_distribution_with_raw_samples_and_aggregates(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)

            evidence = validate_gpu_timing_evidence(report_path, screenshot_path)

            self.assertEqual(7, evidence.screenshot_frame_generation)
            self.assertEqual(13, evidence.first_measured_frame_generation)
            self.assertEqual(43, evidence.last_measured_frame_generation)
            self.assertEqual(31, evidence.measured_sample_count)
            self.assertEqual(
                {
                    "direct_gpu_scene_upload": 0,
                    "direct_output_transfer": 29,
                    "direct_overlays": 30,
                    "direct_scene_content": 28,
                },
                evidence.pass_median_gpu_time_us,
            )
            self.assertEqual(87, evidence.total_distribution.median_us)
            self.assertEqual(129, evidence.total_distribution.p95_us)

    def test_rejects_an_aggregate_that_does_not_match_the_retained_samples(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            _replace_once(
                report_path,
                "pass.direct_scene_content.p95_us=42\n",
                "pass.direct_scene_content.p95_us=41\n",
            )

            with self.assertRaisesRegex(RuntimeError, "aggregate does not match"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_a_downgraded_warmup_or_sample_policy(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            _replace_once(report_path, "warmup_sample_count=5\n", "warmup_sample_count=4\n")
            with self.assertRaisesRegex(RuntimeError, "warmup policy was downgraded"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(report_path, _distribution_report(screenshot_path))
            _replace_once(
                report_path, "measured_sample_count=31\n", "measured_sample_count=30\n"
            )
            with self.assertRaisesRegex(RuntimeError, "sample policy was downgraded"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_a_generation_gap_in_the_retained_samples(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            _replace_once(
                report_path,
                "sample.010.frame_generation=23\n",
                "sample.010.frame_generation=24\n",
            )

            with self.assertRaisesRegex(RuntimeError, "not consecutive"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_missing_or_unknown_pass_coverage(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            _replace_once(
                report_path,
                "pass_coverage=direct_gpu_scene_upload,direct_output_transfer,direct_overlays,direct_scene_content\n",
                "pass_coverage=direct_gpu_scene_upload,direct_output_transfer,direct_overlays\n",
            )
            with self.assertRaisesRegex(RuntimeError, "direct_scene_content"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(report_path, _distribution_report(screenshot_path))
            _replace_once(
                report_path,
                "pass_coverage=direct_gpu_scene_upload,direct_output_transfer,direct_overlays,direct_scene_content\n",
                "pass_coverage=direct_gpu_scene_upload,direct_output_transfer,direct_overlays,direct_scene_content,fabricated_total\n",
            )
            with self.assertRaisesRegex(RuntimeError, "unknown direct pass"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_a_missing_mismatched_or_replaced_screenshot(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(report_path, _distribution_report(screenshot_path))
            screenshot_path.unlink()
            with self.assertRaisesRegex(RuntimeError, "screenshot is unavailable"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            screenshot_path.write_bytes(b"fixture")
            _replace_once(report_path, "screenshot=pbr-ready.png\n", "screenshot=other.png\n")
            with self.assertRaisesRegex(RuntimeError, "does not match"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(report_path, _distribution_report(screenshot_path))
            screenshot_path.write_bytes(b"replacement")
            with self.assertRaisesRegex(RuntimeError, "SHA-256 does not match"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_duplicate_malformed_or_overflow_fields(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            report_path.write_text(
                report_path.read_text(encoding="utf-8") + "status=measured\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(RuntimeError, "repeats a field"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(report_path, _distribution_report(screenshot_path))
            _replace_once(
                report_path,
                "sample.000.pass.direct_scene_content_us=13\n",
                "sample.000.pass.direct_scene_content_us=18446744073709551616\n",
            )
            with self.assertRaisesRegex(RuntimeError, "out of range"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_fields_outside_the_declared_distribution(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path, screenshot_path = _write_valid_fixture(temporary_directory)
            report_path.write_text(
                report_path.read_text(encoding="utf-8")
                + "sample.031.frame_generation=44\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(RuntimeError, "exact field set"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(report_path, _distribution_report(screenshot_path))
            report_path.write_text(
                report_path.read_text(encoding="utf-8")
                + "pass.direct_ui.median_us=1\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(RuntimeError, "exact field set"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_contract_keeps_sampling_policy_and_required_passes_explicit(self):
        self.assertEqual(5, GPU_TIMING_WARMUP_SAMPLE_COUNT)
        self.assertEqual(31, GPU_TIMING_MEASURED_SAMPLE_COUNT)
        self.assertEqual(
            (
                "direct_gpu_scene_upload",
                "direct_scene_content",
                "direct_output_transfer",
                "direct_overlays",
            ),
            REQUIRED_HDRI_DIRECT_GPU_PASSES,
        )
        self.assertEqual(("direct_realtime_ibl", "direct_ui"), OPTIONAL_DIRECT_GPU_PASSES)


def _write_valid_fixture(temporary_directory: str) -> tuple[Path, Path]:
    report_path = Path(temporary_directory) / "gpu-timing.txt"
    screenshot_path = Path(temporary_directory) / "pbr-ready.png"
    screenshot_path.write_bytes(b"fixture")
    _write_report(report_path, _distribution_report(screenshot_path))
    return report_path, screenshot_path


def _write_report(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")


def _replace_once(path: Path, old: str, new: str) -> None:
    contents = path.read_text(encoding="utf-8")
    assert contents.count(old) == 1
    path.write_text(contents.replace(old, new), encoding="utf-8")


def _screenshot_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _distribution_report(screenshot_path: Path) -> str:
    pass_names = tuple(
        sorted(
            (
                "direct_gpu_scene_upload",
                "direct_scene_content",
                "direct_output_transfer",
                "direct_overlays",
            )
        )
    )
    samples = []
    for index, generation in enumerate(range(13, 44)):
        values = {
            "direct_gpu_scene_upload": 0,
            "direct_scene_content": generation,
            "direct_output_transfer": generation + 1,
            "direct_overlays": generation + 2,
        }
        samples.append((index, generation, values))
    lines = [
        f"schema={GPU_TIMING_EVIDENCE_SCHEMA}",
        "status=measured",
        f"screenshot={screenshot_path.name}",
        f"screenshot_sha256={_screenshot_sha256(screenshot_path)}",
        "screenshot_frame_generation=7",
        "warmup_sample_count=5",
        "warmup_first_frame_generation=8",
        "warmup_last_frame_generation=12",
        "measured_sample_count=31",
        "first_measured_frame_generation=13",
        "last_measured_frame_generation=43",
        "timestamp_period_ns_bits=1065353216",
        "timestamp_period_ns=1.000000000",
        "timestamp_frequency_hz=1000000000.000",
        "percentile_policy=nearest_rank",
        "outlier_policy=none_all_samples_retained",
        f"pass_coverage={','.join(pass_names)}",
        "total.min_us=42",
        "total.median_us=87",
        "total.p95_us=129",
        "total.max_us=132",
    ]
    for pass_name in pass_names:
        values = [sample[2][pass_name] for sample in samples]
        lines.extend(
            [
                f"pass.{pass_name}.min_us={values[0]}",
                f"pass.{pass_name}.median_us={values[15]}",
                f"pass.{pass_name}.p95_us={values[29]}",
                f"pass.{pass_name}.max_us={values[30]}",
            ]
        )
    for index, generation, values in samples:
        lines.append(f"sample.{index:03}.frame_generation={generation}")
        lines.append(f"sample.{index:03}.total_us={sum(values.values())}")
        for pass_name in pass_names:
            lines.append(f"sample.{index:03}.pass.{pass_name}_us={values[pass_name]}")
    return "\n".join(lines) + "\n"


@contextmanager
def _controlled_temporary_directory():
    workspace_root = Path(__file__).resolve().parents[2]
    if str(workspace_root).lower().startswith("c:"):
        artifact_parent = Path("D:/ZirconEngineTestArtifacts/gpu_timing_validator")
    else:
        artifact_parent = (
            workspace_root
            / "docs/tests/runtime/shader/.gpu-timing-validator-test-artifacts"
        )
    artifact_parent.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(dir=artifact_parent) as temporary_directory:
        yield temporary_directory
