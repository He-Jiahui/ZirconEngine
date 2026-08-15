import hashlib
import unittest
from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.zircon_validate_shader_pbr_gpu_timing_evidence import (
    GPU_TIMING_EVIDENCE_SCHEMA,
    OPTIONAL_DIRECT_GPU_PASSES,
    REQUIRED_HDRI_DIRECT_GPU_PASSES,
    validate_gpu_timing_evidence,
)


class ZirconValidateShaderPbrGpuTimingEvidenceTests(unittest.TestCase):
    def test_accepts_measured_hdri_direct_passes_and_zero_upload_time(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n",
            )

            evidence = validate_gpu_timing_evidence(report_path, screenshot_path)

            self.assertEqual(17, evidence.frame_generation)
            self.assertEqual(
                {
                    "direct_gpu_scene_upload": 0,
                    "direct_scene_content": 420,
                    "direct_output_transfer": 31,
                    "direct_overlays": 2,
                },
                evidence.pass_gpu_time_us,
            )

    def test_rejects_measured_hdri_report_missing_a_required_direct_pass(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_overlays=2\n",
            )

            with self.assertRaisesRegex(RuntimeError, "direct_output_transfer"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_non_measured_or_duplicate_timing_fields(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=timed_out\n"
                f"screenshot={screenshot_path.name}\n"
                "max_resolve_frames=8\n",
            )
            with self.assertRaisesRegex(RuntimeError, "status=measured"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                "frame_generation=17\n"
                "frame_generation=18\n",
            )
            with self.assertRaisesRegex(RuntimeError, "repeats a field"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_malformed_or_overflow_gpu_timing_values(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=0\n",
            )
            with self.assertRaisesRegex(RuntimeError, "positive u64"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=18446744073709551616\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n",
            )
            with self.assertRaisesRegex(RuntimeError, "exceeds u64"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_a_missing_or_mismatched_screenshot_pair(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                "screenshot=other-ready.png\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n",
            )

            with self.assertRaisesRegex(RuntimeError, "screenshot is unavailable"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

            screenshot_path.write_bytes(b"fixture")
            with self.assertRaisesRegex(RuntimeError, "does not match"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_same_name_png_with_a_different_content_digest(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            first_screenshot = Path(temporary_directory) / "first" / "ready.png"
            second_screenshot = Path(temporary_directory) / "second" / "ready.png"
            first_screenshot.parent.mkdir()
            second_screenshot.parent.mkdir()
            first_screenshot.write_bytes(b"first frame")
            second_screenshot.write_bytes(b"second frame")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                "screenshot=ready.png\n"
                f"screenshot_sha256={_screenshot_sha256(first_screenshot)}\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n",
            )

            with self.assertRaisesRegex(RuntimeError, "SHA-256 does not match"):
                validate_gpu_timing_evidence(report_path, second_screenshot)

    def test_rejects_terminal_status_fields_mixed_into_a_measured_report(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=17\n"
                "renderer_status=Unavailable\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n",
            )

            with self.assertRaisesRegex(RuntimeError, "unexpected measured fields"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_rejects_an_unknown_pass_name(self):
        with _controlled_temporary_directory() as temporary_directory:
            report_path = Path(temporary_directory) / "gpu-timing.txt"
            screenshot_path = Path(temporary_directory) / "pbr-ready.png"
            screenshot_path.write_bytes(b"fixture")
            _write_report(
                report_path,
                f"schema={GPU_TIMING_EVIDENCE_SCHEMA}\n"
                "status=measured\n"
                f"screenshot={screenshot_path.name}\n"
                f"screenshot_sha256={_screenshot_sha256(screenshot_path)}\n"
                "frame_generation=17\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=420\n"
                "pass.direct_output_transfer=31\n"
                "pass.direct_overlays=2\n"
                "pass.fabricated_total=9001\n",
            )

            with self.assertRaisesRegex(RuntimeError, "unknown direct pass"):
                validate_gpu_timing_evidence(report_path, screenshot_path)

    def test_contract_keeps_all_required_hdri_direct_passes_explicit(self):
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


def _write_report(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")


def _screenshot_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
