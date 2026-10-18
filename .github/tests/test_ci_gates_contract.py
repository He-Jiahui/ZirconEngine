from __future__ import annotations

import re
import subprocess
import unittest
from pathlib import Path, PureWindowsPath


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "ci.yml"


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

    def test_pull_requests_compile_the_editor_and_plugins_and_run_commands(self) -> None:
        self.assertRegex(self.workflow, r"(?m)^  pull_request:\s*$")
        gate = job_body(self.workflow, "editor-compile-gate")
        self.assertIn("name: editor-compile-gate", gate)
        self.assertIn("if: github.event_name == 'pull_request'", gate)
        self.assertIn("runs-on: windows-2022", gate)
        self.assertRegex(gate, r"(?m)^    timeout-minutes: [1-9][0-9]*$")
        self.assertNotIn("needs:", gate)
        self.assertNotIn("continue-on-error:", gate)
        self.assertNotIn("secrets.", gate)
        self.assertIn("persist-credentials: false", gate)
        self.assertIn("id: editor_check", gate)
        for name in ("script behavior bridge service admission", "editor commands and effective keymap", "tilemap duplicate layer admission"):
            step = re.search(rf"(?ms)^      - name: Test {name}\n(.*?)(?=^      - |\Z)", gate).group(1)
            self.assertIn("if: ${{ !cancelled() && steps.editor_check.outcome == 'success' }}", step)
        for command in (
            "cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked",
            "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked",
            "-RequireMatchingTests cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --test behavior_bridge --locked",
            "-RequireMatchingTests cargo test -p zircon_editor --lib core::commands --locked",
            "-RequireMatchingTests cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_tilemap_2d_editor --lib tilemap_paint_stroke_rejects_budget_and_duplicate_layer_identity --locked",
        ):
            self.assertRegex(gate, rf"(?m)^        run: ./\.github/scripts/invoke-cargo-msvc\.ps1 {re.escape(command)}$")
        self.assertIn("-StorageRoot 'D:\\cargo-targets\\zircon-local'", gate)
        self.assertLess(gate.index("prepare-cargo-roots.ps1"), gate.index("uses: dtolnay/rust-toolchain@"))
        preparation = re.search(r"(?ms)^      - name: Require physical managed Cargo volume\n(.*?)(?=^      - |\Z)", gate).group(1)
        self.assertIn("TEMP: ${{ runner.temp }}", preparation)
        self.assertIn("TMP: ${{ runner.temp }}", preparation)
        self.assertIn("-BootstrapTemp", preparation)
        for variable in ("CARGO_HOME", "RUSTUP_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "TEMP", "TMP"):
            self.assertRegex(gate, rf"(?m)^      {variable}: D:\\cargo-targets\\zircon-local\\ci\\runs\\.*github.run_id.*github.run_attempt.*e-0\\")

    def test_self_hosted_jobs_are_bounded_and_wait_for_availability(self) -> None:
        for path in (WORKFLOW_PATH, WORKFLOW_PATH.with_name("profile-feature-contract.yml")):
            source = path.read_text(encoding="utf-8")
            signal = job_body(source, "runner-availability")
            self.assertIn("runs-on: ubuntu-24.04", signal)
            self.assertIn("timeout-minutes: 3", signal)
            self.assertIn("if: github.event_name == 'push'", signal)
            self.assertIn("secrets.RUNNER_STATUS_TOKEN", signal)
            self.assertIn("check-runner-availability.py --repository", signal)
            for name in re.findall(r"(?m)^  ([a-z0-9-]+):$", source):
                body = job_body(source, name)
                if "runs-on: [self-hosted, Windows, X64]" in body:
                    self.assertRegex(body, r"(?m)^    timeout-minutes: [1-9][0-9]*$", name)
                    self.assertRegex(body, r"(?m)^    needs: .*runner-availability", name)

    def test_mvp_separates_compilation_focused_tests_and_staged_acceptance(self) -> None:
        source = WORKFLOW_PATH.with_name("mvp-editor-windows.yml").read_text(encoding="utf-8")
        compile_gate = job_body(source, "compile-gate")
        focused = job_body(source, "focused-tests")
        staged = job_body(source, "mvp-editor-windows")
        self.assertIn("needs: compile-gate", focused)
        self.assertIn("needs: focused-tests", staged)
        self.assertIn("--bin zircon_editor --no-default-features --features target-editor-host --locked", compile_gate)
        self.assertIn("--manifest-path zircon_plugins/Cargo.toml --workspace --locked", compile_gate)
        self.assertNotIn("cargo test", compile_gate)
        for selector in ("project_automation_", "renderable_empty_template_has_the_f2_camera_cube_and_sun_contract",
                         "template_creation_rebuilds_regenerable_asset_state_from_source_after_deletion",
                         "template_creation_recovers_a_corrupt_persisted_registry_from_source_metadata",
                         "render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down",
                         "editor_project_document_roundtrips_world_and_workspace",
                         "f4_project_authoring_survives_full_application_restart"):
            self.assertIn(selector, focused)
            self.assertNotIn(selector, staged)
        self.assertEqual(focused.count("test result: ok. 1 passed; 0 failed"), 6)
        for name in ("compile-gate", "focused-tests", "mvp-editor-windows"):
            body = job_body(source, name)
            self.assertIn("runs-on: windows-2022", body)
            self.assertIn("WGPU_BACKEND: dx12", body)
            self.assertIn("persist-credentials: false", body)
            self.assertIn("-UseMvpTargetCache -VerifyOnly", body)
            self.assertLess(body.index("uses: actions/checkout@"), body.index("$lockHash ="))
            self.assertLess(body.index("Require physical managed Cargo volume"), body.index("uses: actions/cache@"))
            self.assertLess(body.index("-VerifyOnly"), body.index("uses: dtolnay/rust-toolchain@"))
            self.assertIn("github.sha", body)
            self.assertIn("hashFiles('Cargo.lock', 'zircon_plugins/Cargo.lock', '.cargo/config.toml')", body)
            self.assertIn("if: always()", body)
        # Cached compiler files are never substituted for source-bound F5 evidence.
        for requirement in ("Build-MvpProductInputs.ps1", "Resolve-MvpProductInputManifest",
                            "Get-MvpBuildGateRegistrySnapshot", "RequireF5Evidence",
                            "-ProfileContractSummaryPath", "-WorkspaceSummaryPath"):
            self.assertIn(requirement, staged)
        self.assertIn("uses: actions/download-artifact@", staged)

    def test_hosted_cache_reuse_keeps_physical_and_source_identity_checks(self) -> None:
        helper = (REPO_ROOT / ".github/scripts/prepare-cargo-roots.ps1").read_text(encoding="utf-8")
        for required in ("[ValidateSet('D:\\cargo-targets\\zircon-local'", "Assert-NoReparseAncestors $target",
                         "$env:RUNNER_ENVIRONMENT -ne 'github-hosted'", "$env:ZR_CI_LOCK_HASH -notmatch '^[a-f0-9]{64}$'",
                         "$target -ne $expectedTarget", "-Recurse -Force -Attributes ReparsePoint",
                         "Restored MVP target cache contains a reparse point"):
            self.assertIn(required, helper)

    def test_cargo_homes_leave_room_for_the_pinned_dependency_checkout(self) -> None:
        # Longest blob path in zr_vm at 3c1352ecb6ddfab02752d46682cc376558d5f481.
        # libgit2 still applies MAX_PATH when Cargo checks out this dependency.
        dependency_path = (
            "git/checkouts/zr_vm-91d161ef2b98ea29/3c1352e/tests/language_server/"
            "test_lsp_numeric_loop_self_dependency_conditional_delta_negative_offset_"
            "zero_inclusive_subtractive_semantic_query.c"
        )
        count = 0
        for path in (REPO_ROOT / ".github/workflows").glob("*.yml"):
            for home in re.findall(r"(?m)^      CARGO_HOME: (.+)$", path.read_text(encoding="utf-8")):
                resolved = home.replace("${{ github.run_id }}", "9" * 20)
                resolved = resolved.replace("${{ github.run_attempt }}", "999")
                resolved = resolved.replace("${{ strategy.job-index || 0 }}", "999")
                self.assertNotIn("${{", resolved, path.name)
                checkout = PureWindowsPath(resolved) / dependency_path
                self.assertLessEqual(len(str(checkout)), 259, f"{path.name}: {checkout}")
                count += 1
        self.assertGreater(count, 0)

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

    def test_runner_images_and_node24_actions_use_reviewed_pins(self) -> None:
        # Checked against each pinned upstream action.yml, not inferred from tags.
        node24 = {
            "actions/checkout": "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09",
            "actions/setup-python": "ece7cb06caefa5fff74198d8649806c4678c61a1",
            "actions/upload-artifact": "b7c566a772e6b6bfb58ed0dc250532a479d7789f",
            "actions/download-artifact": "37930b1c2abaa49bbe596cd826c3c89aef350131",
            "actions/cache": "caa296126883cff596d87d8935842f9db880ef25",
            "actions/configure-pages": "45bfe0192ca1faeb007ade9deae92b16b8254a0d",
            "actions/deploy-pages": "368f82528645a54fb793d4d04e342629a3f51346",
        }
        for path in (REPO_ROOT / ".github/workflows").glob("*.yml"):
            source = path.read_text(encoding="utf-8")
            self.assertNotRegex(source, r"runs-on: (ubuntu|windows)-latest", path.name)
            for action, sha in re.findall(r"uses: ([^@\s]+)@([0-9a-f]{40})", source):
                if action in node24:
                    self.assertEqual(sha, node24[action], action)
                if action == "actions/upload-pages-artifact":
                    # Reviewed composite metadata pins Node-24 upload-artifact v7.
                    self.assertEqual(sha, "fc324d3547104276b827a68afc52ff2a11cc49c9")
                if action == "dtolnay/rust-toolchain":
                    # The old 1.94.1 branch hardcoded its toolchain and rejected
                    # inputs.toolchain, including the intended Miri nightly.
                    self.assertEqual(sha, "683b588c487a69d01f6c6ddf40f036ff682cc2c8")

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
            "invoke-cargo-msvc.ps1 cargo clippy --workspace --all-targets --locked -- -D warnings",
            rust,
        )
        self.assertIn("python -B -m unittest tools.tests.test_path_names -v", job_body(self.workflow, "source-contracts"))
        self.assertIn("python -B tools/audits/path_names.py --summary", job_body(self.workflow, "source-contracts"))

    def test_plugins_workspace_has_its_own_format_and_lint_gates(self) -> None:
        rust = job_body(self.workflow, "rust")
        self.assert_workspace_member_fmt_gate(rust, "zircon_plugins/Cargo.toml")
        self.assertIn(
            "invoke-cargo-msvc.ps1 cargo clippy --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked -- -D warnings",
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
        self.assertEqual(len(re.findall(r"(?m)^          - feature:", smoke)), 15)
        self.assertIn(
            "CARGO_TARGET_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\rs-${{ strategy.job-index || 0 }}\\target",
            smoke,
        )
        self.assertIn(
            "ZIRCON_RENDERING_SMOKE_OUTPUT_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\rs-${{ strategy.job-index || 0 }}\\target\\product-frame",
            smoke,
        )
        self.assertIn("-RequireMatchingTests cargo test", smoke)
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
        self.assertEqual(set(re.findall(r"(?m)^          - feature: ([a-z_]+)$", smoke)), set(expected))
        for feature, (package, selector, source_path) in expected.items():
            row_match = re.search(
                rf"(?ms)^          - feature: {re.escape(feature)}$\n(?P<row>.*?)(?=^          - feature:|^    steps:)",
                smoke,
            )
            self.assertIsNotNone(row_match, feature)
            row = row_match.group("row") if row_match else ""
            self.assertIn(f"package: {package}", row, feature)
            self.assertIn(f"filter: {selector}", row, feature)
            if package == "zircon_runtime":
                self.assertIn("args: --lib --no-default-features --features graphics", row, feature)
            else:
                self.assertIn("args: --lib", row, feature)
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
            self.assertIn(wrapper_selector, smoke)

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
        self.assertIn("contact_shadow_wgpu_product_capture_darkens_", smoke)
        self.assertIn("zircon_plugin_rendering_volumetric_fog_runtime", smoke)
        self.assertIn("smoke_volumetric_fog_executes_a_product_frame", smoke)
        self.assertIn("-RequireMatchingTests cargo test", smoke)
        self.assertNotIn("--exact", smoke)
        self.assertIn(
            "ZIRCON_RENDERING_SMOKE_OUTPUT_DIR: E:\\cargo-targets\\zircon-local\\ci\\runs\\${{ github.run_id }}-${{ github.run_attempt }}\\rs-${{ strategy.job-index || 0 }}\\target\\product-frame",
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
        self.assertIn(
            'cargo run --locked -p cargo-zircon -- plugin check --root . --artifact-root "$env:CARGO_TARGET_DIR\\debug"',
            job_body(self.workflow, "rust"),
        )


if __name__ == "__main__":
    unittest.main()
