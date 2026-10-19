from __future__ import annotations

import importlib.util
import re
import subprocess
import unittest
from unittest import mock
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "ci.yml"
SCOPE_PATH = REPO_ROOT / ".github/scripts/ci-scope.py"
SPEC = importlib.util.spec_from_file_location("ci_scope_contract", SCOPE_PATH)
SCOPE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCOPE)


def job_body(workflow: str, name: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(name)}:\s*$\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\s*$|\Z)",
        workflow,
    )
    if match is None:
        raise AssertionError(f"CI job {name!r} is missing")
    return match.group("body")


class CiGatesContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_persistent_runners_are_trusted_and_run_isolated(self) -> None:
        for path in (WORKFLOW_PATH, WORKFLOW_PATH.with_name("profile-feature-contract.yml")):
            workflow = path.read_text(encoding="utf-8")
            self.assertIn("permissions:\n  contents: read", workflow)
            self.assertIn("concurrency:", workflow)
            for name in re.findall(r"(?m)^  ([a-z0-9-]+):$", workflow):
                body = job_body(workflow, name)
                if "runs-on: [self-hosted, Windows, X64]" not in body:
                    continue
                self.assertIn("if: github.event_name == 'push'", body, name)
                self.assertIn("persist-credentials: false", body, name)
                for variable in ("CARGO_HOME", "RUSTUP_HOME", "TEMP", "TMP", "CARGO_TARGET_DIR"):
                    self.assertRegex(body, rf"(?m)^      {variable}: .*github.run_id.*github.run_attempt.*strategy.job-index", name)
        preparation = (REPO_ROOT / ".github/scripts/prepare-cargo-roots.ps1").read_text()
        self.assertIn("Refusing to reuse a prior CI output tree", preparation)
        self.assertIn("Assert-NoReparseAncestors $directory", preparation)

    def test_workflow_unittest_modules_are_tracked(self) -> None:
        tracked = set(subprocess.check_output(
            ["git", "ls-files", "-z"], cwd=REPO_ROOT
        ).decode().split("\0"))
        count = 0
        for workflow in (REPO_ROOT / ".github/workflows").glob("*.yml"):
            for match in re.finditer(r"-m unittest ([^\n]+)", workflow.read_text(encoding="utf-8")):
                for module in match.group(1).split():
                    if module.startswith("-"):
                        continue
                    path = module.replace(".", "/") + ".py"
                    self.assertIn(path, tracked, f"{workflow.name}: {module}")
                    self.assertTrue((REPO_ROOT / path).is_file(), path)
                    count += 1
        self.assertGreater(count, 0)

    def test_all_workflows_pin_actions_and_explicit_rust_toolchains(self) -> None:
        workflows = sorted((REPO_ROOT / ".github/workflows").glob("*.y*ml"))
        self.assertGreater(len(workflows), 0)
        for path in workflows:
            source = path.read_text(encoding="utf-8")
            for action in re.findall(r"(?m)^\s*(?:-\s*)?uses:\s*(\S+)", source):
                if action.startswith("./"):
                    continue
                self.assertRegex(action, r"^[^@]+@[0-9a-f]{40}$", path.name)
            for step in re.split(r"(?m)^      - ", source):
                if "uses: dtolnay/rust-toolchain@" not in step:
                    continue
                match = re.search(r"(?m)^          toolchain: (.+)$", step)
                self.assertIsNotNone(match, path.name)
                toolchain = match.group(1)
                if toolchain == "${{ env.MVP_RUST_TOOLCHAIN }}":
                    self.assertIn("MVP_RUST_TOOLCHAIN: 1.94.1", source)
                else:
                    self.assertIn(toolchain, ("1.94.1", "nightly-2026-10-10"), path.name)

    def assert_workspace_member_fmt_gate(self, rust: str, manifest: str) -> None:
        step_name = "Check Rust formatting" if manifest == "Cargo.toml" else "Check plugin workspace formatting"
        match = re.search(
            rf"(?ms)^      - name: {re.escape(step_name)}\s*$\n(?P<step>.*?)(?=^      - name:|\Z)",
            rust,
        )
        self.assertIsNotNone(match, f"missing formatting step for {manifest}")
        step = match.group("step")
        for required in (
            "shell: pwsh",
            f"$manifest = '{manifest}'",
            "cargo +1.94.1 metadata --manifest-path $manifest --format-version 1 --no-deps --locked",
            "$LASTEXITCODE -ne 0",
            "$metadata.workspace_members",
            "$memberIdSet.Contains([string]$_.id)",
            "$members.Count -ne $memberIds.Count",
            "Duplicate workspace member ID",
            "Missing or duplicate workspace package name",
            "foreach ($name in $names) { $fmtArguments += @('--package', $name) }",
            "@('--', '--check')",
        ):
            self.assertIn(required, step)
        self.assertNotIn("--all", step)
        self.assertNotIn("workspace_default_members", step)

    def test_both_rust_workspaces_format_every_metadata_member(self) -> None:
        rust = job_body(self.workflow, "rust")
        self.assert_workspace_member_fmt_gate(rust, "Cargo.toml")
        self.assert_workspace_member_fmt_gate(rust, "zircon_plugins/Cargo.toml")

    def test_rust_job_runs_format_clippy_and_path_audit(self) -> None:
        rust = job_body(self.workflow, "rust")
        self.assert_workspace_member_fmt_gate(rust, "Cargo.toml")
        self.assertIn(
            'ci-scope.py run --base "$env:CI_SCOPE_BASE" --head "$env:CI_SCOPE_HEAD" --group rust',
            rust,
        )
        self.assertIn("python -B -m unittest tools.tests.test_path_names -v", job_body(self.workflow, "source-contracts"))
        self.assertIn("python -B tools/audits/path_names.py --summary", job_body(self.workflow, "source-contracts"))

    def test_plugins_workspace_has_its_own_format_and_lint_gates(self) -> None:
        rust = job_body(self.workflow, "rust")
        self.assert_workspace_member_fmt_gate(rust, "zircon_plugins/Cargo.toml")
        self.assertIn(
            'ci-scope.py run --base "$env:CI_SCOPE_BASE" --head "$env:CI_SCOPE_HEAD" --group rust',
            rust,
        )

    def test_miri_executes_storage_table_and_mutable_query_tests(self) -> None:
        miri = job_body(self.workflow, "scene-ecs-miri")
        self.assertIn("invoke-cargo-msvc.ps1 cargo +nightly-2026-10-10 miri setup --target x86_64-unknown-linux-gnu", miri)
        self.assertLess(
            miri.index("install-zig-linux-tools.ps1"),
            miri.index("miri setup --target x86_64-unknown-linux-gnu"),
        )
        self.assertIn("--target x86_64-unknown-linux-gnu", miri)
        self.assertIn("--features native-host", miri)
        self.assertEqual(miri.count("-RequireMatchingTests"), 3)
        storage_filter = "scene::ecs::storage::component_storage::store::tests::sparse_rows_rekey_at_the_target_tick"
        self.assertIn(storage_filter, miri)
        self.assertIn("scene::ecs::archetype::table::tests", miri)
        self.assertIn(
            "scene::tests::ecs_query_many::unsafe_custom_mutable_query_preserves_real_borrows_and_change_ticks",
            miri,
        )
        runtime_root = REPO_ROOT / "zircon_runtime" / "src"
        storage_test = (
            runtime_root
            / "scene"
            / "ecs"
            / "storage"
            / "component_storage"
            / "tests"
            / "store.rs"
        ).read_text(encoding="utf-8")
        table_tests = (
            runtime_root
            / "scene"
            / "ecs"
            / "archetype"
            / "table"
            / "tests"
            / "cases.rs"
        ).read_text(encoding="utf-8")
        query_tests = (
            runtime_root / "scene" / "tests" / "ecs_query_many.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("fn sparse_rows_rekey_at_the_target_tick(", storage_test)
        self.assertIn("fn archetype_table_keeps_each_column_row_aligned(", table_tests)
        self.assertIn(
            "fn unsafe_custom_mutable_query_preserves_real_borrows_and_change_ticks(",
            query_tests,
        )
        runtime_lib = (runtime_root / "lib.rs").read_text(encoding="utf-8")
        self.assertRegex(
            runtime_lib,
            r'(?s)#\[cfg\(any\(\s*feature = "native-host",\s*feature = "graphics",\s*feature = "animation",\s*feature = "ui"\s*\)\)\]\s*#\[path = "scene/mod.rs"\]\s*pub mod scene;',
        )
        self.assertIn("#[cfg(all(test, feature = \"native-host\"))]", runtime_lib)

    def test_miri_uses_pinned_zig_frontends_and_validates_cached_archive(self) -> None:
        miri = job_body(self.workflow, "scene-ecs-miri")
        script_root = REPO_ROOT / ".github" / "scripts"
        bootstrap = (script_root / "install-zig-linux-tools.ps1").read_text(encoding="utf-8")
        self.assertIn("https://ziglang.org/download/0.14.1/zig-x86_64-windows-0.14.1.zip", bootstrap)
        self.assertIn("554f5378228923ffd558eac35e21af020c73789d87afeabf4bfd16f2e6feed2c", bootstrap)
        self.assertIn("82229343L", bootstrap)
        self.assertIn("E:\\cargo-targets\\zircon-local\\ci\\runs\\", bootstrap)
        self.assertIn("Assert-NoReparseAncestors", bootstrap)
        self.assertIn("Assert-InstallMatchesArchive", bootstrap)
        env_frontends = {
            "CC_x86_64_unknown_linux_gnu": "zig-cc-x86_64-unknown-linux-gnu.cmd",
            "CXX_x86_64_unknown_linux_gnu": "zig-cxx-x86_64-unknown-linux-gnu.cmd",
            "AR_x86_64_unknown_linux_gnu": "zig-ar-x86_64-unknown-linux-gnu.cmd",
            "RANLIB_x86_64_unknown_linux_gnu": "zig-ranlib-x86_64-unknown-linux-gnu.cmd",
        }
        for variable, frontend in env_frontends.items():
            self.assertIn(f"{variable}:", miri)
            self.assertIn(f".github\\scripts\\{frontend}", miri)
            contents = (script_root / frontend).read_text(encoding="utf-8")
            self.assertIn("%ZR_CI_OUTPUT_ROOT%\\zig-0.14.1\\zig.exe", contents)
        for frontend in (
            "zig-cc-x86_64-unknown-linux-gnu.cmd",
            "zig-cxx-x86_64-unknown-linux-gnu.cmd",
        ):
            contents = (script_root / frontend).read_text(encoding="utf-8")
            self.assertIn("%* -target x86_64-linux-gnu", contents)
        self.assertIn("ZIG_LOCAL_CACHE_DIR:", miri)
        self.assertIn("ZIG_GLOBAL_CACHE_DIR:", miri)
        self.assertNotIn("MIRIFLAGS:", miri)

    def test_cargo_jobs_use_separate_physical_output_roots(self) -> None:
        for name in (
            "runtime-profile-feature-matrix",
            "runtime-domain-feature-matrix",
            "rust",
            "plugin-standalone-dist",
            "scene-ecs-miri",
            "render-feature-smoke",
            "export-platform-contract",
        ):
            body = job_body(self.workflow, name)
            self.assertIn("runs-on: [self-hosted, Windows, X64]", body, name)
            self.assertIn("CARGO_TARGET_DIR: E:\\cargo-targets\\zircon-local\\ci\\", body, name)
            self.assertIn("CARGO_HOME: E:\\cargo-targets\\zircon-local\\ci\\runs\\", body, name)
            self.assertIn("prepare-cargo-roots.ps1", body, name)
            self.assertNotIn("local-cargo.ps1", body, name)

    def test_render_smoke_matrix_covers_every_rendering_feature(self) -> None:
        smoke = job_body(self.workflow, "render-feature-smoke")
        self.assertEqual(len(SCOPE.RENDER_ROWS), 15)
        self.assertIn(
            "CARGO_TARGET_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\render-feature-smoke-${{ strategy.job-index || 0 }}\\target",
            smoke,
        )
        self.assertIn(
            "ZIRCON_RENDERING_SMOKE_OUTPUT_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\render-feature-smoke-${{ strategy.job-index || 0 }}\\target\\product-frame",
            smoke,
        )
        self.assertIn('ci-scope.py run --base "$env:CI_SCOPE_BASE" --head "$env:CI_SCOPE_HEAD" --recipe "$env:CI_RECIPE"', smoke)
        self.assertIn('command += ["-RequireMatchingTests"]', SCOPE_PATH.read_text(encoding="utf-8"))
        self.assertNotIn("--exact", smoke)

        expected = {
            "baked_lighting": (
                "zircon_plugin_rendering_baked_lighting_runtime",
                "baked_lighting_feature_executes_in_nonempty_wgpu_product_frame",
                "zircon_plugins/rendering/features/baked_lighting/runtime/src/tests/lib.rs",
            ),
            "decals": (
                "zircon_plugin_rendering_decals_runtime",
                "decals_feature_executes_in_nonempty_wgpu_product_frame",
                "zircon_plugins/rendering/features/decals/runtime/src/tests/lib.rs",
            ),
            "contact_shadow": (
                "zircon_plugin_rendering_contact_shadow_runtime",
                "contact_shadow_wgpu_product_capture_darkens_",
                "zircon_plugins/rendering/features/contact_shadow/runtime/src/tests/gpu.rs",
            ),
            "irradiance_volumes": (
                "zircon_runtime",
                "render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame",
                "zircon_runtime/src/graphics/tests/render_product_advanced_lighting/af_m2.rs",
            ),
            "light_cookies": (
                "zircon_runtime",
                "render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame",
                "zircon_runtime/src/graphics/tests/render_product_advanced_lighting/af_m2.rs",
            ),
            "oit": (
                "zircon_runtime",
                "render_product_advanced_lighting_oit_three_crossing_planes_differs_from_sorted",
                "zircon_runtime/src/graphics/tests/render_product_advanced_lighting.rs",
            ),
            "planar_reflections": (
                "zircon_runtime",
                "render_product_planar_reflection_changes_mirror_floor_through_camera_loop",
                "zircon_runtime/src/graphics/tests/render_product_planar_reflection.rs",
            ),
            "post_process": (
                "zircon_runtime",
                "render_product_post_uber_light_effects_change_final_frame",
                "zircon_runtime/src/graphics/tests/render_product_post_process.rs",
            ),
            "reflection_probes": (
                "zircon_runtime",
                "render_product_probe_feature_off_falls_back_to_skybox",
                "zircon_runtime/src/graphics/tests/project_scenes/reflection_probe_product.rs",
            ),
            "ssao": (
                "zircon_runtime",
                "ssao_product_default_is_fail_closed_and_preserves_scene_output",
                "zircon_runtime/src/graphics/tests/project_render/render_quality.rs",
            ),
            "subsurface_scattering": (
                "zircon_runtime",
                "render_product_sss_skin_sphere_executes_burley_indirect_pipeline",
                "zircon_runtime/src/graphics/tests/render_product_subsurface.rs",
            ),
            "ray_tracing_policy": (
                "zircon_plugin_rendering_ray_tracing_policy_runtime",
                "ray_tracing_policy_reports_backend_gates_and_keeps_a_real_product_frame",
                "zircon_plugins/rendering/features/ray_tracing_policy/runtime/src/tests/lib.rs",
            ),
            "shader_graph": (
                "zircon_plugin_rendering_shader_graph_runtime",
                "shader_graph_feature_executes_a_real_product_frame",
                "zircon_plugins/rendering/features/shader_graph/runtime/src/tests/lib.rs",
            ),
            "vfx_graph": (
                "zircon_plugin_rendering_vfx_graph_runtime",
                "vfx_graph_feature_executes_a_real_product_frame",
                "zircon_plugins/rendering/features/vfx_graph/runtime/src/tests/lib.rs",
            ),
            "volumetric_fog": (
                "zircon_plugin_rendering_volumetric_fog_runtime",
                "smoke_volumetric_fog_executes_a_product_frame",
                "zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests/gpu.rs",
            ),
        }
        rows = {row["feature"]: row for row in SCOPE.RENDER_ROWS}
        self.assertEqual(set(rows), set(expected))
        for feature, (package, selector, source_path) in expected.items():
            row = rows[feature]
            self.assertEqual(row["package"], package, feature)
            self.assertEqual(row["filter"], selector, feature)
            self.assertEqual(row["manifest"], "Cargo.toml" if package == "zircon_runtime" else "zircon_plugins/Cargo.toml")
            if package == "zircon_runtime":
                self.assertEqual(row["args"], "--lib --no-default-features --features native-host,graphics", feature)
            else:
                self.assertEqual(row["args"], "--lib --no-default-features", feature)
            source = (REPO_ROOT / source_path).read_text(encoding="utf-8")
            if feature == "contact_shadow":
                matching_tests = re.findall(
                    r"(?m)^fn (contact_shadow_wgpu_product_capture_darkens_\w+)\(",
                    source,
                )
                self.assertGreaterEqual(len(matching_tests), 2)
                self.assertTrue(all(name.startswith(selector) for name in matching_tests))
            else:
                self.assertIn(f"fn {selector}(", source, feature)
        for wrapper_selector in (
            "irradiance_volume_feature_is_optional_and_registers_binding_executor",
            "light_cookie_feature_is_optional_and_registers_atlas_executor",
            "sss_feature_registers_three_passes_executors_and_shading_model",
        ):
            self.assertIn(wrapper_selector, {selector for _, selector in SCOPE.RENDER_WRAPPERS.values()})

    def test_test_filter_guard_lists_matches_and_refuses_zero_before_execution(self) -> None:
        helper = (
            REPO_ROOT / ".github" / "scripts" / "invoke-cargo-msvc.ps1"
        ).read_text(encoding="utf-8")
        self.assertIn("@('--', '--list')", helper)
        self.assertIn("'^\\s*\\S.*: test$'", helper)
        self.assertIn("$matchingTests.Count -eq 0", helper)
        self.assertIn("refusing a vacuous CI pass", helper)
        self.assertIn("$testOutput = & $Executable @ExecutableArguments 2>&1", helper)
        self.assertIn("test result: ok\\.\\s+([1-9][0-9]*) passed;", helper)
        self.assertIn("completed without any passed tests", helper)
        self.assertLess(
            helper.index("$matchingTests.Count -eq 0"),
            helper.index("$testOutput = & $Executable @ExecutableArguments"),
        )
        self.assertLess(
            helper.index("completed without any passed tests"),
            helper.index("exit 0", helper.index("$testOutput = & $Executable")),
        )

    def test_renderer_frame_smokes_are_active_and_have_ci_admission(self) -> None:
        contact_tests = (
            REPO_ROOT
            / "zircon_plugins/rendering/features/contact_shadow/runtime/src/tests/gpu.rs"
        ).read_text(encoding="utf-8")
        volumetric_tests = (
            REPO_ROOT
            / "zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests/gpu.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "fn contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region(",
            contact_tests,
        )
        self.assertIn(
            "fn contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions(",
            contact_tests,
        )
        self.assertIn(".submit_frame_extract(viewport, frame_extract)", contact_tests)
        self.assertIn(".capture_frame(viewport)", contact_tests)
        self.assertIn("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR", volumetric_tests)
        self.assertIn(
            "fn export_volumetric_compiled_scene_window_light_shaft_perf_wgpu_png(",
            volumetric_tests,
        )
        smoke = job_body(self.workflow, "render-feature-smoke")
        self.assertIn("contact_shadow_wgpu_product_capture_darkens_", {row["filter"] for row in SCOPE.RENDER_ROWS})
        self.assertIn("zircon_plugin_rendering_volumetric_fog_runtime", {row["package"] for row in SCOPE.RENDER_ROWS})
        self.assertIn("smoke_volumetric_fog_executes_a_product_frame", {row["filter"] for row in SCOPE.RENDER_ROWS})
        self.assertIn('ci-scope.py run --base "$env:CI_SCOPE_BASE" --head "$env:CI_SCOPE_HEAD" --recipe "$env:CI_RECIPE"', smoke)
        self.assertIn('command += ["-RequireMatchingTests"]', SCOPE_PATH.read_text(encoding="utf-8"))
        self.assertNotIn("--exact", smoke)
        self.assertIn(
            "ZIRCON_RENDERING_SMOKE_OUTPUT_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\render-feature-smoke-${{ strategy.job-index || 0 }}\\target\\product-frame",
            smoke,
        )
        self.assertIn(
            "fn smoke_volumetric_fog_executes_a_product_frame(",
            volumetric_tests,
        )
        self.assertNotRegex(
            volumetric_tests,
            r'(?s)#\[ignore[^\]]*\]\s*fn smoke_volumetric_fog_executes_a_product_frame\(',
        )
        self.assertIn("let (volumetric_frame, volumetric_stats) = render_frame(", volumetric_tests)
        self.assertIn(".submit_frame_extract(viewport, extract)", volumetric_tests)
        self.assertIn(".capture_frame(viewport)", volumetric_tests)
        self.assertIn("Dist recipes fail closed", job_body(self.workflow, "rust"))
        value = SCOPE.recipe("dist-fixture", "fixture", group="dist", features=("dist",), verb="build")
        with mock.patch.object(SCOPE.subprocess, "run") as run:
            with self.assertRaisesRegex(SCOPE.ScopeError, "Native artifact acceptance pending"):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id="dist-fixture")
            run.assert_not_called()
        self.assertIn("python -m tools.export plugin validate --all", job_body(self.workflow, "plugin-standalone-validate"))


    def test_scoped_workflow_consumes_only_recomputed_recipe_ids(self) -> None:
        self.assertNotRegex(self.workflow, r"cargo (?:build|test|check|clippy)[^\n]*(?:--workspace|--all-targets|--all-features)")
        for name, group in (
            ("runtime-profile-feature-matrix", "profile"),
            ("runtime-domain-feature-matrix", "domain"),
            ("plugin-standalone-dist", "dist"),
            ("render-feature-smoke", "render"),
            ("export-platform-contract", "export"),
        ):
            body = job_body(self.workflow, name)
            self.assertIn(f"fromJSON(needs.runtime-profile-feature-matrix-plan.outputs.{group})", body)
            self.assertIn('CI_RECIPE: ${{ matrix.recipe }}', body)
            self.assertIn('--recipe "$env:CI_RECIPE"', body)
            self.assertNotRegex(body, r"matrix\.(?:args|filter|package|manifest)")
        source = SCOPE_PATH.read_text(encoding="utf-8")
        for contract in ('module.parse_cargo(args, enforce_policy=True)',
                         '["--", "-D", "warnings"]', 'execute_single = admission(lint)',
                         'args += ["--target", value["platform"]]',
                         'make_plan(ROOT, args.base, args.head, args.select)'):
            self.assertIn(contract, source)

    def scope_fixture(self):
        packages = {
            "zircon_runtime": dict(name="zircon_runtime", directory="zircon_runtime", dependencies=["zr_math"]),
            "zr_math": dict(name="zr_math", directory="zircon_runtime/crates/zr_math", dependencies=[]),
            "zircon_runtime_interface": dict(name="zircon_runtime_interface", directory="zircon_runtime_interface", dependencies=["zr_math"]),
            "unused": dict(name="unused", directory="unused", dependencies=[]),
        }
        recipes = {
            "runtime-scene": SCOPE.recipe("runtime-scene", "zircon_runtime", features=("native-host",), selector="scene::"),
            "domain-native-host": SCOPE.recipe("domain-native-host", "zircon_runtime", group="domain", features=("native-host",), verb="check"),
            "leaf-zr_math": SCOPE.recipe("leaf-zr_math", "zr_math", selector="tests::"),
            "leaf-zircon_runtime_interface": SCOPE.recipe("leaf-zircon_runtime_interface", "zircon_runtime_interface", selector="tests::"),
            "leaf-unused": SCOPE.recipe("leaf-unused", "unused", selector="tests::"),
        }
        return packages, recipes

    def test_scope_leaf_uses_longest_owner_and_reviewed_reverse_consumers(self) -> None:
        packages, recipes = self.scope_fixture()
        values = SCOPE.plan_paths(["zircon_runtime/crates/zr_math/src/vector.rs"], packages, recipes)
        self.assertEqual({r["id"] for r in values},
                         {"leaf-zr_math", "runtime-scene", "leaf-zircon_runtime_interface"})

    def test_scope_unknown_and_shared_controls_cannot_be_waived(self) -> None:
        packages, recipes = self.scope_fixture()
        for path in ("unknown/code.rs", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                     "zircon_runtime/src/lib.rs", "zircon_runtime/build.rs"):
            with self.subTest(path=path):
                with self.assertRaisesRegex(SCOPE.ScopeError, "Uncovered impact"):
                    SCOPE.plan_paths([path], packages, recipes, ("runtime-scene",))

    def test_scope_uncovered_reverse_consumer_is_a_blocker(self) -> None:
        packages, recipes = self.scope_fixture()
        packages["unused"]["dependencies"] = ["zr_math"]
        with self.assertRaisesRegex(SCOPE.ScopeError, "uncovered reverse consumer unused"):
            SCOPE.plan_paths(["zircon_runtime/crates/zr_math/src/lib.rs"],
                             packages, recipes, ("leaf-unused",))

    def test_scope_known_docs_do_not_compile_and_explicit_ids_are_not_raw_argv(self) -> None:
        packages, recipes = self.scope_fixture()
        self.assertEqual(SCOPE.plan_paths(["docs/testing/example.md"], packages, recipes), [])
        for selection in ("--workspace", "runtime-scene --all-features", "unknown"):
            with self.assertRaisesRegex(SCOPE.ScopeError, "Unknown recipe"):
                SCOPE.plan_paths(["docs/testing/example.md"], packages, recipes, (selection,))

    def test_scope_rename_preserves_both_owners_and_requires_real_base(self) -> None:
        with mock.patch.object(SCOPE, "git", return_value=b"R100\0old/src/a.rs\0new/src/a.rs\0"):
            self.assertEqual(SCOPE.changed_paths(REPO_ROOT, "a" * 40, "b" * 40),
                             ["new/src/a.rs", "old/src/a.rs"])
        with mock.patch.object(SCOPE, "git") as git:
            for base in (None, "", "0" * 40, "HEAD~1", "a" * 39):
                with self.assertRaisesRegex(SCOPE.ScopeError, "real base"):
                    SCOPE.changed_paths(REPO_ROOT, base, "b" * 40)
            git.assert_not_called()
        with mock.patch.object(SCOPE, "git", return_value=b"M\0../escape.rs\0"):
            with self.assertRaisesRegex(SCOPE.ScopeError, "Noncanonical"):
                SCOPE.changed_paths(REPO_ROOT, "a" * 40, "b" * 40)

    def test_scope_rejects_all_package_and_excessive_recipe_selection(self) -> None:
        packages, recipes = self.scope_fixture()
        with self.assertRaisesRegex(SCOPE.ScopeError, "full package enumeration"):
            SCOPE.plan_paths([], packages, recipes, tuple(recipes))
        excessive = {f"r{i}": SCOPE.recipe(f"r{i}", "zircon_runtime", selector="scene::") for i in range(33)}
        with self.assertRaisesRegex(SCOPE.ScopeError, "bounded CI plan"):
            SCOPE.plan_paths([], packages, excessive, tuple(excessive))

    def test_scope_command_admission_keeps_defaults_disabled_target_and_filter(self) -> None:
        value = {**SCOPE.recipe("example", "zircon_runtime", features=("native-host",), selector="scene::ecs::"),
                 "manifest": "Cargo.toml"}
        args = SCOPE.admission(value)
        self.assertEqual(args[:2], ["test", "-p"])
        for item in ("zircon_runtime", "--no-default-features", "--lib", "--locked", "native-host",
                     "scene::ecs::", "--target", SCOPE.HOST):
            self.assertIn(item, args)
        for patch in ({"target": "all-targets"}, {"platform": "bad-target"},
                      {"profile": "release"}, {"selector": ""}, {"selector": "--workspace"},
                      {"features": ["--all-features"]}, {"package": "a --workspace"},
                      {"manifest": "../outside/Cargo.toml"}, {"argv": ["--workspace"]},
                      {"noDefaultFeatures": False}):
            with self.subTest(patch=patch):
                with self.assertRaises(SCOPE.ScopeError):
                    SCOPE.admission({**value, **patch})

    def test_scope_executor_lists_tests_then_lints_same_owner(self) -> None:
        value = {**SCOPE.recipe("example", "zircon_runtime", features=("native-host",), selector="scene::ecs::"),
                 "manifest": "Cargo.toml"}
        with mock.patch.object(SCOPE.subprocess, "run") as run:
            SCOPE.execute(REPO_ROOT, {"recipes": [value]}, group="rust")
        self.assertEqual(run.call_count, 2)
        test = run.call_args_list[0].args[0]
        lint = run.call_args_list[1].args[0]
        self.assertIn("-RequireMatchingTests", test)
        self.assertEqual(test[test.index("cargo") + 1], "test")
        self.assertEqual(lint[lint.index("cargo") + 1], "clippy")
        self.assertEqual(lint[-3:], ["--", "-D", "warnings"])
        for args in (test, lint):
            self.assertIn("--no-default-features", args)
            self.assertEqual(args[args.index("-p") + 1], "zircon_runtime")
            self.assertEqual(args[args.index("--target") + 1], SCOPE.HOST)
            self.assertNotIn("--workspace", args)
        with mock.patch.object(SCOPE.subprocess, "run") as run:
            with self.assertRaisesRegex(SCOPE.ScopeError, "absent"):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id="example --workspace")
            run.assert_not_called()

    def test_scope_failed_nonzero_test_admission_never_proceeds_to_clippy(self) -> None:
        value = {**SCOPE.recipe("example", "zircon_runtime", selector="scene::"),
                 "manifest": "Cargo.toml"}
        with mock.patch.object(SCOPE.subprocess, "run", side_effect=subprocess.CalledProcessError(1, ["pwsh"])) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, group="rust",
                              history_archive_root=str(REPO_ROOT))
            self.assertEqual(run.call_count, 1)


    def test_scope_catalog_matches_current_manifests_and_target_features(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        self.assertEqual(packages["zr_math"]["directory"], "zircon_runtime/crates/zr_math")
        self.assertEqual(packages["zircon_runtime"]["manifest"], "Cargo.toml")
        self.assertEqual(packages["zircon_plugin_rendering_contact_shadow_runtime"]["manifest"],
                         "zircon_plugins/Cargo.toml")
        self.assertIn("zr_math", packages["zircon_runtime"]["dependencies"])
        self.assertEqual({r["selector"] for r in recipes.values() if r["group"] == "miri"},
                         set(SCOPE.MIRI_FILTERS))
        self.assertEqual({r["id"] for r in recipes.values() if r["group"] == "render"},
                         {"render-" + row["feature"] for row in SCOPE.RENDER_ROWS})
        for value in recipes.values():
            self.assertEqual(value["target"], "lib")
            self.assertEqual(value["profile"], "dev")
            self.assertLessEqual(set(value["features"]), packages[value["package"]]["data"].get("features", {}).keys())

    def test_scope_ecs_selects_all_three_miri_contracts_and_runtime_domain(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        chosen = SCOPE.plan_paths(["zircon_runtime/src/scene/ecs/query/mod.rs"], packages, recipes)
        self.assertEqual({r["id"] for r in chosen},
                         {"runtime-scene", "domain-native-host", "miri-0", "miri-1", "miri-2"})
        chosen = SCOPE.plan_paths([], packages, recipes, ("miri-1",))
        self.assertEqual({r["id"] for r in chosen}, {"miri-0", "miri-1", "miri-2"})



    def test_export_policy_registry_preserves_eight_selectable_native_host_contracts(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        expected = {"windows", "linux", "macos", "android", "ios", "web_gpu", "wasm", "headless"}
        self.assertEqual(set(SCOPE.EXPORT_PLATFORMS), expected)
        self.assertEqual({name for name in recipes if name.startswith("export-policy-")},
                         {"export-policy-" + platform for platform in expected})
        self.assertIsNone(recipes["export-policy"]["export_platform"])
        for platform in expected:
            value = recipes["export-policy-" + platform]
            self.assertEqual(value["export_platform"], platform)
            self.assertEqual(value["features"], ["native-host"])
            self.assertEqual(value["package"], "zircon_runtime")
            self.assertEqual(value["platform"], SCOPE.HOST)
            self.assertEqual(value["target"], "lib")
            self.assertEqual(value["profile"], "dev")
            self.assertEqual(value["selector"], SCOPE.PLATFORM_FILTER)
        source = (REPO_ROOT / "zircon_runtime/src/tests/plugins/export_build_plan_platform.rs").read_text(encoding="utf-8")
        self.assertIn("fn platform_target_policy_matches_host_resource_and_plugin_strategy()", source)
        self.assertIn("let all_cases = [", source)
        self.assertIn('std::env::var("ZR_EXPORT_CONTRACT_PLATFORM")', source)
        self.assertIn("requested_platform.is_some_and(|requested| requested != platform)", source)
        self.assertNotIn("ZR_EXPORT_CONTRACT_PLATFORM:", job_body(self.workflow, "export-platform-contract"))

    def test_export_policy_impact_runs_shared_all_cases_once_and_selection_is_finite(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        for path in SCOPE.EXPORT_POLICY_SOURCES:
            chosen = SCOPE.plan_paths([path], packages, recipes)
            self.assertEqual([value["id"] for value in chosen], ["export-policy"])
        chosen = SCOPE.plan_paths(["zircon_runtime/src/platform/mod.rs"], packages, recipes)
        self.assertFalse(any(value["group"] == "export" for value in chosen))
        for platform in SCOPE.EXPORT_PLATFORMS:
            selected = "export-policy-" + platform
            chosen = SCOPE.plan_paths(["docs/example.md"], packages, recipes, (selected,))
            self.assertEqual([value["id"] for value in chosen], [selected])
        with self.assertRaisesRegex(SCOPE.ScopeError, "Uncovered impact"):
            SCOPE.plan_paths(["unknown/export.rs"], packages, recipes, ("export-policy-linux",))

    def test_export_policy_executor_uses_trusted_policy_and_removes_inherited_all_case_filter(self) -> None:
        recipes = SCOPE.registry(SCOPE.catalog(REPO_ROOT))
        key = "ZR_EXPORT_CONTRACT_PLATFORM"
        with mock.patch.dict(SCOPE.os.environ, {key: "forged-input"}):
            for platform in SCOPE.EXPORT_PLATFORMS:
                value = recipes["export-policy-" + platform]
                with mock.patch.object(SCOPE.subprocess, "run") as run:
                    SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id=value["id"])
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.kwargs["env"][key], platform)
                self.assertIn("-RequireMatchingTests", run.call_args.args[0])
                self.assertIn(SCOPE.PLATFORM_FILTER, run.call_args.args[0])
                self.assertIn("--no-default-features", run.call_args.args[0])
            value = recipes["export-policy"]
            with mock.patch.object(SCOPE.subprocess, "run") as run:
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id=value["id"])
            self.assertNotIn(key, run.call_args.kwargs["env"])
            self.assertEqual(SCOPE.os.environ[key], "forged-input")

    def test_export_policy_invalid_or_unselected_input_cannot_execute(self) -> None:
        recipes = SCOPE.registry(SCOPE.catalog(REPO_ROOT))
        value = recipes["export-policy-linux"]
        for policy in ("Linux", "--all", "windows; cargo test", ""):
            with mock.patch.object(SCOPE.subprocess, "run") as run:
                with self.assertRaisesRegex(SCOPE.ScopeError, "Unknown export policy"):
                    SCOPE.execute(REPO_ROOT, {"recipes": [{**value, "export_platform": policy}]},
                                  recipe_id=value["id"])
                run.assert_not_called()
        with mock.patch.object(SCOPE.subprocess, "run") as run:
            with self.assertRaisesRegex(SCOPE.ScopeError, "absent"):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id="export-policy-windows")
            run.assert_not_called()



    def test_framework_contract_owners_select_real_cfg_and_never_native_host_fallback(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        framework = (REPO_ROOT / "zircon_runtime/src/core/framework/mod.rs").read_text(encoding="utf-8")
        for module, filename in (("ai", "manager.rs"), ("net", "http.rs"),
                                 ("physics", "settings.rs"), ("sound", "output.rs")):
            path = f"zircon_runtime/src/core/framework/{module}/{filename}"
            self.assertTrue((REPO_ROOT / path).is_file(), path)
            self.assertIn(f'#[cfg(feature = "{module}-contracts")]\n#[path = "{module}/mod.rs"]', framework)
            chosen = SCOPE.plan_paths([path], packages, recipes)
            self.assertEqual({value["id"] for value in chosen},
                             {f"runtime-core-framework-{module}", f"domain-{module}-contracts"})
            owner = next(value for value in chosen if value["group"] == "rust")
            self.assertEqual(owner["features"], [module + "-contracts"])
            self.assertEqual(next(value for value in chosen if value["group"] == "domain")["features"],
                             [module + "-contracts"])
            if module == "ai":
                self.assertEqual(owner["verb"], "check")
                self.assertEqual(owner["kind"], "compile-check")
                self.assertIsNone(owner["selector"])
                self.assertIn("no mounted unit tests", owner["reason"])
                ai_mod = (REPO_ROOT / "zircon_runtime/src/core/framework/ai/mod.rs").read_text(encoding="utf-8")
                self.assertNotIn("mod tests;", ai_mod)
            else:
                self.assertEqual(owner["verb"], "test")
                self.assertEqual(owner["kind"], "owner-regression")
                self.assertEqual(owner["selector"], f"core::framework::{module}::")
                mounted = REPO_ROOT / f"zircon_runtime/src/core/framework/{module}/mod.rs"
                self.assertIn('#[path = "tests/cases.rs"]', mounted.read_text(encoding="utf-8"))
                self.assertIn("#[test]", mounted.with_name("tests").joinpath("cases.rs").read_text(encoding="utf-8"))
            self.assertFalse(any(value["id"] == "portable-core" for value in chosen))

    def test_ai_compile_check_does_not_claim_an_owner_test_executed(self) -> None:
        recipes = SCOPE.registry(SCOPE.catalog(REPO_ROOT))
        value = recipes["runtime-core-framework-ai"]
        with mock.patch.object(SCOPE.subprocess, "run") as run, \
                mock.patch.object(SCOPE, "admission", wraps=SCOPE.admission) as admit:
            SCOPE.execute(REPO_ROOT, {"recipes": [value]}, group="rust")
        self.assertEqual(run.call_count, 2)
        self.assertEqual(admit.call_count, 2)
        commands = [call.args[0] for call in run.call_args_list]
        check, lint = commands
        self.assertEqual(check[check.index("cargo") + 1], "check")
        self.assertEqual(lint[lint.index("cargo") + 1], "clippy")
        self.assertEqual(lint[:-3], ["clippy" if index == check.index("cargo") + 1 else arg
                                     for index, arg in enumerate(check)])
        self.assertEqual(lint[-3:], ["--", "-D", "warnings"])
        for command in commands:
            self.assertNotIn("-RequireMatchingTests", command)
            self.assertEqual(command[command.index("--features") + 1], "ai-contracts")
            self.assertNotIn("core::", command)
            self.assertNotIn("native-host", command)
            self.assertIn("--no-default-features", command)
            self.assertIn("--lib", command)
        declarations = [call.args[0] for call in admit.call_args_list]
        self.assertEqual(declarations[0], value)
        self.assertEqual(declarations[1], {**value, "verb": "clippy", "selector": None,
                                          "kind": "compile-check"})
        self.assertEqual(value["verb"], "check")
        self.assertEqual(value["kind"], "compile-check")
        with self.assertRaisesRegex(SCOPE.ScopeError, "Recipe kind"):
            SCOPE.admission({**value, "kind": "owner-regression"})

    def test_failed_owner_compile_check_never_proceeds_to_clippy(self) -> None:
        recipes = SCOPE.registry(SCOPE.catalog(REPO_ROOT))
        value = recipes["runtime-core-framework-ai"]
        failure = subprocess.CalledProcessError(101, ["cargo", "check"])
        with mock.patch.object(SCOPE.subprocess, "run", side_effect=failure) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, group="rust")
        self.assertEqual(run.call_count, 1)
        command = run.call_args.args[0]
        self.assertEqual(command[command.index("cargo") + 1], "check")
        self.assertNotIn("-RequireMatchingTests", command)


    def test_shared_feature_and_diagnostic_backend_boundaries_fail_closed(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        actual_paths = (
            "core/mod.rs", "core/framework/mod.rs", "core/framework/tests/cases.rs",
            "core/manager/mod.rs", "core/manager/resolver.rs", "core/manager/service_names.rs",
            "core/manager/tests/cases.rs", "core/resource/mod.rs", "core/runtime/mod.rs",
            "core/runtime/diagnostics/mod.rs", "core/runtime/diagnostics/profiling/mod.rs",
            "core/runtime/diagnostics/profiling/export.rs", "core/runtime/diagnostics/profiling/tracy.rs",
            "core/runtime/render_stats_store/advanced_provider.rs",
        )
        for local in actual_paths:
            path = "zircon_runtime/src/" + local
            self.assertTrue((REPO_ROOT / path).is_file(), path)
            with self.subTest(path=path):
                with self.assertRaisesRegex(SCOPE.ScopeError, "cfg/backend boundary"):
                    SCOPE.plan_paths([path], packages, recipes,
                                     ("runtime-core", "domain-graphics", "domain-sound-contracts"))
        profiling = (REPO_ROOT / "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs").read_text(encoding="utf-8")
        self.assertIn('#[cfg(feature = "profiling-tracy")]', profiling)
        self.assertIn('cfg!(feature = "profiling-chrome")', profiling)
        self.assertIn('cfg!(feature = "profiling")', profiling)

    def test_ungated_framework_audio_animation_use_core_min_not_optional_domain_names(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        for module, filename in (("audio", "channel_layout.rs"), ("animation", "avatar_mask.rs")):
            path = f"zircon_runtime/src/core/framework/{module}/{filename}"
            self.assertTrue((REPO_ROOT / path).is_file())
            chosen = SCOPE.plan_paths([path], packages, recipes)
            owner = next(value for value in chosen if value["group"] == "rust")
            self.assertEqual(owner["features"], ["core-min"])
            self.assertEqual(owner["selector"], f"core::framework::{module}::")
            self.assertEqual({value["id"] for value in chosen},
                             {f"runtime-core-framework-{module}", "domain-core-min", "portable-core"})

    def test_fifteen_render_source_owners_select_only_their_frame_and_required_wrapper(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        rows = {row["feature"]: row for row in SCOPE.RENDER_ROWS}
        self.assertEqual(set(SCOPE.RENDER_SOURCE_OWNERS), set(rows))
        self.assertEqual(len(SCOPE.RENDER_SOURCE_OWNERS), 15)
        for feature, owner_package in SCOPE.RENDER_SOURCE_OWNERS.items():
            directory = "subsurface" if feature == "subsurface_scattering" else feature
            self.assertEqual(packages[owner_package]["directory"],
                             f"zircon_plugins/rendering/features/{directory}/runtime")
            path = packages[owner_package]["directory"] + "/src/capability.rs"
            self.assertTrue((REPO_ROOT / path).is_file(), path)
            chosen = SCOPE.plan_paths([path], packages, recipes)
            identifiers = {value["id"] for value in chosen}
            expected = {"plugin-" + owner_package, "render-" + feature}
            if feature in SCOPE.RENDER_WRAPPERS:
                expected.add("render-wrapper-" + feature)
                wrapper = next(value for value in chosen if value["group"] == "render-wrapper")
                self.assertEqual((wrapper["package"], wrapper["selector"]), SCOPE.RENDER_WRAPPERS[feature])
            self.assertEqual(identifiers, expected, feature)
            frame = next(value for value in chosen if value["group"] == "render")
            self.assertEqual(frame["package"], rows[feature]["package"])
            self.assertEqual(frame["selector"], rows[feature]["filter"])
            if rows[feature]["package"] == "zircon_runtime":
                self.assertNotEqual(owner_package, frame["package"])

    def test_unrelated_private_plugin_does_not_select_render_or_portable_contracts(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        path = "zircon_plugins/sound/runtime/src/module.rs"
        self.assertTrue((REPO_ROOT / path).is_file())
        chosen = SCOPE.plan_paths([path], packages, recipes)
        self.assertEqual([value["id"] for value in chosen], ["plugin-zircon_plugin_sound_runtime"])

    def test_portable_impact_uses_actual_ungated_production_inputs(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        paths = (
            "core/framework/audio/channel_layout.rs", "core/framework/time/product_policy.rs",
            "core/framework/animation/avatar_mask.rs", "core/math/mod.rs",
            "core/runtime/clock_source.rs", "core/runtime/events.rs", "engine_module/engine_module.rs",
        )
        for local in paths:
            path = "zircon_runtime/src/" + local
            self.assertTrue((REPO_ROOT / path).is_file(), path)
            chosen = SCOPE.plan_paths([path], packages, recipes)
            self.assertIn("portable-core", {value["id"] for value in chosen}, path)
            portable = next(value for value in chosen if value["id"] == "portable-core")
            self.assertEqual(portable["features"], ["core-min"])
            self.assertEqual(portable["platform"], "wasm32-unknown-unknown")
            self.assertEqual(portable["verb"], "check")
            self.assertEqual(portable["kind"], "compile-check")
            self.assertTrue(any(value["group"] == "rust" for value in chosen), path)
        lib = (REPO_ROOT / "zircon_runtime/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn('#[path = "core/mod.rs"]\npub mod core;', lib)
        self.assertIn('#[path = "engine_module/mod.rs"]\npub mod engine_module;', lib)
        core = (REPO_ROOT / "zircon_runtime/src/core/mod.rs").read_text(encoding="utf-8")
        self.assertIn('#[path = "framework/mod.rs"]\npub mod framework;', core)
        self.assertIn('#[path = "runtime/mod.rs"]\npub mod runtime;', core)

    def test_host_only_and_test_inputs_do_not_become_portable_production_coverage(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        for local in ("platform/mod.rs", "scene/ecs/mod.rs",
                      "core/framework/audio/tests/cases.rs", "graphics/mod.rs"):
            path = "zircon_runtime/src/" + local
            self.assertTrue((REPO_ROOT / path).is_file(), path)
            chosen = SCOPE.plan_paths([path], packages, recipes)
            self.assertNotIn("portable-core", {value["id"] for value in chosen}, path)




    def test_runtime_render_commands_use_native_assets_through_real_admission(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        expected = {"irradiance_volumes", "light_cookies", "oit", "planar_reflections",
                    "post_process", "reflection_probes", "ssao", "subsurface_scattering"}
        runtime_rows = [row for row in SCOPE.RENDER_ROWS if row["package"] == "zircon_runtime"]
        self.assertEqual({row["feature"] for row in runtime_rows}, expected)
        for row in runtime_rows:
            with self.subTest(feature=row["feature"]):
                owner = packages[SCOPE.RENDER_SOURCE_OWNERS[row["feature"]]]
                source = owner["directory"] + "/src/capability.rs"
                self.assertTrue((REPO_ROOT / source).is_file())
                chosen = SCOPE.plan_paths([source], packages, recipes)
                frame = next(value for value in chosen if value["id"] == "render-" + row["feature"])
                with mock.patch.object(SCOPE.subprocess, "run") as native:
                    SCOPE.execute(REPO_ROOT, {"recipes": chosen}, recipe_id=frame["id"])
                self.assertEqual(native.call_count, 1 + len(frame.get("companions", ())))
                command = native.call_args_list[0].args[0]
                self.assertEqual(command[command.index("cargo") + 1], "test")
                self.assertEqual(command[command.index("--manifest-path") + 1], "Cargo.toml")
                self.assertEqual(command[command.index("-p") + 1], "zircon_runtime")
                self.assertEqual(command[command.index("--target") + 1], SCOPE.HOST)
                self.assertEqual(set(command[command.index("--features") + 1].split(",")),
                                 {"native-host", "graphics"})
                for required in ("--lib", "--no-default-features", "--locked", "-RequireMatchingTests"):
                    self.assertIn(required, command)
                self.assertIn(row["filter"], command)
                for companion in native.call_args_list[1:]:
                    argv = companion.args[0]
                    self.assertNotEqual(argv[argv.index("-p") + 1], "zircon_runtime")
                    self.assertNotIn("--features", argv)
                    self.assertIn("--no-default-features", argv)
                    self.assertIn("--locked", argv)

    def test_graphics_owner_tests_and_clippy_keep_portable_domain_separate(self) -> None:
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        for source, identifier in (
                ("zircon_runtime/src/graphics/tests/render_product_advanced_lighting.rs", "runtime-graphics"),
                ("zircon_runtime/src/render_graph/mod.rs", "runtime-render_graph")):
            with self.subTest(source=source):
                self.assertTrue((REPO_ROOT / source).is_file())
                chosen = SCOPE.plan_paths([source], packages, recipes)
                owner = next(value for value in chosen if value["id"] == identifier)
                domain = next(value for value in chosen if value["id"] == "domain-graphics")
                self.assertEqual(domain["verb"], "check")
                self.assertEqual(domain["features"], ["graphics"])
                self.assertEqual(recipes["portable-core"]["features"], ["core-min"])
                with mock.patch.object(SCOPE.subprocess, "run") as native:
                    SCOPE.execute(REPO_ROOT, {"recipes": chosen}, group="rust")
                self.assertEqual(native.call_count, 2)
                self.assertEqual(native.call_args_list[0].args[0][
                    native.call_args_list[0].args[0].index("cargo") + 1], "test")
                for call in native.call_args_list:
                    command = call.args[0]
                    self.assertEqual(command[command.index("-p") + 1], owner["package"])
                    self.assertEqual(set(command[command.index("--features") + 1].split(",")),
                                     {"native-host", "graphics"})
                    self.assertIn("--no-default-features", command)
                    self.assertIn("--locked", command)
                lint = native.call_args_list[1].args[0]
                self.assertEqual(lint[lint.index("cargo") + 1], "clippy")
                self.assertEqual(lint[-3:], ["--", "-D", "warnings"])

    def test_real_graphics_test_imports_require_native_asset_mount(self) -> None:
        manifest = SCOPE.tomllib.loads((REPO_ROOT / "zircon_runtime/Cargo.toml").read_text(encoding="utf-8"))

        def feature_closure(features):
            selected, pending = set(), list(features)
            while pending:
                feature = pending.pop()
                if feature in selected:
                    continue
                selected.add(feature)
                pending.extend(item for item in manifest["features"].get(feature, ())
                               if item in manifest["features"])
            return selected

        runtime_lib = (REPO_ROOT / "zircon_runtime/src/lib.rs").read_text(encoding="utf-8")
        self.assertRegex(runtime_lib,
                         r'(?s)#\[cfg\(feature = "native-host"\)\]\s*#\[path = "asset/mod.rs"\]\s*pub mod asset;')
        self.assertRegex(runtime_lib,
                         r'(?s)#\[cfg\(all\(feature = "asset-types", not\(feature = "native-host"\)\)\)\]\s*'
                         r'#\[path = "asset/portable.rs"\]\s*pub mod asset;')
        native_asset = (REPO_ROOT / "zircon_runtime/src/asset/mod.rs").read_text(encoding="utf-8")
        portable_asset = (REPO_ROOT / "zircon_runtime/src/asset/portable.rs").read_text(encoding="utf-8")
        self.assertRegex(native_asset, r"pub mod pipeline;")
        self.assertNotRegex(portable_asset, r"\bmod pipeline\s*[;{]")
        graphics = (REPO_ROOT / "zircon_runtime/src/graphics/mod.rs").read_text(encoding="utf-8")
        self.assertRegex(graphics, r'(?s)#\[cfg\(test\)\]\s*#\[path = "tests/mod.rs"\]\s*mod tests;')
        owners = (
            "render_product_advanced_lighting/af_m2.rs", "render_product_advanced_lighting.rs",
            "render_product_planar_reflection.rs", "render_product_post_process.rs",
            "project_scenes/reflection_probe_product.rs", "project_render/render_quality.rs",
            "render_product_subsurface.rs",
        )
        for owner in owners:
            text = (REPO_ROOT / "zircon_runtime/src/graphics/tests" / owner).read_text(encoding="utf-8")
            self.assertRegex(text, r"\b(?:Project)?AssetManager\b", owner)
        project_backed_test = (REPO_ROOT / "zircon_runtime/src/graphics/tests" /
                               "render_product_advanced_lighting.rs").read_text(encoding="utf-8")
        self.assertIn("crate::asset::pipeline::manager::ProjectAssetManager", project_backed_test)
        project_quality = (REPO_ROOT / "zircon_runtime/src/graphics/tests/project_render/render_quality.rs").read_text(
            encoding="utf-8")
        self.assertIn("crate::asset::project::ProjectManager", project_quality)
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        for value in recipes.values():
            if value["group"] == "render" and value["package"] == "zircon_runtime":
                command = SCOPE.admission(value)
                closure = feature_closure(command[command.index("--features") + 1].split(","))
                self.assertIn("native-host", closure)
                self.assertIn("graphics", closure)
                # Regression control: the former admitted argv selects portable assets.
                former = SCOPE.admission({**value, "features": ["graphics"]})
                former_closure = feature_closure(former[former.index("--features") + 1].split(","))
                self.assertIn("asset-types", former_closure)
                self.assertNotIn("native-host", former_closure)

class FixedHistoryCiInputTests(unittest.TestCase):
    def test_explicit_root_reaches_execute_subprocesses_and_companions(self):
        value = SCOPE.recipe("history-routing", "zircon_runtime", features=("native-host",),
                             selector="tests::runtime_absorption::structure_convention")
        companion = SCOPE.recipe("history-companion", "zircon_runtime", group="export",
                                 features=("native-host",), selector=SCOPE.PLATFORM_FILTER)
        value["manifest"] = companion["manifest"] = "Cargo.toml"
        companion["export_platform"] = None
        value["companions"] = [companion["id"]]
        with mock.patch.object(SCOPE, "admission", return_value=["test", "--lib"]), \
                mock.patch.object(SCOPE.subprocess, "run") as native:
            SCOPE.execute(REPO_ROOT, {"recipes": [value, companion]}, group="rust",
                          history_archive_root=str(REPO_ROOT))
        self.assertEqual(native.call_count, 3)
        self.assertEqual(native.call_args_list[0].kwargs["env"]["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"], str(REPO_ROOT.resolve()))
        for call in native.call_args_list[1:]:
            self.assertNotIn("ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT", call.kwargs["env"])
        self.assertNotIn("ZR_EXPORT_CONTRACT_PLATFORM", native.call_args_list[1].kwargs["env"])

    def test_unconfigured_archive_has_no_ambient_checkout_fallback(self):
        env = SCOPE.history_archive_environment(None, {"ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT": str(REPO_ROOT), "PATH": "trusted"})
        self.assertEqual(env, {"PATH": "trusted"})
        self.assertIn("CI_DOCS_HISTORY_ARCHIVE_ROOT: ${{ vars.ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT }}",
                      WORKFLOW_PATH.read_text(encoding="utf-8"))

    def test_invalid_archive_roots_are_rejected_without_native_launch(self):
        for value in ("", "relative/archive", "bad\nroot"):
            with self.subTest(root=value), self.assertRaises((ValueError, SCOPE.ScopeError)):
                SCOPE.history_archive_environment(value, {}, required=True)
        with mock.patch.object(SCOPE.os, "scandir", side_effect=PermissionError("unreadable")):
            with self.assertRaises(PermissionError):
                SCOPE.history_archive_environment(str(REPO_ROOT), {}, required=True)
        with self.assertRaises((ValueError, SCOPE.ScopeError)):
            SCOPE.history_archive_environment(str(SCOPE_PATH), {}, required=True)

    def test_run_recomputes_plan_and_keeps_explicit_archive_argument(self):
        planned = {"recipes": []}
        with mock.patch.object(SCOPE, "make_plan", return_value=planned) as plan, \
                mock.patch.object(SCOPE, "execute") as execute:
            self.assertEqual(SCOPE.main(["run", "--base", "base", "--head", "head", "--recipe", "history",
                                        "--history-archive-root", str(REPO_ROOT)]), 0)
        plan.assert_called_once_with(SCOPE.ROOT, "base", "head", [])
        execute.assert_called_once_with(SCOPE.ROOT, planned, "history", None, history_archive_root=str(REPO_ROOT))


    def test_unused_archive_configuration_cannot_block_unrelated_recipes(self):
        packages = SCOPE.catalog(REPO_ROOT)
        recipes = SCOPE.registry(packages)
        self.assertTrue(SCOPE.recipe_consumes_history(recipes["runtime-tests-contracts-structure"]))
        unrelated = [recipes["portable-core"], recipes["export-policy"], recipes["render-light_cookies"]]
        planned, expected = {}, []

        def include_with_companions(value):
            planned[value["id"]] = value
            expected.append(value["id"])
            for identifier in value.get("companions", ()):
                include_with_companions(recipes[identifier])

        for value in unrelated:
            include_with_companions(value)
        self.assertTrue(any(value.get("companions") for value in unrelated))
        complete_plan = {"recipes": list(planned.values())}
        with mock.patch.dict(SCOPE.os.environ, {"ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT": "ambient-archive",
                                               "CI_DOCS_HISTORY_ARCHIVE_ROOT": "ambient-configuration"}), \
                mock.patch.object(SCOPE, "admission", return_value=["test", "--lib"]) as admitted, \
                mock.patch.object(SCOPE.subprocess, "run") as native:
            for value in unrelated:
                self.assertFalse(SCOPE.recipe_consumes_history(value))
                SCOPE.execute(REPO_ROOT, complete_plan, recipe_id=value["id"],
                              history_archive_root="relative/unused-and-invalid")
        self.assertEqual([call.args[0]["id"] for call in admitted.call_args_list], expected)
        self.assertEqual(native.call_count, len(expected))
        for call in native.call_args_list:
            self.assertNotIn("ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT", call.kwargs["env"])
            self.assertNotIn("CI_DOCS_HISTORY_ARCHIVE_ROOT", call.kwargs["env"])

    def test_real_history_recipe_rejects_missing_input_before_native_launch(self):
        value = SCOPE.recipe("history-routing", "zircon_runtime", features=("native-host",),
                             selector="structure_convention")
        value["manifest"] = "Cargo.toml"
        with mock.patch.object(SCOPE, "admission", return_value=["test", "--lib"]), \
                mock.patch.object(SCOPE.subprocess, "run") as native:
            with self.assertRaises(SCOPE.ScopeError):
                SCOPE.execute(REPO_ROOT, {"recipes": [value]}, recipe_id=value["id"])
        native.assert_not_called()


    def test_finite_history_consumers_keep_current_and_unrelated_cases_separate(self):
        template = SCOPE.registry(SCOPE.catalog(REPO_ROOT))["runtime-tests-contracts-structure"]
        for selector in ('tests::runtime_absorption::ui_architecture::mirror_docs::runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts', "runtime_14_module_family_mirror_docs_match_structure_audit_counts", "tests::runtime_absorption::", ""):
            self.assertTrue(SCOPE.recipe_consumes_history({**template, "selector": selector}), selector)
        for selector in ('tests::runtime_absorption::ui_architecture::architecture_boundaries::runtime_09_ui_architecture_doc_records_current_boundaries', "scene::ecs::", "core::", "not_a_registered_history_case"):
            self.assertFalse(SCOPE.recipe_consumes_history({**template, "selector": selector}), selector)
        for verb in ("check", "build", "clippy"):
            self.assertFalse(SCOPE.recipe_consumes_history({**template, "verb": verb}))
        self.assertFalse(SCOPE.recipe_consumes_history({**template, "package": "zircon_plugin_sdk"}))


if __name__ == "__main__":
    unittest.main()
