import hashlib
import json
import shutil
import subprocess
import sys
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

from tools import zircon_build


REPO_ROOT = Path(__file__).resolve().parents[2]
PREWARM_EXE = Path(
    r"E:\cargo-targets\zircon-runtime-text-0630-clamp-contract\debug\zircon_shader_prewarm.exe"
)


class ZirconBuildShaderPrewarmWrapperOrchestrationTests(unittest.TestCase):
    def test_runtime_server_wrapper_uses_client_features_for_preview_binary(self):
        out_root = REPO_ROOT / "target" / "codex-plan08-wrapper-feature-contract"

        stdout = StringIO()
        with redirect_stdout(stdout):
            exit_code = zircon_build.main(
                [
                    "--targets",
                    "runtime",
                    "--out",
                    str(out_root),
                    "--mode",
                    "debug",
                    "--cargo",
                    "cargo-probe",
                    "--runtime-features",
                    "target-server,profiling",
                    "--dry-run",
                ]
            )

        self.assertEqual(0, exit_code)
        output = stdout.getvalue()
        self.assertIn(
            'cargo-probe build -p zircon_runtime --lib --no-default-features '
            '--features "target-server profiling"',
            output,
        )
        self.assertIn(
            'cargo-probe build -p zircon_app --bin zircon_runtime '
            '--no-default-features --features "target-client profiling"',
            output,
        )

    def test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu(self):
        if not PREWARM_EXE.exists():
            self.skipTest(f"shader prewarm executable is unavailable: {PREWARM_EXE}")

        out_root = REPO_ROOT / "target" / "codex-plan08-wrapper-orchestration-test"
        _safe_reset(out_root)
        fake_cargo = _write_fake_cargo(out_root, PREWARM_EXE)
        project_assets = _write_project_shader_assets(out_root / "project_assets")

        stdout = StringIO()
        with redirect_stdout(stdout):
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
                    "--cargo",
                    str(fake_cargo),
                    "--prewarm-shaders",
                    "--validate-wgpu-shaders",
                    "--shader-asset-root",
                    str(project_assets),
                    "--shader-quality-tier",
                    "medium",
                    "--shader-geometry-source",
                    "static",
                ]
            )

        self.assertEqual(0, exit_code)
        output = stdout.getvalue()
        self.assertIn("Zircon build plan", output)
        self.assertIn("shader WGPU module validation: enabled", output)
        self.assertIn("shader resource registry export:", output)

        log_records = _read_log_records(out_root / "fake_cargo" / "calls.jsonl")
        build_records = [record for record in log_records if record["event"] == "build"]
        run_records = [record for record in log_records if record["event"] == "prewarm_run"]
        self.assertEqual(2, len(build_records), log_records)
        self.assertEqual(1, len(run_records), log_records)

        prewarm_args = run_records[0]["prewarm_args"]
        self.assertIn("--validate-wgpu-modules", prewarm_args)
        self.assertIn("--export-resource-registry", prewarm_args)
        self.assertIn(str(project_assets), _flag_values(prewarm_args, "--asset-root"))
        self.assertIn(
            str(REPO_ROOT / "zircon_plugins" / "native_dynamic_fixture" / "assets"),
            _flag_values(prewarm_args, "--asset-root"),
        )

        engine_root = out_root / "ZirconEngine"
        report = json.loads(
            (engine_root / "cache" / "shader_variants_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(18, report["requested_count"])
        self.assertEqual(18, report["written_count"])
        self.assertEqual(0, report["failed_count"])
        self.assertEqual(18, report["wgpu_module_validation"]["requested_count"])
        self.assertEqual(18, report["wgpu_module_validation"]["validated_count"])
        self.assertEqual(0, report["wgpu_module_validation"]["failed_count"])

        registry = json.loads(
            (engine_root / "cache" / "shader_resource_records.json").read_text(
                encoding="utf-8"
            )
        )
        locators = {
            record["primary_locator"]: record["state"]
            for record in registry["resources"]
        }
        self.assertEqual("Ready", locators["res://project/shaders/project_shader"])
        self.assertEqual(
            "Ready",
            locators["package://native_dynamic_fixture/shaders/shader"],
        )


def _write_fake_cargo(out_root: Path, prewarm_exe: Path) -> Path:
    fake_root = out_root / "fake_cargo"
    fake_root.mkdir(parents=True, exist_ok=True)
    script = fake_root / "fake_cargo.py"
    script.write_text(
        _fake_cargo_script(prewarm_exe, REPO_ROOT, fake_root / "calls.jsonl"),
        encoding="utf-8",
    )
    command = fake_root / "fake_cargo.cmd"
    command.write_text(
        f'@echo off\r\n"{sys.executable}" "%~dp0fake_cargo.py" %*\r\n',
        encoding="utf-8",
    )
    return command


def _fake_cargo_script(prewarm_exe: Path, repo_root: Path, log_path: Path) -> str:
    return f"""import json
import os
import platform
import subprocess
import sys
from pathlib import Path

PREWARM_EXE = Path({str(prewarm_exe)!r})
REPO_ROOT = Path({str(repo_root)!r})
LOG_PATH = Path({str(log_path)!r})


def main():
    args = sys.argv[1:]
    if not args:
        return 2
    if args[0] == "build":
        target_dir = Path(args[args.index("--target-dir") + 1])
        artifact_dir = target_dir / profile_dir(args)
        artifact_dir.mkdir(parents=True, exist_ok=True)
        if "--lib" in args:
            artifact_name = runtime_library_name()
        else:
            artifact_name = executable_name(args[args.index("--bin") + 1])
        (artifact_dir / artifact_name).write_bytes(b"fake cargo artifact\\n")
        append_log({{"event": "build", "args": args, "artifact": str(artifact_dir / artifact_name)}})
        return 0
    if args[0] == "run":
        separator = args.index("--")
        prewarm_args = args[separator + 1 :]
        append_log({{"event": "prewarm_run", "args": args, "prewarm_args": prewarm_args}})
        return subprocess.run([str(PREWARM_EXE), *prewarm_args], cwd=REPO_ROOT).returncode
    append_log({{"event": "unsupported", "args": args}})
    return 2


def append_log(record):
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    with LOG_PATH.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record) + "\\n")


def profile_dir(args):
    if "--release" in args:
        return "release"
    if "--profile" in args:
        return args[args.index("--profile") + 1]
    return "debug"


def executable_name(stem):
    return f"{{stem}}.exe" if os.name == "nt" else stem


def runtime_library_name():
    if os.name == "nt":
        return "zircon_runtime.dll"
    if platform.system().lower() == "darwin":
        return "libzircon_runtime.dylib"
    return "libzircon_runtime.so"


if __name__ == "__main__":
    raise SystemExit(main())
"""


def _write_project_shader_assets(project_assets: Path) -> Path:
    shader_dir = project_assets / "shaders"
    shader_dir.mkdir(parents=True, exist_ok=True)
    source = "@fragment\nfn project_shader_fragment() {}\n"
    (shader_dir / "project_shader.wgsl").write_text(source, encoding="utf-8")
    source_hash = hashlib.sha256(source.encode("utf-8")).hexdigest()
    (shader_dir / "project_shader.wgsl.zmeta").write_text(
        "\n".join(
            [
                "format_version = 6",
                'uuid = "00000000-0000-0000-0000-000000000071"',
                'url = "res://project/shaders/project_shader"',
                'asset_kind = "Shader"',
                'unit = "single"',
                f'source_hash = "{source_hash}"',
                'preview_state = "ready"',
                'importer_id = "zircon.project.shader"',
                "importer_version = 1",
                'config_hash = "project-shader-wrapper-orchestration"',
                "dependencies = []",
                "",
            ]
        ),
        encoding="utf-8",
    )
    return project_assets


def _safe_reset(path: Path) -> None:
    resolved = path.resolve()
    target_root = (REPO_ROOT / "target").resolve()
    if resolved != target_root and target_root not in resolved.parents:
        raise RuntimeError(f"refusing to reset path outside target: {resolved}")
    shutil.rmtree(resolved, ignore_errors=True)


def _read_log_records(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def _flag_values(args: list[str], flag: str) -> list[str]:
    return [args[index + 1] for index, value in enumerate(args[:-1]) if value == flag]


if __name__ == "__main__":
    unittest.main()
