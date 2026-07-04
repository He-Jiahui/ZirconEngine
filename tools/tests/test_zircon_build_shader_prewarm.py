import json
import subprocess
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch

from tools import zircon_build
from tools.zircon_build_shader_prewarm import (
    build_shader_prewarm_command,
    generated_shader_permutation_registry_document,
    print_shader_prewarm_plan,
    shader_prewarm_dimension_summary_lines,
    write_generated_shader_permutation_registry,
)
from tools.tests.shader_prewarm_test_support import (
    FakePluginPackage as _FakePluginPackage,
    FakePrewarmConfig as _FakePrewarmConfig,
)


class ZirconBuildShaderPrewarmTests(unittest.TestCase):
    def test_dimension_summary_lines_format_report_counts(self):
        report = {
            "dimension_summary": {
                "pass_types": {
                    "Forward": {"requested": 2, "written": 1, "failed": 1},
                    "Shadow": {"requested": 1, "written": 1, "failed": 0},
                },
                "geometry_source_ids": {
                    "0": {"requested": 3, "written": 2, "failed": 1},
                },
                "shading_model_ids": {
                    "2": {"requested": 3, "written": 3, "failed": 0},
                },
                "quality_tiers": {
                    "medium": {"requested": 3, "written": 2, "failed": 1},
                },
            }
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  pass types: Forward requested=2 written=1 failed=1; "
                "Shadow requested=1 written=1 failed=0",
                "  geometry source ids: 0 requested=3 written=2 failed=1",
                "  shading model ids: 2 requested=3 written=3 failed=0",
                "  quality tiers: medium requested=3 written=2 failed=1",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_dimension_summary_lines_ignore_older_reports(self):
        self.assertEqual((), shader_prewarm_dimension_summary_lines({}))

    def test_dimension_summary_lines_accept_rust_count_field_names(self):
        report = {
            "dimension_summary": {
                "pass_types": {
                    "forward": {
                        "requested_count": 2,
                        "written_count": 2,
                        "failed_count": 0,
                    },
                },
            }
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  pass types: forward requested=2 written=2 failed=0",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_dimension_summary_lines_format_wgpu_module_validation_counts(self):
        report = {
            "wgpu_module_validation": {
                "enabled": True,
                "requested_count": 3,
                "validated_count": 2,
                "failed_count": 1,
                "skipped_count": 0,
            },
            "dimension_summary": {
                "pass_types": {
                    "forward": {
                        "requested_count": 3,
                        "written_count": 2,
                        "failed_count": 1,
                    },
                },
            },
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  WGPU module validation: enabled requested=3 "
                "validated=2 failed=1 skipped=0",
                "  pass types: forward requested=3 written=2 failed=1",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_dimension_summary_lines_format_wgpu_pipeline_validation_counts(self):
        report = {
            "wgpu_pipeline_validation": {
                "enabled": True,
                "requested_count": 4,
                "validated_count": 4,
                "failed_count": 0,
                "skipped_count": 0,
            },
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  WGPU render pipeline validation: enabled requested=4 "
                "validated=4 failed=0 skipped=0",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_dimension_summary_lines_format_source_provenance(self):
        report = {
            "source_provenance": {
                "source_count": 1,
                "variant_count": 6,
                "sources": {
                    "res://shaders/example#source-a#template-a": {
                        "source_label": "res://shaders/example",
                        "source_hash": "source-a",
                        "template_revision": "template-a",
                        "requested_count": 6,
                        "written_count": 5,
                        "failed_count": 1,
                    },
                },
            },
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  source provenance: "
                "res://shaders/example source_hash=source-a "
                "template=template-a requested=6 written=5 failed=1",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_dimension_summary_lines_skip_malformed_entries(self):
        report = {
            "dimension_summary": {
                "pass_types": {
                    "Forward": "not counts",
                    "Shadow": {"requested": 1, "written": -4, "failed": True},
                    "Velocity": {},
                },
                "quality_tiers": [],
            }
        }

        self.assertEqual(
            (
                "shader prewarm dimension summary:",
                "  pass types: Shadow requested=1 written=0 failed=0",
            ),
            shader_prewarm_dimension_summary_lines(report),
        )

    def test_prewarm_shaders_prints_summary_before_raising_nonzero_exit(self):
        config = _FakePrewarmConfig()
        events: list[str] = []

        def fake_run(command, cwd, check):
            self.assertFalse(check)
            self.assertEqual(config.repo_root, cwd)
            events.append("run")
            return subprocess.CompletedProcess(command, 2)

        def fake_print_summary(report_path):
            events.append(f"summary:{report_path}")

        with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
            with patch.object(
                zircon_build,
                "print_shader_prewarm_report_dimensions",
                side_effect=fake_print_summary,
            ):
                with patch.object(
                    zircon_build,
                    "validate_staged_shader_prewarm_acceptance_contract",
                    side_effect=AssertionError("nonzero prewarm should not validate"),
                ):
                    with self.assertRaises(subprocess.CalledProcessError):
                        zircon_build.prewarm_shaders(config)

        self.assertEqual(
            ["run", f"summary:{config.shader_prewarm_report_path}"],
            events,
        )

    def test_prewarm_shaders_validates_staged_acceptance_after_success(self):
        config = _FakePrewarmConfig()
        config.validate_wgpu_shaders = True
        config.shader_quality_tiers = ("medium", "high")
        config.shader_geometry_sources = ("static", "skinned")
        config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
        config.shader_shading_model_ids = ("toon=16",)
        events: list[str] = []

        def fake_run(command, cwd, check):
            self.assertFalse(check)
            self.assertEqual(config.repo_root, cwd)
            events.append("run")
            return subprocess.CompletedProcess(command, 0)

        def fake_print_summary(report_path):
            events.append(f"summary:{report_path}")

        with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
            with patch.object(
                zircon_build,
                "print_shader_prewarm_report_dimensions",
                side_effect=fake_print_summary,
            ):
                with patch.object(
                    zircon_build,
                    "validate_staged_shader_prewarm_acceptance_contract",
                    side_effect=lambda actual_config: events.append(
                        f"acceptance:{actual_config is config}"
                    ),
                ):
                    zircon_build.prewarm_shaders(config)

        self.assertEqual(
            [
                "run",
                f"summary:{config.shader_prewarm_report_path}",
                "acceptance:True",
            ],
            events,
        )

    def test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry(self):
        config = _FakePrewarmConfig()
        config.shader_resource_registry = Path("Project") / "shader_resource_records.json"
        events: list[str] = []

        def fake_run(command, cwd, check):
            self.assertFalse(check)
            self.assertEqual(config.repo_root, cwd)
            events.append("run")
            return subprocess.CompletedProcess(command, 0)

        def fake_print_summary(report_path):
            events.append(f"summary:{report_path}")

        with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
            with patch.object(
                zircon_build,
                "print_shader_prewarm_report_dimensions",
                side_effect=fake_print_summary,
            ):
                with patch.object(
                    zircon_build,
                    "validate_staged_shader_prewarm_acceptance_contract",
                    side_effect=lambda actual_config: events.append(
                        f"acceptance:{actual_config.shader_resource_registry}"
                    ),
                ):
                    zircon_build.prewarm_shaders(config)

        self.assertEqual(
            [
                "run",
                f"summary:{config.shader_prewarm_report_path}",
                f"acceptance:{config.shader_resource_registry}",
            ],
            events,
        )

    def test_build_command_forwards_shader_permutation_registries(self):
        config = _FakePrewarmConfig()
        config.shader_permutation_registries = (
            Path("Project") / "shader_permutation_registry.json",
            Path("Plugin") / "shader_permutation_registry.json",
        )

        command = build_shader_prewarm_command(config)

        self.assertIn("--shader-permutation-registry", command)
        first = command.index("--shader-permutation-registry")
        second = command.index("--shader-permutation-registry", first + 1)
        self.assertEqual(
            str(Path("Project") / "shader_permutation_registry.json"),
            command[first + 1],
        )
        self.assertEqual(
            str(Path("Plugin") / "shader_permutation_registry.json"),
            command[second + 1],
        )

    def test_build_command_uses_generated_shader_permutation_registry_for_custom_ids(self):
        config = _FakePrewarmConfig()
        config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
        config.shader_shading_model_ids = ("toon=16",)

        command = build_shader_prewarm_command(config)

        registry_index = command.index("--shader-permutation-registry")
        self.assertEqual(
            str(config.shader_prewarm_permutation_registry_path),
            command[registry_index + 1],
        )

    def test_build_command_prefers_explicit_shader_permutation_registry(self):
        config = _FakePrewarmConfig()
        config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
        config.shader_shading_model_ids = ("toon=16",)
        config.shader_permutation_registries = (
            Path("Project") / "shader_permutation_registry.json",
        )

        command = build_shader_prewarm_command(config)

        registry_index = command.index("--shader-permutation-registry")
        self.assertEqual(
            str(Path("Project") / "shader_permutation_registry.json"),
            command[registry_index + 1],
        )
        self.assertNotIn(str(config.shader_prewarm_permutation_registry_path), command)

    def test_generated_shader_permutation_registry_document_groups_custom_ids(self):
        config = _FakePrewarmConfig()
        config.shader_geometry_source_ids = ("gpu-driven=4",)
        config.shader_shading_model_ids = ("custom:toon=16",)

        self.assertEqual(
            {
                "geometry_source_ids": [{"token": "custom:gpu-driven", "id": 4}],
                "geometry_source_descriptors": [],
                "shading_model_ids": [{"token": "custom:toon", "id": 16}],
                "shading_model_descriptors": [],
                "shader_modules": [],
            },
            generated_shader_permutation_registry_document(config),
        )

    def test_generated_shader_permutation_registry_document_merges_selected_plugin_ids(self):
        config = _FakePrewarmConfig()
        config.plugins = (
            _FakePluginPackage(
                shader_geometry_source_ids=("custom:virtual_geometry=4",),
                shader_shading_model_ids=("custom:toon=16",),
            ),
        )

        self.assertEqual(
            {
                "geometry_source_ids": [
                    {"token": "custom:virtual_geometry", "id": 4}
                ],
                "geometry_source_descriptors": [],
                "shading_model_ids": [{"token": "custom:toon", "id": 16}],
                "shading_model_descriptors": [],
                "shader_modules": [],
            },
            generated_shader_permutation_registry_document(config),
        )

    def test_generated_shader_permutation_registry_document_exports_selected_plugin_descriptors(self):
        config = _FakePrewarmConfig()
        descriptor = {
            "id": 4,
            "token": "custom:virtual_geometry",
            "wgsl_include": "zr_geometry_virtual_geometry.wgsl",
            "vertex_attributes": ["position", "normal", "tangent", "uv0"],
            "required_bindings": [
                {
                    "kind": "virtual_geometry_pages",
                    "slot_token": "virtual_geometry.pages",
                },
                {
                    "kind": "virtual_geometry_clusters",
                    "slot_token": "virtual_geometry.clusters",
                }
            ],
            "shader_defines": [
                {
                    "kind": "bool",
                    "name": "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
                    "value": True,
                }
            ],
        }
        config.plugins = (
            _FakePluginPackage(
                shader_geometry_source_ids=("custom:virtual_geometry=4",),
                shader_geometry_source_descriptors=(descriptor,),
            ),
        )

        self.assertEqual(
            {
                "geometry_source_ids": [
                    {"token": "custom:virtual_geometry", "id": 4}
                ],
                "geometry_source_descriptors": [descriptor],
                "shading_model_ids": [],
                "shading_model_descriptors": [],
                "shader_modules": [],
            },
            generated_shader_permutation_registry_document(config),
        )

    def test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors(self):
        config = _FakePrewarmConfig()
        descriptor = {
            "id": 16,
            "token": "custom:toon",
            "forward_include": "zr_shading_toon_forward.wgsl",
            "gbuffer_encode_include": "zr_shading_toon_gbuffer.wgsl",
            "deferred_include": "zr_shading_toon_deferred.wgsl",
            "required_channels": 7,
        }
        config.plugins = (
            _FakePluginPackage(
                shader_shading_model_ids=("custom:toon=16",),
                shader_shading_model_descriptors=(descriptor,),
            ),
        )

        self.assertEqual(
            {
                "geometry_source_ids": [],
                "geometry_source_descriptors": [],
                "shading_model_ids": [{"token": "custom:toon", "id": 16}],
                "shading_model_descriptors": [descriptor],
                "shader_modules": [],
            },
            generated_shader_permutation_registry_document(config),
        )

    def test_generated_shader_permutation_registry_document_exports_selected_plugin_shader_modules(self):
        config = _FakePrewarmConfig()
        config.plugins = (
            _FakePluginPackage(
                shader_modules=(
                    {
                        "import_path": "custom::toon::noise",
                        "source": "assets/shaders/noise.zshader",
                        "content_hash": "a" * 64,
                    },
                ),
            ),
        )

        self.assertEqual(
            {
                "geometry_source_ids": [],
                "geometry_source_descriptors": [],
                "shading_model_ids": [],
                "shading_model_descriptors": [],
                "shader_modules": [
                    {
                        "import_path": "custom::toon::noise",
                        "content_hash": "a" * 64,
                        "source": "assets/shaders/noise.zshader",
                    }
                ],
            },
            generated_shader_permutation_registry_document(config),
        )

    def test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_ids(self):
        config = _FakePrewarmConfig()
        config.plugins = (
            _FakePluginPackage(shader_geometry_source_ids=("custom:virtual_geometry=4",)),
        )

        command = build_shader_prewarm_command(config)

        registry_index = command.index("--shader-permutation-registry")
        self.assertEqual(
            str(config.shader_prewarm_permutation_registry_path),
            command[registry_index + 1],
        )
        self.assertNotIn("custom:virtual_geometry=4", command)

    def test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_modules(self):
        config = _FakePrewarmConfig()
        config.plugins = (
            _FakePluginPackage(
                shader_modules=(
                    {
                        "import_path": "custom::toon::noise",
                        "content_hash": "a" * 64,
                    },
                ),
            ),
        )

        command = build_shader_prewarm_command(config)

        registry_index = command.index("--shader-permutation-registry")
        self.assertEqual(
            str(config.shader_prewarm_permutation_registry_path),
            command[registry_index + 1],
        )

    def test_build_command_includes_selected_plugin_asset_roots(self):
        config = _FakePrewarmConfig()
        config.plugins = (
            _FakePluginPackage(asset_roots=(Path("plugins") / "toon" / "assets",)),
        )

        command = build_shader_prewarm_command(config)

        asset_root_indices = [
            index
            for index, argument in enumerate(command)
            if argument == "--asset-root"
        ]
        self.assertEqual(2, len(asset_root_indices))
        self.assertEqual(
            str(config.engine_root / "assets"),
            command[asset_root_indices[0] + 1],
        )
        self.assertEqual(
            str(Path("plugins") / "toon" / "assets"),
            command[asset_root_indices[1] + 1],
        )

    def test_build_command_auto_export_registry_scans_all_asset_roots(self):
        config = _FakePrewarmConfig()
        config.shader_asset_roots = (
            Path("Project") / "assets",
            Path("Project") / "generated" / "shaders",
        )
        config.plugins = (
            _FakePluginPackage(asset_roots=(Path("plugins") / "toon" / "assets",)),
            _FakePluginPackage(asset_roots=(Path("plugins") / "vfx" / "assets",)),
        )

        command = build_shader_prewarm_command(config)

        asset_roots = [
            command[index + 1]
            for index, argument in enumerate(command)
            if argument == "--asset-root"
        ]
        self.assertEqual(
            [
                str(config.engine_root / "assets"),
                str(Path("Project") / "assets"),
                str(Path("Project") / "generated" / "shaders"),
                str(Path("plugins") / "toon" / "assets"),
                str(Path("plugins") / "vfx" / "assets"),
            ],
            asset_roots,
        )
        export_index = command.index("--export-resource-registry")
        self.assertEqual(
            str(config.shader_prewarm_resource_registry_path),
            command[export_index + 1],
        )

    def test_build_command_auto_export_registry_uses_native_dynamic_fixture_assets(self):
        repo_root = Path(__file__).resolve().parents[2]
        packages = {
            package.plugin_id: package
            for package in zircon_build.discover_plugins(repo_root)
        }
        plugin = packages["native_dynamic_fixture"]
        plugin_asset_root = (
            repo_root / "zircon_plugins" / "native_dynamic_fixture" / "assets"
        )

        self.assertIn(plugin_asset_root, plugin.asset_roots)
        self.assertTrue((plugin_asset_root / "shader.wgsl.zmeta").is_file())

        config = _FakePrewarmConfig()
        config.plugins = (plugin,)
        command = build_shader_prewarm_command(config)

        asset_roots = [
            command[index + 1]
            for index, argument in enumerate(command)
            if argument == "--asset-root"
        ]
        self.assertIn(str(plugin_asset_root), asset_roots)
        export_index = command.index("--export-resource-registry")
        self.assertEqual(
            str(config.shader_prewarm_resource_registry_path),
            command[export_index + 1],
        )

    def test_cli_selects_native_dynamic_fixture_assets_for_prewarm_command(self):
        repo_root = Path(__file__).resolve().parents[2]
        out_root = repo_root / "target" / "prewarm-native-dynamic-fixture-cli-test"
        plugin_asset_root = (
            repo_root / "zircon_plugins" / "native_dynamic_fixture" / "assets"
        )
        args = zircon_build.parse_args(
            [
                "--targets",
                "runtime",
                "--plugins",
                "native_dynamic_fixture",
                "--out",
                str(out_root),
                "--mode",
                "debug",
                "--prewarm-shaders",
            ]
        )

        config = zircon_build.resolve_config(
            args,
            repo_root,
            zircon_build.discover_plugins(repo_root),
        )
        command = build_shader_prewarm_command(config)

        self.assertEqual(("runtime",), config.targets)
        self.assertEqual(
            ("native_dynamic_fixture",),
            tuple(plugin.plugin_id for plugin in config.plugins),
        )
        self.assertIn(plugin_asset_root, config.plugins[0].asset_roots)
        self.assertTrue((plugin_asset_root / "shader.wgsl.zmeta").is_file())
        asset_roots = [
            command[index + 1]
            for index, argument in enumerate(command)
            if argument == "--asset-root"
        ]
        self.assertIn(str(plugin_asset_root), asset_roots)
        self.assertIn("--export-resource-registry", command)
        self.assertNotIn("--resource-registry", command)

    def test_cli_dry_run_prints_native_dynamic_fixture_prewarm_command(self):
        repo_root = Path(__file__).resolve().parents[2]
        out_root = repo_root / "target" / "prewarm-native-dynamic-fixture-dry-run-test"
        plugin_asset_root = (
            repo_root / "zircon_plugins" / "native_dynamic_fixture" / "assets"
        )
        output = StringIO()

        with redirect_stdout(output):
            exit_code = zircon_build.main(
                [
                    "--targets",
                    "runtime",
                    "--plugins",
                    "native_dynamic_fixture",
                    "--out",
                    str(out_root),
                    "--mode",
                    "debug",
                    "--prewarm-shaders",
                    "--dry-run",
                ]
            )

        text = output.getvalue()
        self.assertEqual(0, exit_code)
        self.assertIn("DRY-RUN", text)
        self.assertIn("zircon_shader_prewarm", text)
        self.assertIn(str(plugin_asset_root), text)
        self.assertIn("--export-resource-registry", text)
        self.assertNotIn("--resource-registry ", text)

    def test_prewarm_plan_lists_asset_roots_for_registry_export(self):
        config = _FakePrewarmConfig()
        config.shader_asset_roots = (Path("Project") / "assets",)
        config.plugins = (
            _FakePluginPackage(asset_roots=(Path("plugins") / "toon" / "assets",)),
            _FakePluginPackage(asset_roots=(Path("plugins") / "vfx" / "assets",)),
        )
        output = StringIO()

        with redirect_stdout(output):
            print_shader_prewarm_plan(config)

        self.assertIn(
            "  shader asset roots: "
            + ",".join(
                [
                    str(config.engine_root / "assets"),
                    str(Path("Project") / "assets"),
                    str(Path("plugins") / "toon" / "assets"),
                    str(Path("plugins") / "vfx" / "assets"),
                ]
            ),
            output.getvalue(),
        )
        self.assertIn(
            f"  shader resource registry export: {config.shader_prewarm_resource_registry_path}",
            output.getvalue(),
        )

    def test_zircon_build_resolves_project_shader_asset_roots_for_prewarm(self):
        args = zircon_build.parse_args(
            [
                "--targets",
                "runtime",
                "--out",
                "target/project-shader-roots",
                "--mode",
                "debug",
                "--prewarm-shaders",
                "--shader-asset-root",
                "Project/assets",
                "--shader-asset-root",
                "Project/generated/shaders",
            ]
        )

        config = zircon_build.resolve_config(args, Path("."), ())

        self.assertEqual(
            (
                (Path.cwd() / "Project" / "assets").resolve(),
                (Path.cwd() / "Project" / "generated" / "shaders").resolve(),
            ),
            config.shader_asset_roots,
        )

    def test_prewarm_plan_lists_runtime_fallback_handoff_paths(self):
        config = _FakePrewarmConfig()
        output = StringIO()

        with redirect_stdout(output):
            print_shader_prewarm_plan(config)

        self.assertIn(
            "  shader prewarm cache root: " f"{config.shader_prewarm_cache_root}",
            output.getvalue(),
        )
        self.assertIn(
            "  shader prewarm report: " f"{config.shader_prewarm_report_path}",
            output.getvalue(),
        )
        self.assertIn(
            "  shader runtime fallback root: "
            f"{config.engine_root / 'cache' / 'shader_variants'}",
            output.getvalue(),
        )

    def test_build_command_forwards_wgpu_shader_module_validation(self):
        config = _FakePrewarmConfig()
        config.validate_wgpu_shaders = True

        command = build_shader_prewarm_command(config)

        self.assertIn("--validate-wgpu-modules", command)

    def test_build_command_forwards_wgpu_shader_pipeline_validation(self):
        config = _FakePrewarmConfig()
        config.validate_wgpu_pipelines = True

        command = build_shader_prewarm_command(config)

        self.assertIn("--validate-wgpu-pipelines", command)

    def test_write_generated_shader_permutation_registry_writes_json(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            config = _FakePrewarmConfig()
            config.engine_root = Path(temp_dir) / "ZirconEngine"
            config.shader_geometry_source_ids = ("custom:gpu-driven=4",)

            written_path = write_generated_shader_permutation_registry(config)

            self.assertEqual(config.shader_prewarm_permutation_registry_path, written_path)
            self.assertEqual(
                {
                    "geometry_source_ids": [{"token": "custom:gpu-driven", "id": 4}],
                    "geometry_source_descriptors": [],
                    "shading_model_ids": [],
                    "shading_model_descriptors": [],
                    "shader_modules": [],
                },
                json.loads(written_path.read_text(encoding="utf-8")),
            )

if __name__ == "__main__":
    unittest.main()
