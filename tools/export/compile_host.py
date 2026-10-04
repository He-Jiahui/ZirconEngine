"""CompileHost stage execution for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import subprocess
from pathlib import Path
from typing import Any, Sequence

from .command_plan import command_with_option
from .compile_host_plan import load_compile_host_plan
from .path_resolve import resolve_stage_optional_path
from .report_io import write_report_targets
from .subprocess_output import split_subprocess_output

try:
    from ..zircon_build_product_profile import (
        ProductBuildProfile,
        ProductBuildProfileError,
        load_product_build_profile,
    )
except ImportError:  # pragma: no cover - direct package/script invocation.
    try:
        from tools.build.zircon_build_product_profile import (
            ProductBuildProfile,
            ProductBuildProfileError,
            load_product_build_profile,
        )
    except ImportError:  # pragma: no cover - generic export installs omit this helper.
        ProductBuildProfile = Any  # type: ignore[misc,assignment]
        ProductBuildProfileError = ValueError  # type: ignore[assignment]
        load_product_build_profile = None  # type: ignore[assignment]

REPORT_FILE_NAME = "report.json"


def run_compile_host(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    diagnostics: list[str] = []
    product_profile = resolve_product_profile(args, diagnostics)
    repo_root = (
        resolve_compile_host_path(args.repo_root, "repo_root", diagnostics)
        if getattr(args, "repo_root", None)
        else default_repo_root()
    )
    validate_report = compile_host_validate_report_path(
        args,
        out_root,
        diagnostics,
    )
    stage_dir = out_root / "stages" / "compile_host"
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=CompileHost profile={args.profile}")
    print(f"validate_report={validate_report if validate_report else '<invalid>'}")
    print(f"report={report_path}")

    # Product profiles are build-only compositions and do not require the
    # project export validator's (deliberately narrower) debug/release plan.
    # Generic export invocations retain the existing validate-report gate.
    fatal = repo_root is None or (product_profile is None and validate_report is None)
    if fatal:
        compile_plan = None
    elif product_profile is not None:
        compile_plan = product_compile_host_plan(product_profile)
    else:
        compile_plan = load_compile_host_plan(validate_report, args.profile, diagnostics)
    command: list[str] = []
    host_executable = None
    link_plan: dict[str, object] | None = None
    stdout_lines: list[str] = []
    stderr_lines: list[str] = []

    if compile_plan is None:
        fatal = True
    else:
        target_dir = compile_host_resolved_target_dir(args, out_root, diagnostics)
        if target_dir is None:
            fatal = True
        else:
            command = compile_host_command(
                args,
                out_root,
                compile_plan,
                target_dir=target_dir,
            )
            link_plan = compile_host_link_plan(compile_plan)
            host_executable = compile_host_executable_path(
                out_root,
                compile_plan,
                args,
                target_dir=target_dir,
            )
            print(shell_join(command))
            print(f"host={host_executable}")

    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if fatal else 0

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        fatal = True
        diagnostics.append(
            f"CompileHost stage directory {stage_dir} could not be created: {error}"
        )
        report = {
            "stage": "CompileHost",
            "profile": args.profile,
            "fatal": fatal,
            "diagnostics": diagnostics,
            "validate_report": str(validate_report) if validate_report else None,
            "command": command,
            "host_executable": str(host_executable) if host_executable else None,
            "exit_code": 2,
            "stdout_lines": stdout_lines,
            "stderr_lines": stderr_lines,
        }
        add_compile_host_link_plan(report, link_plan)
        print(json.dumps(report, indent=2))
        return 2
    exit_code = 2
    if fatal:
        exit_code = 2
    else:
        try:
            compile_result = subprocess.run(
                command,
                cwd=repo_root,
                capture_output=True,
                text=True,
            )
            exit_code = compile_result.returncode
            stdout_lines = split_subprocess_output(compile_result.stdout)
            stderr_lines = split_subprocess_output(compile_result.stderr)
            if not fatal and compile_result.returncode == 0 and compile_plan is not None:
                materialize_product_host_output(
                    compile_plan,
                    target_dir,
                    host_executable,
                )
        except OSError as error:
            exit_code = 2
            fatal = True
            diagnostics.append(
                f"CompileHost cargo command {command[0]} could not start: {error}"
            )
        if not fatal and exit_code != 0:
            fatal = True
            diagnostics.append(f"CompileHost cargo command exited with code {exit_code}")
        elif not fatal and host_executable:
            output_diagnostic = compile_host_output_diagnostic(host_executable)
            if output_diagnostic:
                fatal = True
                diagnostics.append(output_diagnostic)

    report = {
        "stage": "CompileHost",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report) if validate_report else None,
        "command": command,
        "host_executable": str(host_executable) if host_executable else None,
        "exit_code": exit_code,
        "stdout_lines": stdout_lines,
        "stderr_lines": stderr_lines,
    }
    if product_profile is not None:
        report["product"] = {
            "profile": product_profile.name,
            "build_set_id": product_profile.build_set_id,
            "product_kind": product_profile.product_kind,
            "target_triple": product_profile.target_triple,
            "cargo_profile": product_profile.cargo_profile,
        }
    add_compile_host_link_plan(report, link_plan)
    report_written = write_report_targets([("CompileHost report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0


def compile_host_validate_report_path(
    args: argparse.Namespace,
    out_root: Path,
    diagnostics: list[str],
) -> Path | None:
    if not getattr(args, "validate_report", None):
        return out_root / "stages" / "validate" / REPORT_FILE_NAME
    try:
        return resolve_user_path(args.validate_report)
    except OSError as error:
        diagnostics.append(
            f"CompileHost validate_report {args.validate_report} could not be resolved: {error}"
        )
        return None


def resolve_compile_host_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="CompileHost")


def compile_host_command(
    args: argparse.Namespace,
    out_root: Path,
    compile_plan: dict[str, Any],
    *,
    target_dir: Path | None = None,
) -> list[str]:
    command = list(compile_plan["command"])
    if command:
        command[0] = args.cargo
    if not args.no_locked and "--locked" not in command:
        command.append("--locked")
    if args.offline and "--offline" not in command:
        command.append("--offline")
    product_profile = compile_plan.get("product_profile")
    target_triple = compile_plan.get("target_triple")
    cargo_profile = compile_plan.get("cargo_profile")
    if isinstance(product_profile, str):
        if isinstance(cargo_profile, str) and cargo_profile not in {"debug", "release"}:
            if "--profile" not in command and not any(
                entry == "--release" or entry.startswith("--release=")
                for entry in command
            ):
                command.extend(["--profile", cargo_profile])
        if isinstance(target_triple, str) and target_triple:
            command = command_with_option(command, "--target", target_triple)
    if target_dir is None:
        target_dir = (
            resolve_user_path(args.target_dir)
            if args.target_dir
            else compile_host_target_dir(out_root)
        )
    return command_with_option(command, "--target-dir", str(target_dir))


def compile_host_target_dir(out_root: Path) -> Path:
    return (out_root / "stages" / "compile_host" / "target").resolve()


def compile_host_resolved_target_dir(
    args: argparse.Namespace,
    out_root: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        if args.target_dir:
            return resolve_user_path(args.target_dir)
        return compile_host_target_dir(out_root)
    except OSError as error:
        label = "CompileHost target_dir" if args.target_dir else "CompileHost default target_dir"
        value = args.target_dir if args.target_dir else out_root / "stages" / "compile_host" / "target"
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def compile_host_executable_path(
    out_root: Path,
    compile_plan: dict[str, Any],
    args: argparse.Namespace | None = None,
    *,
    target_dir: Path | None = None,
) -> Path | None:
    binary = compile_plan.get("binary")
    cargo_profile = compile_plan.get("cargo_profile", "debug")
    if not isinstance(binary, str) or not binary:
        return None
    if not isinstance(cargo_profile, str) or not cargo_profile:
        cargo_profile = "debug"
    executable_name = str(binary) + (".exe" if os.name == "nt" else "")
    if target_dir is None:
        target_dir = (
            resolve_user_path(args.target_dir)
            if args is not None and getattr(args, "target_dir", None)
            else compile_host_target_dir(out_root)
        )
    target_triple = compile_plan.get("target_triple")
    if isinstance(target_triple, str) and target_triple.strip():
        # A Cargo build with --target stores products under
        # <target-dir>/<triple>/<profile>. Keep a flat fallback for test
        # fixtures and older custom Cargo wrappers.
        targeted = target_dir / target_triple.strip() / cargo_profile / executable_name
        flat = target_dir / cargo_profile / executable_name
        return targeted if targeted.exists() or not flat.exists() else flat
    return target_dir / cargo_profile / executable_name


def compile_host_cargo_executable_path(
    out_root: Path,
    compile_plan: dict[str, Any],
    args: argparse.Namespace | None = None,
    *,
    target_dir: Path | None = None,
) -> Path | None:
    """Resolve the unrenamed Cargo output for a product host target."""

    cargo_binary = compile_plan.get("cargo_binary", compile_plan.get("binary"))
    if not isinstance(cargo_binary, str) or not cargo_binary:
        return None
    plan = dict(compile_plan)
    plan["binary"] = cargo_binary
    return compile_host_executable_path(
        out_root,
        plan,
        args,
        target_dir=target_dir,
    )


def materialize_product_host_output(
    compile_plan: dict[str, Any],
    target_dir: Path,
    published_path: Path | None,
) -> None:
    """Copy a separately gated Cargo product bin to its stable public name."""

    if published_path is None:
        return
    cargo_binary = compile_plan.get("cargo_binary")
    published_binary = compile_plan.get("binary")
    if not isinstance(cargo_binary, str) or not isinstance(published_binary, str):
        return
    cargo_name = cargo_binary + (".exe" if os.name == "nt" else "")
    published_name = published_binary + (".exe" if os.name == "nt" else "")
    if cargo_name.casefold() == published_name.casefold():
        return
    profile = str(compile_plan.get("cargo_profile", "debug"))
    triple = str(compile_plan.get("target_triple", "")).strip()
    candidates = [
        target_dir / triple / profile / cargo_name if triple else None,
        target_dir / profile / cargo_name,
    ]
    source = next((candidate for candidate in candidates if candidate and candidate.is_file()), None)
    if source is None:
        # Keep the normal output diagnostic responsible for the useful error;
        # this helper must not fabricate a successful product artifact.
        return
    published_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, published_path)


def resolve_product_profile(
    args: argparse.Namespace,
    diagnostics: list[str],
) -> ProductBuildProfile | None:
    name = getattr(args, "product_profile", None)
    if not name:
        return None
    if load_product_build_profile is None:
        diagnostics.append("product profiles are unavailable in this export installation")
        return None
    try:
        profile = load_product_build_profile(
            name,
            path=getattr(args, "product_profile_file", None),
        )
    except ProductBuildProfileError as error:
        diagnostics.append(str(error))
        return None
    requested_target = getattr(args, "target_triple", None)
    if requested_target and requested_target != profile.target_triple:
        diagnostics.append(
            f"product profile {profile.name} requires target {profile.target_triple}"
        )
        return None
    return profile


def product_compile_host_plan(profile: ProductBuildProfile) -> dict[str, object]:
    """Create the explicit CompileHost plan owned by a product profile."""

    app_features = list(profile.app_features)
    cargo_binary = profile.cargo_app_binary
    return {
        "product_profile": profile.name,
        "build_set_id": profile.build_set_id,
        "product_kind": profile.product_kind,
        "target_triple": profile.target_triple,
        "package": profile.app_package,
        "binary": profile.app_binary,
        "cargo_binary": cargo_binary,
        "manifest_path": "Cargo.toml",
        "target_dir": "stages/compile_host/target",
        "cargo_profile": profile.cargo_profile,
        "release": profile.cargo_profile not in {"debug", "profiling"},
        "app_features": app_features,
        "runtime_features": list(profile.runtime_features),
        "expected_runtime_plugins": [],
        "linked_runtime_crates": [],
        "command": [
            "cargo",
            "build",
            "-p",
            profile.app_package,
            "--bin",
            cargo_binary,
            "--no-default-features",
            "--features",
            " ".join(app_features),
        ],
    }


def compile_host_output_diagnostic(host_executable: Path) -> str | None:
    if not host_executable.exists():
        return f"CompileHost output {host_executable} does not exist"
    if not host_executable.is_file():
        return f"CompileHost output {host_executable} is not a file"
    try:
        if host_executable.stat().st_size <= 0:
            return f"CompileHost output {host_executable} is empty"
    except OSError as error:
        return f"CompileHost output {host_executable} could not be inspected: {error}"
    return None


def compile_host_link_plan(compile_plan: dict[str, Any]) -> dict[str, object]:
    return {
        "app_features": compile_host_plan_list(compile_plan, "app_features"),
        "runtime_features": compile_host_plan_list(compile_plan, "runtime_features"),
        "expected_runtime_plugins": compile_host_plan_list(
            compile_plan,
            "expected_runtime_plugins",
        ),
        "linked_runtime_crates": compile_host_plan_list(
            compile_plan,
            "linked_runtime_crates",
        ),
    }


def add_compile_host_link_plan(
    report: dict[str, object],
    link_plan: dict[str, object] | None,
) -> None:
    if link_plan is not None:
        report["link_plan"] = link_plan


def compile_host_plan_list(
    compile_plan: dict[str, Any],
    field: str,
) -> list[object]:
    value = compile_plan.get(field)
    return list(value) if isinstance(value, list) else []


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)
