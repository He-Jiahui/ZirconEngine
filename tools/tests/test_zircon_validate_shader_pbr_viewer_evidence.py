import json
import re
import struct
import tempfile
import unittest
import zlib
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from unittest import mock

from tools.zircon_validate_shader_pbr_viewer_evidence import (
    _CURRENT_IBL_BAKE_ALGORITHM_VERSION,
    main,
    validate_ready_frame_evidence,
)


class ZirconValidateShaderPbrViewerEvidenceTests(unittest.TestCase):
    def test_validates_ready_frame_png_and_v2_provenance_sidecar(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                4,
                3,
                [
                    (12, 18, 24, 255),
                    (42, 74, 96, 255),
                    (121, 163, 186, 255),
                    (238, 205, 128, 255),
                ]
                * 3,
            )
            _write_sidecar(png_path, viewport="4x3")

            evidence = validate_ready_frame_evidence(
                png_path,
                expected_backend="Dx12",
                require_direct_present=True,
            )

            self.assertEqual((4, 3), evidence.viewport)
            self.assertEqual("Dx12", evidence.backend)
            self.assertEqual("environment_only_pbr_preview", evidence.render_profile)
            self.assertEqual(4, evidence.distinct_rgba_colors)
            self.assertGreaterEqual(evidence.non_black_pixel_count, 1)

    def test_v3_provenance_requires_startup_pipeline_ready_state(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )
            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v3",
                viewport="2x2",
            )

            with self.assertRaisesRegex(RuntimeError, "v3 provenance sidecar is missing"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v3",
                environment_only_base_prewarm_pipeline_ready="false",
                viewport="2x2",
            )
            evidence = validate_ready_frame_evidence(png_path)

            self.assertEqual((2, 2), evidence.viewport)

    def test_v7_provenance_requires_capture_time_base_pipeline_readiness_and_timing(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v7",
                environment_only_base_prewarm_pipeline_ready="false",
                environment_only_base_pipeline_ready_at_capture="false",
                viewport="2x2",
            )
            with self.assertRaisesRegex(RuntimeError, "capture-time Base pipeline readiness"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v7",
                environment_only_base_prewarm_pipeline_ready="false",
                environment_only_base_pipeline_ready_at_capture="true",
                viewport="2x2",
            )
            evidence = validate_ready_frame_evidence(png_path)

            self.assertEqual((2, 2), evidence.viewport)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v7",
                environment_only_base_prewarm_pipeline_ready="false",
                environment_only_base_pipeline_ready_at_capture="true",
                scene_startup_total_ns="1",
                viewport="2x2",
            )
            with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v7",
                environment_only_base_prewarm_pipeline_ready="false",
                environment_only_base_pipeline_ready_at_capture="true",
                one_shot_base_pipeline_wait_elapsed_ns="-1",
                viewport="2x2",
            )
            with self.assertRaisesRegex(RuntimeError, "duration is malformed"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **{
                    **_v9_ibl_staging_fields(),
                    "ibl_staging_cubemap_build_ns": "0",
                },
            )
            with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                ibl_staging_elapsed_ns="1",
                **_v9_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v7",
                environment_only_base_prewarm_pipeline_ready="false",
                environment_only_base_pipeline_ready_at_capture="true",
                viewer_scene_load_elapsed_ns="1",
                viewport="2x2",
            )
            with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                validate_ready_frame_evidence(png_path)

    def test_cli_requires_v12_schema_and_shader_pipeline_metrics_unless_legacy_read_is_explicit(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )
            for legacy_schema_version in range(2, 12):
                legacy_ibl_fields = _legacy_ibl_staging_fields(legacy_schema_version)
                _write_sidecar(
                    png_path,
                    schema=(
                        "zircon_shader_pbr_viewer_ready_frame_evidence_"
                        f"v{legacy_schema_version}"
                    ),
                    environment_only_base_prewarm_pipeline_ready="true",
                    environment_only_base_pipeline_ready_at_capture="true",
                    **legacy_ibl_fields,
                )

                stderr = StringIO()
                with (
                    mock.patch("sys.argv", ["validator", str(png_path)]),
                    redirect_stderr(stderr),
                ):
                    self.assertEqual(1, main())
                self.assertIn(
                    "requires schema=zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                    stderr.getvalue(),
                )

                with (
                    mock.patch(
                        "sys.argv",
                        ["validator", "--allow-legacy-schema", str(png_path)],
                    ),
                    redirect_stdout(StringIO()),
                ):
                    self.assertEqual(0, main())

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v11",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v10_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(
                RuntimeError,
                "v11 provenance sidecar is missing required fields: "
                "ibl_staging_irradiance_cube_source_sample_visits",
            ):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v11",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v11_ibl_staging_fields(),
            )
            stderr = StringIO()
            with (
                mock.patch("sys.argv", ["validator", str(png_path)]),
                redirect_stderr(stderr),
            ):
                self.assertEqual(1, main())
            self.assertIn(
                "requires schema=zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                stderr.getvalue(),
            )
            with (
                mock.patch(
                    "sys.argv", ["validator", "--allow-legacy-schema", str(png_path)]
                ),
                redirect_stdout(StringIO()),
            ):
                self.assertEqual(0, main())

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v11_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(
                RuntimeError,
                "v12 provenance sidecar is missing required fields: "
                "registered_pipeline_variant_count",
            ):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v11_ibl_staging_fields(),
                **_v12_shader_pipeline_metrics(),
            )
            stdout = StringIO()
            with (
                mock.patch("sys.argv", ["validator", str(png_path)]),
                redirect_stdout(stdout),
            ):
                self.assertEqual(0, main())
            summary = json.loads(stdout.getvalue())
            self.assertEqual(
                8,
                summary["shader_pipeline_metrics"]["render_pipeline_creation_count"],
            )
            self.assertEqual(
                1,
                summary["shader_pipeline_metrics"][
                    "async_base_pipeline_queue_wait_count"
                ],
            )
            self.assertEqual(
                1_350_000_000,
                summary["startup_timing_ns"]["viewer_ready_elapsed_ns"],
            )

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v11",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                ibl_staging_status="Written",
                **{
                    **_v11_ibl_staging_fields(),
                    "ibl_staging_parallel_executor_work_items": "42",
                    "ibl_staging_irradiance_cube_source_sample_visits": "37748736",
                },
            )
            evidence = validate_ready_frame_evidence(png_path)
            self.assertEqual((2, 2), evidence.viewport)

            for value, expected_error in (
                ("37748736", "cache reuse must not report IEM candidate iterations"),
                ("-1", "non-negative count is malformed"),
                ("not-a-number", "non-negative count is malformed"),
                (str(1 << 64), "count exceeds u64"),
            ):
                with self.subTest(sample_visits=value), tempfile.TemporaryDirectory() as sample_dir:
                    sample_png_path = Path(sample_dir) / "pbr-ready.png"
                    _write_rgba_png(
                        sample_png_path,
                        2,
                        2,
                        [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
                    )
                    _write_sidecar(
                        sample_png_path,
                        schema="zircon_shader_pbr_viewer_ready_frame_evidence_v11",
                        environment_only_base_prewarm_pipeline_ready="true",
                        environment_only_base_pipeline_ready_at_capture="true",
                        **{
                            **_v11_ibl_staging_fields(),
                            "ibl_staging_irradiance_cube_source_sample_visits": value,
                        },
                    )
                    with self.assertRaisesRegex(RuntimeError, expected_error):
                        validate_ready_frame_evidence(sample_png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                ibl_bake_algorithm_version="202607310004",
                **_v11_ibl_staging_fields(),
                **_v12_shader_pipeline_metrics(),
            )
            stderr = StringIO()
            with (
                mock.patch("sys.argv", ["validator", str(png_path)]),
                redirect_stderr(stderr),
            ):
                self.assertEqual(1, main())
            self.assertIn(
                "requires IBL bake algorithm version=202608090006", stderr.getvalue()
            )

            with (
                mock.patch(
                    "sys.argv", ["validator", "--allow-legacy-schema", str(png_path)]
                ),
                redirect_stdout(StringIO()),
            ):
                self.assertEqual(0, main())

    def test_v9_provenance_requires_ready_timing_and_ibl_staging_phases(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                viewer_ready_elapsed_ns="1340000000",
                **_v9_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v9_ibl_staging_fields(),
            )
            evidence = validate_ready_frame_evidence(png_path)

            self.assertEqual((2, 2), evidence.viewport)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
            )
            with self.assertRaisesRegex(RuntimeError, "v9 provenance sidecar is missing"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v9",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **{**_v9_ibl_staging_fields(), "ibl_staging_pmrem_build_ns": "-1"},
            )
            with self.assertRaisesRegex(RuntimeError, "duration is malformed"):
                validate_ready_frame_evidence(png_path)

    def test_v12_pipeline_metrics_reject_malformed_and_inconsistent_snapshots(self):
        cases = (
            (
                {"render_pipeline_creation_count": str(1 << 64)},
                "count exceeds u64",
            ),
            (
                {"texture_presence_equivalent_pipeline_variant_count": "14"},
                "texture-presence normalization is inconsistent",
            ),
            (
                {"cached_render_pipeline_count": "9"},
                "cached render pipelines exceed creation events",
            ),
            (
                {"cached_render_pipeline_count": "0"},
                "requires resident Base pipeline GPU objects",
            ),
            (
                {"cached_shader_module_count": "3"},
                "cached shader modules exceed creation events",
            ),
            (
                {"cached_shader_module_count": "0"},
                "requires resident Base pipeline GPU objects",
            ),
            (
                {
                    "async_base_pipeline_queue_wait_count": "0",
                    "async_base_pipeline_queue_wait_microseconds": "1",
                },
                "queue wait time has no admitted async job",
            ),
        )
        for overrides, expected_error in cases:
            with self.subTest(overrides=overrides), tempfile.TemporaryDirectory() as temp_dir:
                png_path = Path(temp_dir) / "pbr-ready.png"
                _write_rgba_png(
                    png_path,
                    2,
                    2,
                    [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
                )
                _write_sidecar(
                    png_path,
                    schema="zircon_shader_pbr_viewer_ready_frame_evidence_v12",
                    environment_only_base_prewarm_pipeline_ready="true",
                    environment_only_base_pipeline_ready_at_capture="true",
                    **_v11_ibl_staging_fields(),
                    **{**_v12_shader_pipeline_metrics(), **overrides},
                )

                with self.assertRaisesRegex(RuntimeError, expected_error):
                    validate_ready_frame_evidence(png_path)

    def test_v10_provenance_requires_persisted_output_sizes_and_executor_work(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v10_ibl_staging_fields(),
            )
            evidence = validate_ready_frame_evidence(png_path)
            self.assertEqual((2, 2), evidence.viewport)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                ibl_staging_status="Written",
                **{
                    **_v10_ibl_staging_fields(),
                    "ibl_staging_parallel_executor_work_items": "42",
                },
            )
            evidence = validate_ready_frame_evidence(png_path)
            self.assertEqual((2, 2), evidence.viewport)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **_v9_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(RuntimeError, "v10 provenance sidecar is missing"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **{**_v10_ibl_staging_fields(), "ibl_staging_source_zcube_bytes": "0"},
            )
            with self.assertRaisesRegex(RuntimeError, "non-empty staged IBL outputs"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                ibl_staging_status="Written",
                **_v10_ibl_staging_fields(),
            )
            with self.assertRaisesRegex(RuntimeError, "written HDRI must report IBL executor work"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                schema="zircon_shader_pbr_viewer_ready_frame_evidence_v10",
                environment_only_base_prewarm_pipeline_ready="true",
                environment_only_base_pipeline_ready_at_capture="true",
                **{
                    **_v10_ibl_staging_fields(),
                    "ibl_staging_parallel_executor_work_items": "1",
                },
            )
            with self.assertRaisesRegex(RuntimeError, "cache reuse must not submit"):
                validate_ready_frame_evidence(png_path)

    def test_rejects_viewport_mismatch_and_wrong_cache_scope(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            _write_sidecar(png_path, viewport="3x2")
            with self.assertRaisesRegex(RuntimeError, "viewport does not match PNG dimensions"):
                validate_ready_frame_evidence(png_path)

            _write_sidecar(
                png_path,
                environment_only_base_prewarm_cache_scope="driver_cache",
            )
            with self.assertRaisesRegex(RuntimeError, "cache scope"):
                validate_ready_frame_evidence(png_path)

    def test_rejects_visually_blank_ready_frame(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(png_path, 4, 4, [(0, 0, 0, 255)] * 16)
            _write_sidecar(png_path, viewport="4x4")

            with self.assertRaisesRegex(RuntimeError, "visually insufficient"):
                validate_ready_frame_evidence(png_path)

    def test_rejects_fully_transparent_ready_frame_with_rgb_data(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 0), (64, 96, 128, 0)] * 2,
            )
            _write_sidecar(png_path, viewport="2x2")

            with self.assertRaisesRegex(RuntimeError, "visually insufficient"):
                validate_ready_frame_evidence(png_path)

    def test_rejects_png_larger_than_encoded_evidence_budget(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            with mock.patch(
                "tools.zircon_validate_shader_pbr_viewer_evidence._MAX_ENCODED_PNG_BYTES",
                1,
            ):
                with self.assertRaisesRegex(RuntimeError, "encoded evidence budget"):
                    validate_ready_frame_evidence(png_path)

    def test_rejects_png_with_too_many_chunks(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )

            with mock.patch(
                "tools.zircon_validate_shader_pbr_viewer_evidence._MAX_PNG_CHUNKS",
                2,
            ):
                with self.assertRaisesRegex(RuntimeError, "chunk budget"):
                    validate_ready_frame_evidence(png_path)

    def test_validates_zlib_stream_split_across_idat_chunks(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
                idat_split_after=1,
            )
            _write_sidecar(png_path, viewport="2x2")

            evidence = validate_ready_frame_evidence(png_path)

            self.assertEqual(2, evidence.distinct_rgba_colors)

    def test_rejects_inconsistent_active_cubemap_mip_layout(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            png_path = Path(temp_dir) / "pbr-ready.png"
            _write_rgba_png(
                png_path,
                2,
                2,
                [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
            )
            _write_sidecar(
                png_path,
                active_source_cubemap_mip_count="9",
                viewport="2x2",
            )

            with self.assertRaisesRegex(RuntimeError, "cubemap mip layout"):
                validate_ready_frame_evidence(png_path)

    def test_rejects_inconsistent_duration_hierarchies(self):
        inconsistent_durations = (
            (
                "environment_only_base_prewarm_elapsed_ns",
                "12000000",
                "Base-prewarm",
            ),
            ("ibl_total_elapsed_ns", "7000000", "IBL"),
            ("ready_frame_render_elapsed_ns", "15000000", "Ready-frame"),
        )
        for field, value, phase in inconsistent_durations:
            with self.subTest(phase=phase), tempfile.TemporaryDirectory() as temp_dir:
                png_path = Path(temp_dir) / "pbr-ready.png"
                _write_rgba_png(
                    png_path,
                    2,
                    2,
                    [(32, 64, 96, 255), (64, 96, 128, 255)] * 2,
                )
                _write_sidecar(png_path, viewport="2x2", **{field: value})

                with self.assertRaisesRegex(RuntimeError, "duration hierarchy"):
                    validate_ready_frame_evidence(png_path)

    def test_cli_bake_algorithm_version_matches_runtime_constant(self):
        artifact_source_path = (
            Path(__file__).resolve().parents[2]
            / "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs"
        )
        artifact_source = artifact_source_path.read_text(encoding="utf-8")
        self.assertIn(
            "pub const IBL_BAKE_ALGORITHM_VERSION: u64 = CANONICAL_IBL_BAKE_ALGORITHM_VERSION;",
            artifact_source,
        )
        source_path = (
            Path(__file__).resolve().parents[2]
            / "zircon_runtime/src/core/framework/render/environment/ibl_bake_recipe.rs"
        )
        source = source_path.read_text(encoding="utf-8")
        match = re.search(
            r"CANONICAL_IBL_BAKE_ALGORITHM_VERSION: u64 = ([0-9_]+);", source
        )

        self.assertIsNotNone(match)
        self.assertEqual(
            match.group(1).replace("_", ""), _CURRENT_IBL_BAKE_ALGORITHM_VERSION
        )


def _write_sidecar(png_path: Path, **overrides: str) -> None:
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
    metadata.update(overrides)
    png_path.with_name(f"{png_path.name}.txt").write_text(
        "\n".join(f"{key}={value}" for key, value in metadata.items()) + "\n",
        encoding="utf-8",
    )


def _v9_ibl_staging_fields() -> dict[str, str]:
    return {
        "ibl_staging_source_decode_ns": "0",
        "ibl_staging_cubemap_build_ns": "4000000",
        "ibl_staging_equirect_projection_ns": "3000000",
        "ibl_staging_source_mip_build_ns": "1000000",
        "ibl_staging_pmrem_build_ns": "0",
        "ibl_staging_sh9_build_ns": "0",
        "ibl_staging_irradiance_cube_build_ns": "0",
        "ibl_staging_bundle_write_ns": "0",
    }


def _v10_ibl_staging_fields() -> dict[str, str]:
    return {
        **_v9_ibl_staging_fields(),
        "ibl_staging_source_zcube_bytes": "1024",
        "ibl_staging_asset_derived_bytes": "2048",
        "ibl_staging_parallel_executor_work_items": "0",
    }


def _v11_ibl_staging_fields() -> dict[str, str]:
    return {
        **_v10_ibl_staging_fields(),
        "ibl_staging_irradiance_cube_source_sample_visits": "0",
    }


def _v12_shader_pipeline_metrics() -> dict[str, str]:
    return {
        "registered_pipeline_variant_count": "16",
        "registered_shader_variant_count": "1",
        "texture_presence_normalized_pipeline_variant_count": "1",
        "texture_presence_equivalent_pipeline_variant_count": "15",
        "cached_render_pipeline_count": "8",
        "cached_shader_module_count": "2",
        "render_pipeline_creation_count": "8",
        "shader_module_creation_count": "2",
        "render_pipeline_creation_cpu_microseconds": "42000",
        "shader_module_creation_cpu_microseconds": "17000",
        "async_base_pipeline_queue_wait_count": "1",
        "async_base_pipeline_queue_wait_microseconds": "88",
    }


def _legacy_ibl_staging_fields(schema_version: int) -> dict[str, str]:
    if schema_version == 11:
        return _v11_ibl_staging_fields()
    if schema_version == 10:
        return _v10_ibl_staging_fields()
    if schema_version == 9:
        return _v9_ibl_staging_fields()
    return {}


def _write_rgba_png(
    path: Path,
    width: int,
    height: int,
    pixels: list[tuple[int, int, int, int]],
    *,
    idat_split_after: int | None = None,
) -> None:
    if len(pixels) != width * height:
        raise ValueError("fixture pixels must match PNG dimensions")
    rows = []
    for row_index in range(height):
        row = pixels[row_index * width : (row_index + 1) * width]
        rows.append(b"\0" + b"".join(bytes(pixel) for pixel in row))
    compressed = zlib.compress(b"".join(rows))
    if idat_split_after is None:
        idat_chunks = (compressed,)
    elif 0 < idat_split_after < len(compressed):
        idat_chunks = (compressed[:idat_split_after], compressed[idat_split_after:])
    else:
        raise ValueError("IDAT split must leave bytes in both chunks")
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + b"".join(_png_chunk(b"IDAT", chunk) for chunk in idat_chunks)
        + _png_chunk(b"IEND", b"")
    )


def _png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )
