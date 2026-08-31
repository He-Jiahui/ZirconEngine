import json
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.zircon_editor_ui_visual_oracle import (
    _RgbImage,
    _validate_antialias_population,
    VisualOracleError,
    validate_capture_manifest,
)


def rounded_fixture_pixels(
    *,
    antialiased: bool,
    icon_antialiased: bool,
    text_antialiased: bool,
    rounded_radius: float = 8.0,
    sparse_corner_coverage: bool = False,
) -> bytes:
    width = 64
    height = 64
    background = (32, 34, 38)
    foreground = (104, 128, 180)
    rounded_samples = 8 if antialiased else 1
    icon_samples = 8 if icon_antialiased else 1
    rectangle = (8.0, 12.0, 56.0, 44.0)
    radius = rounded_radius
    icon_center = (32.0, 28.0)
    icon_radius = 6.75
    icon_foreground = (225, 232, 244)
    pixels = bytearray(width * height * 3)
    destination = 0

    for y in range(height):
        for x in range(width):
            inside_samples = 0
            for sample_y in range(rounded_samples):
                point_y = y + (sample_y + 0.5) / rounded_samples
                for sample_x in range(rounded_samples):
                    point_x = x + (sample_x + 0.5) / rounded_samples
                    center_x = min(
                        max(point_x, rectangle[0] + radius),
                        rectangle[2] - radius,
                    )
                    center_y = min(
                        max(point_y, rectangle[1] + radius),
                        rectangle[3] - radius,
                    )
                    distance_squared = (point_x - center_x) ** 2 + (
                        point_y - center_y
                    ) ** 2
                    inside_samples += int(distance_squared <= radius**2)
            coverage = inside_samples / (rounded_samples * rounded_samples)
            icon_inside_samples = 0
            for sample_y in range(icon_samples):
                point_y = y + (sample_y + 0.5) / icon_samples
                for sample_x in range(icon_samples):
                    point_x = x + (sample_x + 0.5) / icon_samples
                    icon_inside_samples += int(
                        (point_x - icon_center[0]) ** 2
                        + (point_y - icon_center[1]) ** 2
                        <= icon_radius**2
                    )
            icon_coverage = icon_inside_samples / (icon_samples * icon_samples)
            for channel in range(3):
                control_channel = (
                    background[channel]
                    + (foreground[channel] - background[channel]) * coverage
                )
                pixels[destination] = round(
                    control_channel
                    + (icon_foreground[channel] - control_channel) * icon_coverage
                )
                destination += 1
    if sparse_corner_coverage:
        sparse_color = (78, 95, 134)
        for x, y in ((9, 12), (54, 12)):
            offset = (y * width + x) * 3
            pixels[offset : offset + 3] = sparse_color

    text_samples = 8 if text_antialiased else 1
    text_foreground = (225, 232, 244)
    for y in range(47, 60):
        for x in range(10, 40):
            inside_samples = 0
            for sample_y in range(text_samples):
                point_y = y + (sample_y + 0.5) / text_samples
                for sample_x in range(text_samples):
                    point_x = x + (sample_x + 0.5) / text_samples
                    top_bar = 12.0 <= point_x <= 28.0 and 48.5 <= point_y <= 50.0
                    bottom_bar = (
                        12.0 <= point_x <= 28.0 and 56.0 <= point_y <= 57.5
                    )
                    diagonal_center = 27.0 - (point_y - 49.25) * 2.0
                    diagonal = (
                        49.25 <= point_y <= 56.75
                        and abs(point_x - diagonal_center) <= 0.8
                    )
                    inside_samples += int(top_bar or bottom_bar or diagonal)
            coverage = inside_samples / (text_samples * text_samples)
            if coverage <= 0.0:
                continue
            offset = (y * width + x) * 3
            for channel in range(3):
                pixels[offset + channel] = round(
                    background[channel]
                    + (text_foreground[channel] - background[channel]) * coverage
                )
    return bytes(pixels)


