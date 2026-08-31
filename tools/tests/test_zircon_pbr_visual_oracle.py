import hashlib
import json
import struct
import tempfile
import unittest
import zlib
from pathlib import Path

from tools.zircon_pbr_visual_oracle import decode_rgba_png, validate_display_visual_oracle
from tools.zircon_validate_shader_pbr_viewer_evidence import (
    ready_frame_evidence_summary,
    validate_ready_frame_evidence,
)


class ZirconPbrVisualOracleTests(unittest.TestCase):
    def test_accepts_provenance_bound_semantic_regions(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            reference_png = root / "reference.png"
            candidate_png = root / "candidate.png"
            pixels = [
                (16, 32, 64, 255),
                (64, 96, 128, 255),
                (128, 96, 64, 255),
                (240, 224, 192, 255),
            ]
            _write_rgba_png(reference_png, 2, 2, pixels)
            _write_rgba_png(candidate_png, 2, 2, pixels)
            decoded_candidate = decode_rgba_png(candidate_png)
            oracle_path = root / "display-oracle.json"
            _write_display_visual_oracle(
                oracle_path,
                reference_png=reference_png.name,
                expected_metadata={
                    "host_mode": "offscreen-diagnostic",
                    "scene_id": "single_pbr_mirror_sphere",
                    "backend": "Dx12",
                },
                semantic_regions=[
                    {
                        "id": "upper_environment",
                        "x": 0,
                        "y": 0,
                        "width": 2,
                        "height": 1,
                        "max_mean_abs_error": 0.0,
                    },
                    {
                        "id": "sphere_highlight",
                        "x": 1,
                        "y": 1,
                        "width": 1,
                        "height": 1,
                        "max_mean_abs_error": 0.0,
                    },
                ],
            )

            result = validate_display_visual_oracle(
                candidate_png,
                metadata={
                    "host_mode": "offscreen-diagnostic",
                    "scene_id": "single_pbr_mirror_sphere",
                    "backend": "Dx12",
                },
                oracle_path=oracle_path,
                _candidate_image=decoded_candidate,
            )

            self.assertEqual(4, result.compared_pixel_count)
            self.assertEqual(0.0, result.mean_abs_error)
            self.assertEqual(0.0, result.p99_abs_error)
            self.assertEqual(0.0, result.exceeding_pixel_fraction)
            self.assertEqual(
                {"upper_environment": 0.0, "sphere_highlight": 0.0},
                result.semantic_region_mean_abs_errors,
            )

    def test_rejects_unbound_provenance_and_large_error(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            reference_png = root / "reference.png"
            candidate_png = root / "candidate.png"
            _write_rgba_png(reference_png, 2, 2, [(128, 96, 64, 255)] * 4)
            _write_rgba_png(candidate_png, 2, 2, [(0, 0, 0, 255)] * 4)
            oracle_path = root / "display-oracle.json"
            _write_display_visual_oracle(
                oracle_path,
                reference_png=reference_png.name,
                expected_metadata={
                    "host_mode": "offscreen-diagnostic",
                    "scene_id": "single_pbr_mirror_sphere",
                },
            )

            with self.assertRaisesRegex(RuntimeError, "provenance does not match"):
                validate_display_visual_oracle(
                    candidate_png,
                    metadata={
                        "host_mode": "native-present",
                        "scene_id": "single_pbr_mirror_sphere",
                    },
                    oracle_path=oracle_path,
                )
            with self.assertRaisesRegex(RuntimeError, "mean absolute error"):
                validate_display_visual_oracle(
                    candidate_png,
                    metadata={
                        "host_mode": "offscreen-diagnostic",
                        "scene_id": "single_pbr_mirror_sphere",
                    },
                    oracle_path=oracle_path,
                )

    def test_current_ior_oracle_requires_material_pso_provenance(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            reference_png = root / "reference.png"
            candidate_png = root / "candidate.png"
            _write_rgba_png(reference_png, 2, 2, [(128, 96, 64, 255)] * 4)
            _write_rgba_png(candidate_png, 2, 2, [(128, 96, 64, 255)] * 4)
            oracle_path = root / "display-oracle.json"
            current_metadata = {
                "schema": "zircon_shader_pbr_viewer_ready_frame_evidence_v17",
                "material_fixture": "dielectric-ior",
                "required_material_base_pipeline_kind": "generic-forward-pbr-ior",
                "required_material_base_pipeline_ready_at_capture": "true",
                "environment_only_base_prewarm_requested": "false",
            }
            _write_display_visual_oracle(
                oracle_path,
                reference_png=reference_png.name,
                expected_metadata={"schema": current_metadata["schema"]},
            )

            with self.assertRaisesRegex(RuntimeError, "must bind current material provenance"):
                validate_display_visual_oracle(
                    candidate_png,
                    metadata=current_metadata,
                    oracle_path=oracle_path,
                )

            _write_display_visual_oracle(
                oracle_path,
                reference_png=reference_png.name,
                expected_metadata=current_metadata,
            )
            result = validate_display_visual_oracle(
                candidate_png,
                metadata=current_metadata,
                oracle_path=oracle_path,
            )

            self.assertEqual(0.0, result.mean_abs_error)

    def test_ready_frame_validator_reports_display_visual_oracle_metrics(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            reference_png = root / "reference.png"
            candidate_png = root / "pbr-ready.png"
            pixels = [(32, 64, 96, 255), (64, 96, 128, 255)] * 2
            _write_rgba_png(reference_png, 2, 2, pixels)
            _write_rgba_png(candidate_png, 2, 2, pixels)
            _write_ready_sidecar(candidate_png)
            oracle_path = root / "display-oracle.json"
            _write_display_visual_oracle(
                oracle_path,
                reference_png=reference_png.name,
                expected_metadata={"backend": "Dx12"},
            )

            evidence = validate_ready_frame_evidence(
                candidate_png,
                visual_oracle_path=oracle_path,
            )
            summary = ready_frame_evidence_summary(evidence)

            self.assertIsNotNone(evidence.display_visual_oracle)
            self.assertEqual(0.0, summary["display_visual_oracle"]["mean_abs_error"])
            self.assertEqual(4, summary["display_visual_oracle"]["compared_pixel_count"])


def _write_display_visual_oracle(
    path: Path,
    *,
    reference_png: str,
    expected_metadata: dict[str, str],
    semantic_regions: list[dict[str, object]] | None = None,
) -> None:
    reference_path = path.parent / reference_png
    path.write_text(
        json.dumps(
            {
                "schema": "zircon_pbr_display_visual_oracle_v1",
                "reference_png": reference_png,
                "reference_png_sha256": hashlib.sha256(reference_path.read_bytes()).hexdigest(),
                "expected_metadata": expected_metadata,
                "comparison": {
                    "max_mean_abs_error": 2.0,
                    "max_p99_abs_error": 4,
                    "exceeding_abs_error": 8,
                    "max_exceeding_pixel_fraction": 0.05,
                },
                "semantic_regions": semantic_regions or [],
            },
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )


def _write_ready_sidecar(png_path: Path) -> None:
    metadata = {
        "schema": "zircon_shader_pbr_viewer_ready_frame_evidence_v2",
        "screenshot": png_path.name,
        "screenshot_presentation": "cpu_readback",
        "interactive_direct_present_enabled": "true",
        "backend": "Dx12",
        "hdri_path": "assets/lakes.hdr",
        "requested_source_face_size": "automatic",
        "requested_pmrem_face_size": "256",
        "active_source_cubemap_face_size": "512",
        "active_source_cubemap_mip_count": "10",
        "active_pmrem_face_size": "256",
        "active_pmrem_mip_count": "9",
        "render_profile": "environment_only_pbr_preview",
        "environment_only_base_prewarm_cache_hit": "false",
        "environment_only_base_prewarm_cache_scope": "process_local_mesh_pipeline_cache",
        "environment_only_base_prewarm_shader_source_resolution_ns": "2000000",
        "environment_only_base_prewarm_pipeline_creation_ns": "11000000",
        "environment_only_base_prewarm_elapsed_ns": "13000000",
        "viewport": "2x2",
        "camera_yaw_degrees": "12.500",
        "camera_pitch_degrees": "-7.000",
        "ibl_bake_algorithm_version": "202608090006",
        "ibl_staging_status": "Reused",
        "ibl_staging_elapsed_ns": "8000000",
        "ibl_total_elapsed_ns": "12000000",
        "scene_startup_hdri_decode_ns": "21000000",
        "scene_startup_project_assets_ns": "34000000",
        "scene_startup_runtime_bootstrap_ns": "55000000",
        "scene_startup_project_open_ns": "89000000",
        "scene_startup_world_load_ns": "144000000",
        "scene_startup_renderer_initialization_ns": "233000000",
        "scene_startup_renderer_backend_initialization_ns": "34000000",
        "scene_startup_renderer_deferred_initialization_ns": "89000000",
        "scene_startup_renderer_deferred_standard_pipeline_ns": "55000000",
        "scene_startup_resource_streamer_initialization_ns": "34000000",
        "scene_startup_ibl_restore_ns": "610000000",
        "scene_startup_total_ns": "1200000000",
        "one_shot_base_pipeline_wait_elapsed_ns": "75000000",
        "viewer_scene_load_elapsed_ns": "1250000000",
        "viewer_ready_elapsed_ns": "1350000000",
        "ready_frame_render_elapsed_ns": "16000000",
        "ready_frame_extract_ns": "2000000",
        "ready_frame_renderer_call_ns": "11000000",
        "ready_frame_readback_and_completion_ns": "3000000",
    }
    png_path.with_name(f"{png_path.name}.txt").write_text(
        "\n".join(f"{key}={value}" for key, value in metadata.items()) + "\n",
        encoding="utf-8",
    )


def _write_rgba_png(
    path: Path,
    width: int,
    height: int,
    pixels: list[tuple[int, int, int, int]],
) -> None:
    if len(pixels) != width * height:
        raise ValueError("fixture pixels must match PNG dimensions")
    rows = []
    for row_index in range(height):
        row = pixels[row_index * width : (row_index + 1) * width]
        rows.append(b"\0" + b"".join(bytes(pixel) for pixel in row))
    compressed = zlib.compress(b"".join(rows))
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + _png_chunk(b"IDAT", compressed)
        + _png_chunk(b"IEND", b"")
    )


def _png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )
