"""Capture the real Linux startup card, first frame, and normal editor shutdown.

Uses only windows belonging to the launched process/its owned splash child, so
other editor sessions on the same desktop are left alone. Each run starts with
fresh application and Mesa caches; it does not evict the machine's page cache.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import threading
import time


def windows(pid: int, env: dict[str, str]) -> list[str]:
    result = subprocess.run(
        ["xdotool", "search", "--onlyvisible", "--pid", str(pid)],
        env=env, capture_output=True, text=True, timeout=5,
    )
    return result.stdout.split() if result.returncode == 0 else []


def splash_children(pid: int) -> list[int]:
    try:
        children = Path(f"/proc/{pid}/task/{pid}/children").read_text().split()
    except FileNotFoundError:
        return []
    owned = []
    for child in children:
        try:
            args = Path(f"/proc/{child}/cmdline").read_bytes().split(b"\0")
            if b"--zircon-startup-splash-child" in args:
                owned.append(int(child))
        except FileNotFoundError:
            pass
    return owned


def capture(window: str, output: Path, env: dict[str, str]) -> None:
    subprocess.run(
        ["import", "-window", window, str(output)], env=env, check=True, timeout=20,
    )


def probe_child(staging: Path, evidence: Path) -> dict:
    """Exercise rendering/dismissal without pretending this is engine readiness."""
    evidence.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, DISPLAY=os.environ.get("DISPLAY", ":3"))
    results: dict = {"scope": "splash child protocol; no runtime/workbench is launched", "cases": []}
    for case in ["stages-ready", "parent-eof", "escape"]:
        started = time.monotonic()
        messages: list[str] = []
        with (evidence / f"{case}-stderr.log").open("wb") as log:
            process = subprocess.Popen(
                [str(staging / "zircon_editor"), "--zircon-startup-splash-child", "project"],
                env=env, cwd=staging, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
                text=True,
            )

            def drain() -> None:
                assert process.stdout is not None
                messages.extend(line.strip() for line in process.stdout)

            reader = threading.Thread(target=drain, daemon=True)
            reader.start()
            try:
                while "visible" not in messages:
                    if process.poll() is not None or time.monotonic() - started > 15:
                        raise RuntimeError(f"{case}: splash child did not become visible")
                    time.sleep(0.02)
                window = windows(process.pid, env)[-1]
                assert process.stdin is not None
                if case == "stages-ready":
                    for stage in ["runtime", "gpu", "project"]:
                        process.stdin.write(stage + "\n")
                        process.stdin.flush()
                        time.sleep(0.2)
                        capture(window, evidence / f"stage-{stage}.png", env)
                    process.stdin.write("ready\n")
                    process.stdin.flush()
                elif case == "escape":
                    subprocess.run(["xdotool", "windowactivate", "--sync", window, "key", "Escape"],
                                   env=env, check=True, timeout=5)
                else:
                    process.stdin.close()
                process.wait(timeout=5)
                reader.join(timeout=2)
                if process.returncode != 0 or "closed" not in messages:
                    raise RuntimeError(f"{case}: child did not close cleanly")
                results["cases"].append({"case": case, "returncode": process.returncode,
                                         "messages": messages, "seconds": time.monotonic() - started})
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=5)
                if not process.stdin.closed:
                    process.stdin.close()
    (evidence / "metrics.json").write_text(json.dumps(results, indent=2) + "\n")
    return results


def run(staging: Path, evidence: Path, mode: str, project: Path | None, timeout: float) -> dict:
    evidence.mkdir(parents=True, exist_ok=False)
    xdg = evidence / "xdg-runtime"
    xdg.mkdir(mode=0o700)
    env = dict(os.environ)
    env.update(
        DISPLAY=env.get("DISPLAY", ":3"), WGPU_BACKEND="vulkan",
        ZIRCON_ALLOW_SOFTWARE_ADAPTER="1", XDG_RUNTIME_DIR=str(xdg),
        ZIRCON_ASSET_ROOT=str(staging / "assets"),
        ZIRCON_RUNTIME_LIBRARY=str(staging / "libzircon_runtime.so"),
        ZIRCON_CONFIG_PATH=str(evidence / "profile" / "editor-config.json"),
        ZIRCON_LOG_ROOT=str(evidence / "logs"),
        ZIRCON_UI_COMPILED_ARTIFACT_CACHE=str(evidence / "ui-cache"),
        XDG_CONFIG_HOME=str(evidence / "profile" / "config"),
        XDG_CACHE_HOME=str(evidence / "profile" / "cache"),
        XDG_DATA_HOME=str(evidence / "profile" / "data"),
        MESA_SHADER_CACHE_DIR=str(evidence / "mesa-cache"),
        ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG=str(evidence / "first-frame.png"),
        ZIRCON_EDITOR_STARTUP_SPLASH="1",
    )
    env.pop("ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME", None)
    command = [str(staging / "zircon_editor")]
    if mode == "create":
        command += ["--create-project", "--project-name", "SplashSmoke", "--location",
                    str(evidence / "projects"), "--template", "renderable-empty"]
    elif mode == "open":
        if project is None:
            raise ValueError("--project is required for open mode")
        command += ["--project", str(project)]
    started = time.monotonic()
    metrics: dict = {
        "command": command, "mode": mode, "events": [],
        "cold_scope": "new process, app profile, UI cache, and Mesa cache; shared OS page cache",
    }
    lines: list[str] = []
    stdout = (evidence / "stdout.log").open("wb")
    process = subprocess.Popen(command, cwd=staging, env=env, stdout=stdout, stderr=subprocess.PIPE)
    metrics["pid"] = process.pid

    def drain() -> None:
        assert process.stderr is not None
        with (evidence / "stderr.log").open("wb") as log:
            for raw in process.stderr:
                log.write(raw)
                log.flush()
                line = raw.decode(errors="replace").strip()
                lines.append(line)
                if "startup_splash " in line or "first_frame_presented" in line:
                    event = {"observed_seconds": time.monotonic() - started, "line": line}
                    metrics["events"].append(event)
                    print(f"{mode} {event['observed_seconds']:.3f}s {line}", flush=True)

    reader = threading.Thread(target=drain, daemon=True)
    reader.start()
    child_pids: set[int] = set()
    splash_window: str | None = None
    resized = False
    try:
        while True:
            if process.poll() is not None:
                raise RuntimeError(f"editor exited before smoke completion: {process.returncode}")
            if time.monotonic() - started > timeout:
                raise TimeoutError("editor did not present its first frame in time")
            child_pids.update(splash_children(process.pid))
            visible = any("startup_splash event=visible " in line for line in lines)
            first_frame = any("startup_splash event=first_frame " in line for line in lines)
            if visible and splash_window is None:
                for pid in child_pids:
                    found = windows(pid, env)
                    if found:
                        splash_window = found[-1]
                        metrics["splash_window"] = splash_window
                        capture(splash_window, evidence / "splash.png", env)
                        break
            if splash_window is not None and not resized and not first_frame:
                requested = time.monotonic()
                resize_log_start = len(lines)
                subprocess.run(["xdotool", "windowsize", splash_window, "420", "360"],
                               env=env, check=True, timeout=5)
                while not any("startup_splash event=resized " in line
                              for line in lines[resize_log_start:]):
                    if time.monotonic() - requested > 5:
                        raise TimeoutError("splash did not handle resize during loading")
                    time.sleep(0.02)
                metrics["splash_resize_ack_seconds"] = time.monotonic() - requested
                # Allow the native software buffer to repaint at the new size.
                time.sleep(0.15)
                capture(splash_window, evidence / "splash-compact.png", env)
                metrics["splash_resize_response_seconds"] = time.monotonic() - requested
                subprocess.run(["xdotool", "windowsize", splash_window, "640", "440"],
                               env=env, check=True, timeout=5)
                resized = True
            if first_frame and (evidence / "first-frame.png").is_file():
                metrics["first_frame_capture_observed_seconds"] = time.monotonic() - started
                break
            time.sleep(0.025)
        if splash_window is None or not resized:
            raise RuntimeError("startup finished without observable, responsive splash evidence")
        # The first-present callback must dismiss/reap the child without waiting for quit.
        deadline = time.monotonic() + 5
        while any(Path(f"/proc/{pid}").exists() for pid in child_pids):
            if time.monotonic() > deadline:
                raise TimeoutError("splash child survived the first workbench frame")
            time.sleep(0.025)
        metrics["splash_child_reaped_before_quit"] = True
        parent_windows = windows(process.pid, env)
        if not parent_windows:
            raise RuntimeError("workbench has no visible parent-owned native window")
        window = parent_windows[-1]
        metrics["workbench_window"] = window
        capture(window, evidence / "workbench.png", env)
        subprocess.run(["xdotool", "windowactivate", "--sync", window, "key", "alt+F4"],
                       env=env, check=True, timeout=10)
        process.wait(timeout=45)
        if process.returncode != 0:
            raise RuntimeError(f"normal close returned {process.returncode}")
        metrics["clean_quit"] = True
    except Exception as error:
        metrics["error"] = str(error)
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
    finally:
        reader.join(timeout=5)
        stdout.close()
        metrics["returncode"] = process.returncode
        metrics["total_seconds"] = time.monotonic() - started
        metrics["owned_splash_pids"] = sorted(child_pids)
        for line in lines:
            if "startup_splash event=first_frame " in line:
                first = re.search(r"elapsed_ms=([0-9.]+)", line)
                visible = re.search(r"visible_ms=([0-9.]+)", line)
                if first:
                    metrics["splash_launch_to_first_frame_ms"] = float(first.group(1))
                if visible:
                    metrics["splash_visible_until_first_frame_ms"] = float(visible.group(1))
            if "startup_splash event=visible " in line:
                visible = re.search(r"elapsed_ms=([0-9.]+)", line)
                if visible:
                    metrics["splash_launch_to_visible_ms"] = float(visible.group(1))
        for event in metrics["events"]:
            if "startup_splash event=first_frame " in event["line"]:
                metrics["process_to_first_frame_observed_ms"] = event["observed_seconds"] * 1000.0
        (evidence / "metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")
    return metrics


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staging", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--mode", choices=["welcome", "create", "open"], default="create")
    parser.add_argument("--project", type=Path)
    parser.add_argument("--timeout", type=float, default=240)
    parser.add_argument("--probe-child", action="store_true")
    args = parser.parse_args()
    if args.probe_child:
        print(json.dumps(probe_child(args.staging.resolve(), args.evidence.resolve()), indent=2))
        return 0
    metrics = run(args.staging.resolve(), args.evidence.resolve(), args.mode,
                  args.project.resolve() if args.project else None, args.timeout)
    print(json.dumps({key: value for key, value in metrics.items() if key != "events"}, indent=2))
    return 0 if metrics.get("clean_quit") and not metrics.get("error") else 1


if __name__ == "__main__":
    raise SystemExit(main())