def write_capture_fixture(
    root: Path,
    *,
    antialiased: bool,
    icon_antialiased: bool | None = None,
    text_antialiased: bool = True,
    backend: str = "gpu",
    fractional_frame: bool = False,
    rounded_radius: float = 8.0,
    sparse_corner_coverage: bool = False,
) -> Path:
    extent = (64, 64)
    revision = "b" * 40
    source_relative_path = "zircon_editor/assets/ui/editor/fixture.zui"
    source_path = root / source_relative_path
    source_path.parent.mkdir(parents=True, exist_ok=True)
    source_bytes = b'[meta]\nfixture = "ui"\n'
    source_path.write_bytes(source_bytes)
    source_file_sha256 = hashlib.sha256(source_bytes).hexdigest()
    source_canonical = (
        f"revision={revision}\n"
        f"{source_relative_path}\0{source_file_sha256}\0{len(source_bytes)}\n"
    ).encode("utf-8")
    source_sha256 = hashlib.sha256(source_canonical).hexdigest()
    editor = root / "zircon_editor.exe"
    runtime = root / "zircon_runtime.dll"
    editor.write_bytes(b"fixture editor binary")
    runtime.write_bytes(b"fixture runtime binary")
    bundle_asset = root / "assets/ui/editor/fixture.zui"
    bundle_asset.parent.mkdir(parents=True, exist_ok=True)
    bundle_asset.write_bytes(source_bytes)
    bundle_asset_canonical = (
        f"ui/editor/fixture.zui\0{source_file_sha256}\0{len(source_bytes)}\n"
    ).encode("utf-8")
    bundle_asset_sha256 = hashlib.sha256(bundle_asset_canonical).hexdigest()
    editor_sha256 = hashlib.sha256(editor.read_bytes()).hexdigest()
    runtime_sha256 = hashlib.sha256(runtime.read_bytes()).hexdigest()
    screenshot = root / "editor-64x64.png"
    if icon_antialiased is None:
        icon_antialiased = antialiased
    _RgbImage(
        width=extent[0],
        height=extent[1],
        pixels=rounded_fixture_pixels(
            antialiased=antialiased,
            icon_antialiased=icon_antialiased,
            text_antialiased=text_antialiased,
            rounded_radius=rounded_radius,
            sparse_corner_coverage=sparse_corner_coverage,
        ),
    ).save_png(screenshot)

    profile = root / "ui_profile_geometry.json"
    profile.write_text(
        json.dumps(
            {
                "schema_version": 4,
                "presenter_backend": backend,
                "window_client_size": {"width": 64, "height": 64},
                "layout": {
                    "center_band": {
                        "x": 0.0,
                        "y": 0.0,
                        "width": 64.0,
                        "height": 56.0,
                    },
                    "document_region": {
                        "x": 8.0,
                        "y": 8.0,
                        "width": 48.0,
                        "height": 40.0,
                    },
                    "left_region": {
                        "x": 0.0,
                        "y": 0.0,
                        "width": 0.0,
                        "height": 0.0,
                    },
                    "right_region": {
                        "x": 0.0,
                        "y": 0.0,
                        "width": 0.0,
                        "height": 0.0,
                    },
                    "bottom_region": {
                        "x": 0.0,
                        "y": 0.0,
                        "width": 0.0,
                        "height": 0.0,
                    },
                    "status_bar": {
                        "x": 0.0,
                        "y": 56.0,
                        "width": 64.0,
                        "height": 8.0,
                    },
                },
                "template_controls": [
                    {
                        "id": "FixtureActionButton",
                        "kind": "template_control",
                        "surface": "document",
                        "frame": {
                            "x": 8.25 if fractional_frame else 8.0,
                            "y": 12.25 if fractional_frame else 12.0,
                            "width": 47.5 if fractional_frame else 48.0,
                            "height": 31.5 if fractional_frame else 32.0,
                        },
                    }
                ],
                "viewport_toolbar_controls": [],
                "activity_rail_buttons": [
                    {
                        "id": "activity.scene.button",
                        "kind": "activity_rail_button",
                        "surface": "host",
                        "frame": {
                            "x": 8.0,
                            "y": 12.0,
                            "width": 48.0,
                            "height": 32.0,
                        },
                    }
                ],
                "clickable_frames": [
                    {
                        "id": "activity.scene.button",
                        "kind": "activity_rail_button",
                        "surface": "host",
                        "frame": {
                            "x": 8.0,
                            "y": 12.0,
                            "width": 48.0,
                            "height": 32.0,
                        },
                    }
                ],
                "rounded_shapes": [
                    {
                        "command_index": 0,
                        "frame": {
                            "x": 8.0,
                            "y": 12.0,
                            "width": 48.0,
                            "height": 32.0,
                        },
                        "clip": None,
                        "corner_radius": rounded_radius,
                        "border_width": 0.0,
                    }
                ],
                "text_runs": [
                    {
                        "command_index": 1,
                        "frame": {
                            "x": 10.0,
                            "y": 47.0,
                            "width": 30.0,
                            "height": 13.0,
                        },
                        "clip": None,
                        "color": [225, 232, 244, 255],
                        "font_size": 10.0,
                        "line_height": 13.0,
                        "text_length": 1,
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    manifest = root / "capture-manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "schema_version": 2,
                "repository": {
                    "root": str(root),
                    "source_sha256": source_sha256,
                    "git": {
                        "revision": revision,
                        "dirty": True,
                        "dirty_entry_count": 1,
                        "dirty_tree_sha256": "c" * 64,
                    },
                    "critical_source_files": [
                        {
                            "relative_path": source_relative_path,
                            "sha256": source_file_sha256,
                            "byte_length": len(source_bytes),
                        }
                    ],
                },
                "binaries": {
                    "editor": {
                        "path": str(editor),
                        "expected_sha256": editor_sha256,
                        "actual_sha256": editor_sha256,
                    },
                    "runtime": {
                        "path": str(runtime),
                        "expected_sha256": runtime_sha256,
                        "actual_sha256": runtime_sha256,
                    },
                },
                "assets": {
                    "root": str(root / "assets"),
                    "bundle_asset_sha256": bundle_asset_sha256,
                    "bundle_asset_file_count": 1,
                },
                "captures": [
                    {
                        "presenter_backend": backend,
                        "window_dpi": 96,
                        "window_scale_factor": 1.0,
                        "profile_geometry_path": str(profile),
                        "profile_geometry_sha256": hashlib.sha256(
                            profile.read_bytes()
                        ).hexdigest(),
                        "profile_surface_width": 64,
                        "profile_surface_height": 64,
                        "screenshot": {
                            "path": str(screenshot),
                            "width": 64,
                            "height": 64,
                            "sha256": hashlib.sha256(
                                screenshot.read_bytes()
                            ).hexdigest(),
                        },
                    }
                ]
            }
        ),
        encoding="utf-8",
    )
    return manifest


class ZirconEditorUiVisualOracleTests(unittest.TestCase):
    def test_capture_profile_schema_v4_is_mandatory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            manifest_payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(
                manifest_payload["captures"][0]["profile_geometry_path"]
            )
            profile_payload = json.loads(profile.read_text(encoding="utf-8"))
            profile_payload["schema_version"] = 3
            profile.write_text(json.dumps(profile_payload), encoding="utf-8")
            manifest_payload["captures"][0]["profile_geometry_sha256"] = (
                hashlib.sha256(profile.read_bytes()).hexdigest()
            )
            manifest.write_text(json.dumps(manifest_payload), encoding="utf-8")

            with self.assertRaisesRegex(
                VisualOracleError, "profile geometry must use schema_version=4"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_antialias_population_rejects_one_smooth_token_among_three_candidates(self):
        with self.assertRaisesRegex(
            VisualOracleError, "rounded-control antialias coverage is too sparse"
        ):
            _validate_antialias_population(
                label="rounded-control",
                candidate_count=3,
                antialiased_count=1,
            )

    def test_antialias_population_accepts_at_least_half_of_candidates(self):
        _validate_antialias_population(
            label="vector-icon",
            candidate_count=3,
            antialiased_count=2,
        )

    def test_capture_manifest_rejects_sparse_rounded_control_antialias_population(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            manifest_payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(
                manifest_payload["captures"][0]["profile_geometry_path"]
            )
            profile_payload = json.loads(profile.read_text(encoding="utf-8"))
            sparse_control = dict(profile_payload["template_controls"][0])
            sparse_control["id"] = "FixtureSecondaryButton"
            profile_payload["template_controls"].append(sparse_control)
            profile.write_text(json.dumps(profile_payload), encoding="utf-8")
            manifest_payload["captures"][0]["profile_geometry_sha256"] = (
                hashlib.sha256(profile.read_bytes()).hexdigest()
            )
            manifest.write_text(json.dumps(manifest_payload), encoding="utf-8")

            corner_call_count = 0

            def corner_report(*_args, **_kwargs):
                nonlocal corner_call_count
                antialiased = corner_call_count < 4
                corner_call_count += 1
                return {
                    "analyzable": True,
                    "antialiased": antialiased,
                    "fractional_pixel_count": 8 if antialiased else 0,
                    "coverage_bin_count": 4 if antialiased else 0,
                    "fractional_row_count": 6 if antialiased else 0,
                    "fractional_column_count": 6 if antialiased else 0,
                }

            with patch(
                "tools.zircon_editor_ui_visual_oracle._analyze_corner",
                side_effect=corner_report,
            ):
                with self.assertRaisesRegex(
                    VisualOracleError,
                    "rounded-control antialias coverage is too sparse",
                ):
                    validate_capture_manifest(
                        manifest,
                        expected_extents={(64, 64)},
                        output_directory=root / "oracle",
                    )

            self.assertEqual(corner_call_count, 12)

    def test_capture_manifest_schema_v2_is_mandatory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            payload["schema_version"] = 1
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "schema_version=2"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_requires_source_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            del payload["repository"]
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "source provenance"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_binary_build_receipt_mismatch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            payload["binaries"]["editor"]["actual_sha256"] = "f" * 64
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "managed build receipt"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_binary_changed_after_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            (root / "zircon_editor.exe").write_bytes(b"changed editor binary")

            with self.assertRaisesRegex(VisualOracleError, "binary fingerprint"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_source_changed_after_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            (root / "zircon_editor/assets/ui/editor/fixture.zui").write_text(
                "changed = true\n", encoding="utf-8"
            )

            with self.assertRaisesRegex(VisualOracleError, "source fingerprint"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_screenshot_changed_after_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            screenshot = root / "editor-64x64.png"
            screenshot.write_bytes(screenshot.read_bytes() + b"tamper")

            with self.assertRaisesRegex(VisualOracleError, "screenshot fingerprint"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_profile_geometry_changed_after_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            profile = root / "ui_profile_geometry.json"
            payload = json.loads(profile.read_text(encoding="utf-8"))
            payload["tampered"] = True
            profile.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "profile geometry fingerprint"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_capture_manifest_rejects_bundle_asset_changed_after_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            (root / "assets/ui/editor/fixture.zui").write_text(
                "changed = true\n", encoding="utf-8"
            )

            with self.assertRaisesRegex(VisualOracleError, "bundle asset"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_antialiased_rounded_control_and_vector_icon_publish_fractional_coverage(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)

            report = validate_capture_manifest(
                manifest,
                expected_extents={(64, 64)},
                output_directory=root / "oracle",
            )

            self.assertTrue(report["passed"])
            capture = report["captures"][0]
            self.assertGreaterEqual(capture["analyzable_corner_count"], 2)
            self.assertGreaterEqual(capture["antialiased_corner_count"], 2)
            self.assertGreaterEqual(capture["expected_radius_curve_control_count"], 1)
            self.assertEqual(capture["rounded_control_antialiased_ratio"], 1.0)
            self.assertGreater(capture["fractional_pixel_count"], 0)
            self.assertGreaterEqual(capture["analyzable_vector_icon_count"], 1)
            self.assertGreaterEqual(capture["antialiased_vector_icon_count"], 1)
            self.assertEqual(capture["vector_icon_antialiased_ratio"], 1.0)
            self.assertGreater(capture["vector_icon_fractional_pixel_count"], 0)
            self.assertGreaterEqual(capture["analyzable_text_run_count"], 1)
            self.assertGreaterEqual(capture["antialiased_text_run_count"], 1)
            self.assertEqual(capture["text_run_antialiased_ratio"], 1.0)
            self.assertGreater(capture["text_fractional_pixel_count"], 0)
            self.assertTrue(list((root / "oracle" / "crops").glob("*.png")))

    def test_binary_text_is_rejected_when_shapes_and_icons_are_antialiased(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(
                root,
                antialiased=True,
                icon_antialiased=True,
                text_antialiased=False,
            )

            with self.assertRaisesRegex(
                VisualOracleError,
                "text runs do not provide fractional glyph edge coverage",
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_binary_rounded_control_is_rejected_as_aliased(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=False)

            with self.assertRaisesRegex(VisualOracleError, "fractional corner coverage"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_sparse_fractional_corner_noise_is_not_continuous_antialiasing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(
                root,
                antialiased=False,
                icon_antialiased=True,
                sparse_corner_coverage=True,
            )

            with self.assertRaisesRegex(
                VisualOracleError, "continuous four-corner coverage"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_tiny_rounded_radius_is_rejected_even_when_antialiased(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(
                root,
                antialiased=True,
                rounded_radius=3.0,
            )

            with self.assertRaisesRegex(
                VisualOracleError, "corner_radius must be at least"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_rounded_shape_geometry_rejects_missing_or_too_small_radius(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            manifest_payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(manifest_payload["captures"][0]["profile_geometry_path"])
            profile_payload = json.loads(profile.read_text(encoding="utf-8"))
            profile_payload["rounded_shapes"][0]["corner_radius"] = 3.5
            profile.write_text(json.dumps(profile_payload), encoding="utf-8")
            manifest_payload["captures"][0]["profile_geometry_sha256"] = (
                hashlib.sha256(profile.read_bytes()).hexdigest()
            )
            manifest.write_text(json.dumps(manifest_payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "corner_radius must be at least"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_binary_vector_icon_is_rejected_when_rounded_control_is_antialiased(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(
                root, antialiased=True, icon_antialiased=False
            )

            with self.assertRaisesRegex(
                VisualOracleError, "vector icons do not provide fractional edge coverage"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_layout_frame_outside_the_presented_surface_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(payload["captures"][0]["profile_geometry_path"])
            geometry = json.loads(profile.read_text(encoding="utf-8"))
            geometry["layout"]["document_region"].update(
                {"x": 60.0, "width": 20.0}
            )
            profile.write_text(json.dumps(geometry), encoding="utf-8")
            payload["captures"][0]["profile_geometry_sha256"] = hashlib.sha256(
                profile.read_bytes()
            ).hexdigest()
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(
                VisualOracleError, "layout frame exceeds screenshot bounds"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_empty_primary_layout_frame_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(payload["captures"][0]["profile_geometry_path"])
            geometry = json.loads(profile.read_text(encoding="utf-8"))
            geometry["layout"]["document_region"]["width"] = 0.0
            profile.write_text(json.dumps(geometry), encoding="utf-8")
            payload["captures"][0]["profile_geometry_sha256"] = hashlib.sha256(
                profile.read_bytes()
            ).hexdigest()
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(
                VisualOracleError, "required layout frame is empty"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_near_duplicate_clickable_frames_on_one_surface_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            profile = Path(payload["captures"][0]["profile_geometry_path"])
            geometry = json.loads(profile.read_text(encoding="utf-8"))
            duplicate = dict(geometry["clickable_frames"][0])
            duplicate["id"] = "activity.scene.duplicate"
            duplicate["frame"] = dict(duplicate["frame"])
            duplicate["frame"]["x"] += 0.25
            geometry["clickable_frames"].append(duplicate)
            profile.write_text(json.dumps(geometry), encoding="utf-8")
            payload["captures"][0]["profile_geometry_sha256"] = hashlib.sha256(
                profile.read_bytes()
            ).hexdigest()
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(
                VisualOracleError, "near-duplicate clickable frames overlap"
            ):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_fractional_dpi_control_frames_use_a_physical_coverage_box(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(
                root, antialiased=True, fractional_frame=True
            )

            report = validate_capture_manifest(
                manifest,
                expected_extents={(64, 64)},
                output_directory=root / "oracle",
            )

            self.assertTrue(report["passed"])
            self.assertGreater(report["captures"][0]["fractional_pixel_count"], 0)

    def test_gpu_backend_is_mandatory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True, backend="software")

            with self.assertRaisesRegex(VisualOracleError, "presenter_backend=gpu"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )

    def test_expected_physical_extent_set_must_be_complete(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)

            with self.assertRaisesRegex(VisualOracleError, "capture extents"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64), (80, 60)},
                    output_directory=root / "oracle",
                )

    def test_duplicate_capture_does_not_satisfy_one_process_per_extent(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = write_capture_fixture(root, antialiased=True)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            payload["captures"].append(dict(payload["captures"][0]))
            manifest.write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(VisualOracleError, "exactly one capture"):
                validate_capture_manifest(
                    manifest,
                    expected_extents={(64, 64)},
                    output_directory=root / "oracle",
                )


if __name__ == "__main__":
    unittest.main()
