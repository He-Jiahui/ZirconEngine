#!/usr/bin/env python3
"""Exercise the staged Linux editor through X11, with real profiler receipts.

This observes opt-in operator JSON; edits and Play/Stop still enter through native
mouse/keyboard events. A baseline run records failed operations without declaring
acceptance. Output directories must be new to preserve preceding evidence.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import threading
import time

import psutil
from PIL import Image, ImageStat


class OperatorRun:
    def __init__(self, args: argparse.Namespace):
        self.args = args
        self.root = args.evidence.resolve()
        self.root.mkdir(parents=True, exist_ok=False)
        self.stage = args.staging.resolve()
        self.state_root = args.reuse_state.resolve() if args.reuse_state else self.root
        for name in ("profile", "projects", "ui-cache", "appdata"):
            (self.state_root / name).mkdir(exist_ok=True)
        (self.root / "xdg-runtime").mkdir(mode=0o700)
        self.env = dict(os.environ, DISPLAY=args.display, WGPU_BACKEND="vulkan",
            ZIRCON_ALLOW_SOFTWARE_ADAPTER="1", XDG_RUNTIME_DIR=str(self.root / "xdg-runtime"),
            ZIRCON_ASSET_ROOT=str(self.stage / "assets"),
            ZIRCON_RUNTIME_LIBRARY=str(self.stage / "libzircon_runtime.so"),
            ZIRCON_CONFIG_PATH=str(self.state_root / "profile/editor-config.json"),
            XDG_CONFIG_HOME=str(self.state_root / "profile/config"),
            XDG_CACHE_HOME=str(self.state_root / "profile/cache"),
            XDG_DATA_HOME=str(self.state_root / "profile/data"),
            ZIRCON_LOG_ROOT=str(self.root / "logs"),
            ZIRCON_UI_COMPILED_ARTIFACT_CACHE=str(self.state_root / "ui-cache"),
            ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG=str(self.root / "first-frame.png"),
            ZIRCON_EDITOR_OPERATOR_EVIDENCE=str(self.root / "operator.json"),
            ZIRCON_EDITOR_OPERATOR_GIZMO="1",
            ZIRCON_TRACE_PLAY_STOP="1", ZIRCON_EDITOR_TRACE_SLOW_REDRAW="1")
        if args.profile:
            self.env.update(ZIRCON_PROFILE_CAPTURE="1", ZIRCON_PROFILE_TIMELINE_ONLY="1",
                ZIRCON_PROFILE_COUNTERS_ONLY="1", TRACY_NO_SYS_TRACE="1",
                ZIRCON_EDITOR_TRACE_UI_ACTIVITY="1",
                ZIRCON_PROFILE_OUTPUT_ROOT=str(self.root / "metrics"),
                ZIRCON_PROFILE_SESSION="operator", ZIRCON_PROFILE_MAX_SPANS="500000",
                ZIRCON_PROFILE_MAX_COUNTERS="500000", ZIRCON_PROFILE_MAX_FRAMES="30000")
        self.record = {"staging": str(self.stage), "phases": [], "samples": [], "boundaries": []}
        self.capture_preload = None
        if args.renderdoccmd:
            # Isolated XDG_DATA_HOME must still discover the installed capture layer.
            distribution = args.renderdoccmd.resolve().parent.parent
            layers = self.root / "vulkan-layers"
            layers.mkdir()
            layer = json.loads((distribution / "etc/vulkan/implicit_layer.d/renderdoc_capture.json").read_text())
            layer["layer"]["library_path"] = str(distribution / "lib/librenderdoc.so")
            (layers / "renderdoc_capture.json").write_text(json.dumps(layer, indent=2) + "\n")
            self.env.update(ENABLE_VULKAN_RENDERDOC_CAPTURE="1",
                VK_ADD_IMPLICIT_LAYER_PATH=str(layers))
            if getattr(args, "renderdoc_api_helper", None):
                library = args.renderdoccmd.resolve().parent.parent / "lib/librenderdoc.so"
                captures = self.root / "renderdoc"
                captures.mkdir()
                # RenderDoc removes its preload from child environments unless
                # child capture is enabled. Inject only into the editor, after
                # time/bwrap, so their constructors cannot consume the preload.
                self.capture_preload = f"{args.renderdoc_api_helper.resolve()}:{library}"
                self.env.update(
                    ZE_RENDERDOC_LIBRARY=str(library),
                    ZE_RENDERDOC_CONTROL=str(self.root / "renderdoc-request"),
                    ZE_RENDERDOC_CAPTURE_ROOT=str(captures))
        self.profiler = None
        self.window = None
        self.width, self.height = 1672, 941
        self.done = threading.Event()
        self.monitor_processes = []

    def xd(self, *args):
        return subprocess.check_output(["xdotool", *map(str, args)], env=self.env,
                                       text=True, timeout=10).strip()

    def state(self):
        try:
            return json.loads((self.root / "operator.json").read_text())
        except (OSError, json.JSONDecodeError):
            return {}

    def wait(self, predicate, timeout=30):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.proc.poll() is not None:
                raise RuntimeError(f"editor exited early: {self.proc.returncode}")
            value = predicate()
            if value:
                return value
            time.sleep(.025)
        return None

    def shot(self, label):
        subprocess.run(["import", "-window", self.window, str(self.root / f"{label}.png")],
                       env=self.env, check=True, timeout=15)
        (self.root / f"{label}.json").write_text(json.dumps(self.state(), indent=2) + "\n")

    def key(self, chord):
        self.xd("windowactivate", "--sync", self.window)
        self.xd("key", "--window", self.window, "--clearmodifiers", chord)

    def type_text(self, value):
        self.xd("type", "--window", self.window, "--clearmodifiers", "--delay", 25, value)

    def wait_mode(self, expected, timeout):
        observed = set()

        def poll():
            state = self.state()
            mode = state.get("play_mode")
            if mode in ("Starting", "Stopping") and mode not in observed:
                observed.add(mode)
                self.shot(mode.lower())
            return state if mode == expected else None

        return self.wait(poll, timeout)

    def click(self, x, y):
        self.xd("mousemove", "--window", self.window, round(x), round(y), "click", 1)

    def control(self, control_id):
        return next((c for c in self.state().get("controls", [])
                     if c["id"] == control_id and c["frame"][2] > 0), None)

    def click_control(self, control_id, fallback):
        control = self.control(control_id)
        if control:
            x, y, width, height = control["frame"]
            self.click(x + width / 2, y + height / 2)
        else:
            self.click(*fallback)

    def request_gpu_capture(self, label):
        # Optional external RenderDoc API helper. Ordinary measurements have no
        # helper, capture polling or additional redraw demand.
        control = self.env.get("ZE_RENDERDOC_CONTROL")
        if self.args.renderdoccmd and control:
            Path(control).write_text(label + "\n")

    def phase(self, name, duration, action=None):
        phase = {"name": name, "start_seconds": time.monotonic() - self.started,
                 "start_unix_ns": time.time_ns()}
        print(f"phase {name}", flush=True)
        self.request_gpu_capture("idle-forced-hover-present" if name == "idle" else name)
        if self.args.renderdoccmd and name == "idle":
            # Demand-driven idle has no GPU frame to capture. Label the single
            # hover-triggered present explicitly and keep it out of timing runs.
            time.sleep(.15)
            self.xd("mousemove", "--window", self.window, self.width - 50, 76)
        processes = []
        if self.args.profile:
            perf_log = (self.root / f"{name}.perf.log").open("wb")
            processes.append(subprocess.Popen(["perf", "record", "-e", "cpu-clock:u", "-F", "49",
                "--call-graph", "dwarf,8192", "-p", str(self.pid), "-o",
                str(self.root / f"{name}.perf.data"), "--", "sleep", str(duration)],
                stdout=perf_log, stderr=subprocess.STDOUT))
            if name == "interaction":
                stat_log = (self.root / "layout.perf-stat.txt").open("wb")
                processes.append(subprocess.Popen(["perf", "stat", "-e",
                    "instructions,cycles,cache-misses,context-switches,cpu-clock", "-p", str(self.pid),
                    "--", "sleep", str(duration)], stdout=stat_log, stderr=subprocess.STDOUT))
                software_log = (self.root / "layout.perf-stat-software.txt").open("wb")
                processes.append(subprocess.Popen(["perf", "stat", "-e",
                    "task-clock,context-switches,page-faults", "-p", str(self.pid),
                    "--", "sleep", str(duration)], stdout=software_log, stderr=subprocess.STDOUT))
        deadline = time.monotonic() + duration
        if action:
            action(deadline)
        while time.monotonic() < deadline:
            time.sleep(min(.1, max(.001, deadline - time.monotonic())))
        for process in processes:
            process.wait(timeout=10)
            self.record.setdefault("profiler_commands", []).append(
                {"phase": name, "command": process.args, "returncode": process.returncode})
        phase.update(end_seconds=time.monotonic() - self.started, end_unix_ns=time.time_ns())
        self.record["phases"].append(phase)
        self.shot(name)
        if self.args.profile and name in ("idle", "play"):
            with (self.root / f"{name}.ps.txt").open("w") as output:
                subprocess.run(["ps", "-p", str(self.pid), "-o",
                    "pid,pcpu,rss,vsz,etimes,comm"], stdout=output, check=True)
            memory = subprocess.run(["smem", "-c", "pid rss pss uss", "-P",
                re.escape(str(self.stage / "zircon_editor"))], capture_output=True, text=True)
            self.record.setdefault("profiler_commands", []).append(
                {"phase": name, "command": memory.args, "returncode": memory.returncode,
                 "stderr": memory.stderr})
            rows = memory.stdout.splitlines()
            selected = [line for line in rows[1:] if line.split() and line.split()[0] == str(self.pid)]
            (self.root / f"{name}.smem.txt").write_text("\n".join(rows[:1] + selected) + "\n")

    def sample(self):
        process = psutil.Process(self.pid)
        process.cpu_percent()
        while not self.done.wait(.5):
            try:
                memory = process.memory_full_info()
                self.record["samples"].append(dict(seconds=time.monotonic() - self.started,
                    cpu_percent=process.cpu_percent(), rss_bytes=memory.rss, pss_bytes=memory.pss,
                    unix_ns=time.time_ns()))
                mode = self.state().get("play_mode")
                if mode and (not self.record.get("mode_observations") or
                             self.record["mode_observations"][-1]["mode"] != mode):
                    self.record.setdefault("mode_observations", []).append(
                        dict(unix_ns=time.time_ns(), mode=mode))
            except psutil.NoSuchProcess:
                return

    def capture_splash(self):
        while not self.done.wait(.1):
            try:
                children = psutil.Process(self.pid).children()
                splash = next((p for p in children if "--zircon-startup-splash-child" in p.cmdline()), None)
                if not splash:
                    continue
                windows = self.xd("search", "--onlyvisible", "--pid", splash.pid).splitlines()
                if windows:
                    subprocess.run(["import", "-window", windows[-1], str(self.root / "splash.png")],
                                   env=self.env, check=True, timeout=10)
                    with Image.open(self.root / "splash.png") as captured:
                        data = captured.convert("RGB").tobytes()
                    # X11 can map the child before its first software-buffer
                    # presentation. Require the Zircon accent markers, rather
                    # than accepting a nonuniform window-manager background.
                    if sum(b > r + 50 and b > g + 20
                           for r, g, b in zip(data[0::3], data[1::3], data[2::3])) < 8:
                        continue
                    self.record["splash_observed_seconds"] = time.monotonic() - self.started
                    return
            except (psutil.Error, subprocess.SubprocessError):
                pass

    def drain(self):
        with (self.root / "stderr.log").open("wb") as log:
            for line in self.proc.stderr:
                log.write(line)
                log.flush()
                if b"startup_boundary" in line or b"startup_splash" in line or b"mvp_play_boundary" in line:
                    self.record["boundaries"].append(dict(seconds=time.monotonic() - self.started,
                        text=line.decode(errors="replace").strip()))

    def interaction(self, deadline):
        i = 0
        while time.monotonic() < deadline:
            # Exercise hierarchy hover/scroll, inspector and toolbar hit testing.
            self.xd("mousemove", "--window", self.window,
                    int(self.width * .82), 215 + (i % 8) * 28)
            if i % 4 == 0:
                self.xd("click", 5 if i % 8 else 4)
            i += 1
            time.sleep(.05)

    def resize(self, deadline):
        while time.monotonic() < deadline:
            for width, height in ((1500, 870), (1672, 941)):
                self.xd("windowsize", self.window, width, height)
                time.sleep(.3)

    def dock_resize(self, deadline):
        # The bottom dock has a real native splitter. The Level Editor's
        # authored viewport/right-panel band is currently fixed.
        self.click(70, self.height - 46)
        splitter = self.wait(lambda: frame if (frame := self.state().get("splitters", {}).get("bottom"))
                             and frame[2] > 0 and frame[3] > 0 else None, 10)
        if not splitter:
            raise AssertionError("bottom dock splitter geometry missing")
        before = self.control("UnrealSceneViewViewport")
        if not before:
            raise AssertionError("scene viewport geometry missing")
        x, y, width, height = splitter
        pointer_x, pointer_y = x + width / 2, y + height / 2
        self.xd("mousemove", "--window", self.window, round(pointer_x),
                round(pointer_y), "mousedown", 1)
        saw_change = False
        while time.monotonic() < deadline:
            for offset in (-25, -60, -90, 0):
                self.xd("mousemove", "--window", self.window, round(pointer_x),
                        round(pointer_y + offset))
                time.sleep(.15)
                current = self.control("UnrealSceneViewViewport")
                if current and abs(current["frame"][3] - before["frame"][3]) > 10 and not saw_change:
                    saw_change = True
                    self.shot("dock-resize-mid")
        self.xd("mouseup", 1)
        after = self.control("UnrealSceneViewViewport")
        self.record["dock_resize_frames"] = [before["frame"], after["frame"] if after else None]
        self.record["dock_resize_verified"] = saw_change
        if not saw_change:
            raise AssertionError("native splitter drag did not change viewport geometry")
        self.click(70, self.height - 46)

    def drag(self, deadline):
        # The canonical controller publishes read-only projected handle segments.
        # All mutations below still enter the native pointer pipeline.
        self.key("f")
        self.key("w")
        before = self.state().get("inspector")
        lines = self.wait(lambda: [line for line in self.state().get("gizmo_lines", [])
                                  if line["axis"] == "Some(X)"], 10)
        if not lines:
            raise AssertionError("selected dynamic primitive has no move handle")
        line = max(lines, key=lambda line: sum((b-a)**2 for a,b in zip(line["start"],line["end"])))
        origin_x, origin_y, _, _ = self.state()["viewport_frame"]
        start, end = line["start"], line["end"]
        dx, dy = end[0]-start[0], end[1]-start[1]
        x, y = origin_x + start[0] + dx*.65, origin_y + start[1] + dy*.65
        self.xd("mousemove", "--window", self.window, round(x), round(y), "mousedown", 1)
        for step in range(1, 13):
            self.xd("mousemove", "--window", self.window, round(x+dx*step/12), round(y+dy*step/12))
            time.sleep(.04)
        self.xd("mouseup", 1)
        changed = self.wait(lambda: (current := self.state().get("inspector")) and
                            current["id"] == before["id"] and current["translation"] != before["translation"], 15)
        self.record["gizmo_drag_verified"] = bool(changed)
        if not changed:
            raise AssertionError("native move-handle drag did not change the selected transform")

    def edit(self):
        self.request_gpu_capture("scene-edit")
        before = self.state()
        self.click_control("WorkbenchToolbarAddActor", (320, 76))
        opened = self.wait(lambda: next((c for c in self.state().get("controls", [])
                           if c["id"] == "UnrealAddMenuPopup" and c.get("popup_open")
                           and c["frame"][2] > 0), None), 10)
        self.shot("add-menu")
        popup = opened
        if popup and popup.get("popup_open") and popup.get("menu_items"):
            row = next(i for i, item in enumerate(popup["menu_items"])
                       if "Cube" in item.split("|")[0])
            x, y, width, _ = popup["frame"]
            # Canonical menu contract: padding6, row28, gap2 logical pixels.
            self.click(x + width / 2, y + 6 + row * 30 + 14)
        else:
            anchor = self.control("WorkbenchToolbarAddActor")
            if not anchor:
                raise AssertionError("Add toolbar control is missing")
            x, y, width, height = anchor["frame"]
            self.click(x + width / 2, y + height + 4 + 6 + 14)
        after = self.wait(lambda: (s if len(s.get("scene", [])) > len(before.get("scene", []))
                                  else None) if (s := self.state()) else None, 10)
        self.record["add_verified"] = bool(after)
        self.shot("scene-edit")
        if not after and not self.args.baseline:
            raise AssertionError("Add Cube did not increase the authoritative hierarchy")
        if after:
            selected = [row for row in after["scene"] if row["selected"]]
            inspector = after.get("inspector") or {}
            self.record["selection_sync_verified"] = len(selected) == 1 and selected[0]["id"] == inspector.get("id")
            if not self.record["selection_sync_verified"]:
                raise AssertionError("Outliner and Details selected identities disagree")
        self.key("Escape")
        if after and self.args.extended_edit:
            self.edit_transform("WorkbenchTransformPositionX", "2.5", "translation", 0)
            self.edit_transform("WorkbenchTransformRotationZ", "30", "rotation_degrees", 2)
            self.edit_transform("WorkbenchTransformScaleX", "1.25", "scale", 0)
            identity = self.state()["inspector"]["id"]
            self.rename_selected(f"OperatorCube {identity}")
            self.delete_and_undo_selected()
        self.record["edit_before_play"] = self.state()

    def select_outliner(self, name):
        state = self.state()
        entry = next(row for row in state["scene"] if row["name"] == name)
        row = next(c for c in state["controls"] if c["id"].startswith("WorkbenchScene")
                   and c["text"] == name and c["frame"][3] > 0)
        x, y, width, height = row["frame"]
        self.click(x + width * .35, y + height / 2)
        if not self.wait(lambda: (s.get("inspector") or {}).get("id") == entry["id"]
                         and [r["id"] for r in s.get("scene", []) if r["selected"]] == [entry["id"]]
                         if (s := self.state()) else False, 15):
            raise AssertionError(f"Outliner selection did not reach Details: {name}")
        return entry["id"]

    def verify_scene_selection(self):
        camera = self.select_outliner("Camera")
        self.shot("outliner-selection")
        x, y, width, height = self.state()["viewport_frame"]
        self.click(x + width / 2, y + height / 2)
        picked = self.wait(lambda: s if (s := self.state()).get("inspector", {})
                           and s["inspector"]["id"] != camera
                           and s["inspector"]["name"] == "Cube" else None, 15)
        if not picked:
            self.shot("viewport-pick-failed")
            raise AssertionError("native viewport pick did not select the template Cube")
        self.record["viewport_pick_verified"] = True
        self.record["outliner_selection_verified"] = self.select_outliner("Camera") == camera
        self.shot("selection-roundtrip")

    def verify_saved_reopen(self):
        previous = json.loads((self.state_root / "metrics.json").read_text())["edit_before_play"]
        expected = previous["inspector"]
        if {(r["id"], r["name"]) for r in self.state()["scene"]} != \
                {(r["id"], r["name"]) for r in previous["scene"]}:
            raise AssertionError("reopened hierarchy differs from the saved native edit")
        self.select_outliner(expected["name"])
        actual = self.state()["inspector"]
        if any(actual[k] != expected[k] for k in ("id", "name", "translation", "scale")) or \
                any(abs(float(a)-float(b)) > .001 for a, b in zip(actual["rotation_degrees"], expected["rotation_degrees"])):
            raise AssertionError("reopened Details differs from the saved native transform")
        self.record["saved_reopen_verified"] = True
        self.shot("saved-reopen")

    def edit_transform(self, control_id, value, field, axis):
        control = self.control(control_id)
        if not control or control.get("disabled"):
            raise AssertionError(f"transform field unavailable: {control_id}")
        self.click_control(control_id, (0, 0))
        self.key("ctrl+a")
        self.type_text(value)
        self.key("Return")

        def committed():
            inspector = self.state().get("inspector") or {}
            values = inspector.get(field) or []
            visible = self.control(control_id)
            try:
                return len(values) > axis and abs(float(values[axis]) - float(value)) < .001 and \
                    visible and abs(float(visible["value"].split()[0]) - float(value)) < .001
            except (ValueError, KeyError):
                return False

        if not self.wait(committed, 15):
            self.shot("transform-failed")
            raise AssertionError(f"{control_id} did not commit to both scene and visible Details")
        self.record.setdefault("transform_verified", []).append(control_id)
        self.shot(field)

    def verify_visible_viewport(self, label):
        # A state transition alone cannot establish that the native viewport painted.
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            self.shot(label)
            observed = self.state()
            media = self.control("WorkbenchViewportSurface")
            x, y, width, height = media["frame"] if media else observed["viewport_frame"]
            image = observed.get("game_image") or {}
            matched = []
            with Image.open(self.root / f"{label}.png") as picture:
                rgb = picture.convert("RGB")
                viewport = rgb.crop((round(x + 8), round(y + 40),
                    round(x + width - 8), round(y + height - 8)))
                deviation = max(ImageStat.Stat(viewport).stddev)
                samples = list(image.get("cpu_samples") or [])
                if image.get("cpu_center_rgba"):
                    samples.append({"x": .5, "y": .5, "rgba": image["cpu_center_rgba"]})
                for sample in samples:
                    actual = rgb.getpixel((int(x + width * sample["x"]), int(y + height * sample["y"])))
                    expected = sample["rgba"]
                    matched.append({"actual": actual, "expected": expected,
                        "matches": max(abs(a-b) for a,b in zip(actual, expected[:3])) <= 12})
            pixels_match = not image.get("cpu_bytes") or (
                len([p for p in matched if p["matches"]]) >= 4
                and matched[-1]["matches"]  # Compare the scene center as well as background samples.
                and len({tuple(p["expected"][:3]) for p in matched}) > 1)
            if deviation > 5 and pixels_match and not observed.get("play_transition"):
                self.record[label + "_pixel_stddev"] = deviation
                self.record[label + "_runtime_pixel_comparison"] = matched
                return
            time.sleep(.25)
        raise AssertionError(f"{label}: native viewport did not paint the completed runtime image")

    def rename_selected(self, name):
        inspector = self.state().get("inspector") or {}
        row = next((c for c in self.state().get("controls", [])
                    if c["id"].startswith("WorkbenchScene") and c["text"] == inspector.get("name")
                    and c["frame"][3] > 0), None)
        if not row:
            raise AssertionError("selected Outliner row is not published")
        x, y, width, height = row["frame"]
        self.click(x + width * .35, y + height / 2)
        self.key("F2")
        self.key("ctrl+a")
        self.type_text(name)
        self.key("Return")
        renamed = self.wait(lambda: (s.get("inspector") or {}).get("name") == name and
            any(r["id"] == inspector["id"] and r["name"] == name for r in s.get("scene", []))
            if (s := self.state()) else False, 15)
        self.record["rename_verified"] = bool(renamed)
        self.shot("rename")
        if not renamed:
            raise AssertionError("F2 rename did not reach Outliner and Details")

    def delete_and_undo_selected(self):
        before = self.state()
        target = before["inspector"]["id"]
        row = next(c for c in before["controls"] if c["id"].startswith("WorkbenchScene")
                   and c["text"] == before["inspector"]["name"] and c["frame"][3] > 0)
        x, y, width, height = row["frame"]
        self.click(x + width * .35, y + height / 2)
        self.key("Delete")
        deleted = self.wait(lambda: not any(r["id"] == target for r in s.get("scene", []))
                            if (s := self.state()) else False, 15)
        self.record["delete_verified"] = bool(deleted)
        if not deleted:
            raise AssertionError("Delete did not remove the selected scene node")
        self.shot("delete")
        self.key("ctrl+z")
        restored = self.wait(lambda: any(r["id"] == target and r["name"] == before["inspector"]["name"]
                                        for r in s.get("scene", []))
                             if (s := self.state()) else False, 15)
        self.record["delete_undo_verified"] = bool(restored)
        if not restored:
            raise AssertionError("Undo did not restore the deleted scene node")
        self.select_outliner(before["inspector"]["name"])
        self.shot("delete-undo")

    def quit(self, save=True):
        if self.state().get("play_mode") in ("Playing", "Starting", "Stopping"):
            self.key("Escape")
            self.wait_mode("Edit", 45)
        if save:
            self.key("ctrl+s")
            time.sleep(1)
        self.key("alt+F4")
        deadline = time.monotonic() + 45
        while self.proc.poll() is None and time.monotonic() < deadline:
            prompt = self.state().get("close_prompt") or {}
            if prompt.get("visible"):
                self.shot("close-decision")
                x, y, width, height = prompt["discard_frame"]
                # This run owns the isolated smoke project and records the decision.
                self.click(x + width / 2, y + height / 2)
                self.record["close_discarded_owned_smoke_changes"] = True
                break
            time.sleep(.1)
        self.proc.wait(timeout=max(1, deadline - time.monotonic()))

    def run(self):
        if self.args.tracy_capture:
            self.profiler = subprocess.Popen([str(self.args.tracy_capture), "-a", "127.0.0.1",
                "-o", str(self.root / "operator.tracy")],
                stdout=(self.root / "tracy.log").open("wb"), stderr=subprocess.STDOUT)
        command = [str(self.stage / "zircon_editor")]
        if self.args.reuse_state:
            command += ["--project", str(self.state_root / "projects/OperatorSmoke")]
        else:
            command += ["--create-project", "--project-name", "OperatorSmoke", "--location",
                        str(self.state_root / "projects"), "--template", "renderable-empty"]
        if self.args.renderdoccmd and not getattr(self.args, "renderdoc_api_helper", None):
            command = [str(self.args.renderdoccmd), "capture", "--wait-for-exit", "-c",
                       str(self.root / "frame"), *command]
        # The Hub registry uses ~/.zircon independently of XDG. Give only this
        # child a private mount; preserve HOME and the other editors' registry.
        preload = ["--setenv", "LD_PRELOAD", self.capture_preload] if self.capture_preload else []
        command = ["bwrap", "--die-with-parent", "--bind", "/", "/", "--bind",
                   str(self.state_root / "appdata"), str(Path.home() / ".zircon"),
                   "--chdir", str(self.stage), *preload, "--", *command]
        command = ["/usr/bin/time", "-v", "-o", str(self.root / "time.txt"), *command]
        self.request_gpu_capture("startup")
        self.started = time.monotonic()
        self.record["launch_unix_ns"] = time.time_ns()
        self.proc = subprocess.Popen(command, cwd=self.stage, env=self.env,
            stdout=(self.root / "stdout.log").open("wb"), stderr=subprocess.PIPE)
        self.pid = self.wait(lambda: next((p.pid for p in psutil.Process(self.proc.pid).children(recursive=True)
                                         if p.name() == "zircon_editor" and
                                         "--zircon-startup-splash-child" not in p.cmdline()), None), 10)
        if not self.pid:
            raise RuntimeError("no editor child")
        (self.root / "pid").write_text(str(self.pid))
        reader = threading.Thread(target=self.drain, daemon=True)
        sampler = threading.Thread(target=self.sample, daemon=True)
        reader.start()
        sampler.start()
        threading.Thread(target=self.capture_splash, daemon=True).start()
        if self.args.profile:
            self.monitor_processes.append(subprocess.Popen(
                ["pidstat", "-h", "-u", "-r", "-p", str(self.pid), "1"],
                stdout=(self.root / "pidstat.txt").open("wb"), stderr=subprocess.STDOUT))
        try:
            if not self.wait(lambda: (self.root / "first-frame.png").exists(), 180):
                raise RuntimeError("no first workbench frame")
            self.record["first_workbench_seconds"] = time.monotonic() - self.started
            self.window = self.xd("search", "--onlyvisible", "--pid", self.pid).splitlines()[-1]
            self.xd("windowactivate", "--sync", self.window)
            self.shot("workbench")
            if self.args.renderdoccmd:
                if not self.capture_preload:
                    self.key("F12")
                self.xd("windowsize", self.window, 1668, 940)
                time.sleep(1)
            time.sleep(2)
            if self.args.verify_reopen:
                self.verify_saved_reopen()
            if self.args.clutter_count:
                extended = self.args.extended_edit
                self.args.extended_edit = False
                try:
                    for _ in range(self.args.clutter_count):
                        self.edit()
                finally:
                    self.args.extended_edit = extended
                self.record["clutter_scene_nodes"] = len(self.state().get("scene", []))
                self.key("ctrl+s")
                time.sleep(1)
            if not self.args.quick:
                self.phase("idle", self.args.idle_seconds or (20 if self.args.profile else 2))
                self.phase("interaction", 8, self.interaction)
                self.phase("resize", 4, self.resize)
                if self.args.extended_edit:
                    self.phase("dock-resize", 4, self.dock_resize)
            elif self.args.idle_seconds:
                self.phase("idle", self.args.idle_seconds)
            if not self.args.skip_edit:
                if self.args.extended_edit:
                    self.verify_scene_selection()
                self.edit()
                if not self.args.quick:
                    self.phase("gizmo", 3, self.drag)
                    self.record["edit_before_play"] = self.state()
            self.record["play_requested_unix_ns"] = time.time_ns()
            self.record["play_requested_seconds"] = time.monotonic() - self.started
            if self.args.play_trigger == "toolbar":
                self.click_control("WorkbenchRunPlay", (164, 75))
            else:
                self.key("alt+p")
            playing = self.wait_mode("Playing", 60 if not self.args.baseline else 3)
            self.record["play_verified"] = bool(playing)
            self.record["play_ready_unix_ns"] = time.time_ns()
            self.record["play_ready_seconds"] = time.monotonic() - self.started
            if not playing and not self.args.baseline:
                raise AssertionError("Alt+P did not reach Playing")
            if playing and not self.args.baseline:
                def game_ready():
                    state = self.state()
                    image = state.get("game_image") or {}
                    center = image.get("cpu_center_rgba")
                    # The asynchronous runtime returns a transparent placeholder
                    # until its first GPU readback completes. It is not game view
                    # evidence; it can merely reveal the cached Edit viewport.
                    return image.get("valid") and image.get("generation", 0) > 0 and state.get("document_pane") == "Game" and \
                        (not image.get("cpu_bytes") or (center and center[3] > 0))
                game = self.wait(game_ready, 60)
                self.record["game_view_verified"] = bool(game)
                if not game:
                    self.shot("game-view-failed")
                    raise AssertionError("Playing did not publish a valid image in the Game pane")
                self.request_gpu_capture("game-view")
                self.verify_visible_viewport("game-view")
                self.record["game_visible_unix_ns"] = time.time_ns()
                self.record["game_visible_seconds"] = time.monotonic() - self.started
            self.phase("play", 30 if self.args.profile else 3)
            self.record["stop_requested_unix_ns"] = time.time_ns()
            self.record["stop_requested_seconds"] = time.monotonic() - self.started
            if self.args.play_trigger == "toolbar":
                self.click_control("WorkbenchRunStop", (164, 75))
            else:
                self.key("Escape")
            stopped = self.wait_mode("Edit", 60 if not self.args.baseline else 3)
            # Mode publication precedes the UI projection by a callback. Verify
            # the restored workbench, rather than accepting that intermediate state.
            if stopped and not self.args.baseline:
                stopped = self.wait(lambda: s if (s := self.state()).get("play_mode") == "Edit"
                                    and s.get("document_pane") == "Scene"
                                    and (play := self.control("WorkbenchRunPlay"))
                                    and not play.get("disabled") else None, 30)
            self.record["stop_ready_unix_ns"] = time.time_ns()
            self.record["stop_ready_seconds"] = time.monotonic() - self.started
            self.record["stop_verified"] = bool(stopped) and bool(playing)
            self.shot("stop")
            if not stopped and not self.args.baseline:
                raise AssertionError("Escape did not restore Edit")
            if stopped and self.record.get("edit_before_play"):
                before = self.record["edit_before_play"]
                self.record["edit_restore_verified"] = stopped.get("scene") == before.get("scene") and \
                    stopped.get("inspector") == before.get("inspector")
                if not self.record["edit_restore_verified"] and not self.args.baseline:
                    raise AssertionError("Stop did not restore the edit hierarchy and Details values")
            self.quit()
            self.record["clean_shutdown"] = self.proc.returncode == 0
        except Exception as error:
            self.record["error"] = str(error)
            try:
                if self.window and self.proc.poll() is None:
                    self.quit(save=False)
                self.proc.wait(timeout=30)
            except (subprocess.SubprocessError, RuntimeError):
                if self.proc.poll() is None:
                    psutil.Process(self.pid).terminate()
                    self.proc.wait(timeout=15)
        finally:
            self.done.set()
            for monitor in self.monitor_processes:
                monitor.terminate()
                monitor.wait(timeout=5)
                self.record.setdefault("profiler_commands", []).append(
                    {"command": monitor.args, "returncode": monitor.returncode,
                     "termination": "owned sampler stopped after editor exit"})
            reader.join(timeout=5)
            sampler.join(timeout=2)
            if self.profiler:
                try:
                    self.profiler.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    self.profiler.terminate()
                    self.profiler.wait(timeout=5)
                self.record.setdefault("profiler_commands", []).append(
                    {"command": self.profiler.args, "returncode": self.profiler.returncode})
            self.record["returncode"] = self.proc.returncode
            self.record["total_monotonic_seconds"] = time.monotonic() - self.started
            self.record["total_realtime_seconds"] = (time.time_ns() - self.record["launch_unix_ns"]) / 1e9
            self.record["realtime_minus_monotonic_seconds"] = (
                self.record["total_realtime_seconds"] - self.record["total_monotonic_seconds"])
            idle = next((p for p in self.record["phases"] if p["name"] == "idle"), {})
            samples = [s for s in self.record["samples"]
                       if idle.get("start_seconds", 1e30) < s["seconds"] < idle.get("end_seconds", 0)]
            if samples:
                self.record["idle"] = dict(cpu_percent=statistics.mean(s["cpu_percent"] for s in samples),
                    rss_mib=statistics.mean(s["rss_bytes"] for s in samples) / 1024**2,
                    pss_mib=statistics.mean(s["pss_bytes"] for s in samples) / 1024**2)
            (self.root / "metrics.json").write_text(json.dumps(self.record, indent=2) + "\n")
            print(json.dumps({k: v for k, v in self.record.items() if k not in ("samples", "boundaries", "edit_before_play")}), flush=True)
        return 0 if self.record.get("clean_shutdown") and not self.record.get("error") else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staging", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--reuse-state", type=Path)
    parser.add_argument("--verify-reopen", action="store_true",
                        help="Compare the reopened scene with this state's prior native edit receipt")
    parser.add_argument("--display", default=":3")
    parser.add_argument("--profile", action="store_true")
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--tracy-capture", type=Path)
    parser.add_argument("--renderdoccmd", type=Path)
    parser.add_argument("--renderdoc-api-helper", type=Path,
                        help="Optional preloaded helper to trigger the installed RenderDoc API")
    parser.add_argument("--quick", action="store_true", help="Skip the profiling workload phases")
    parser.add_argument("--idle-seconds", type=float,
                        help="Override idle duration; also enables an idle sample with --quick")
    parser.add_argument("--skip-edit", action="store_true", help="Diagnose Play independently of scene editing")
    parser.add_argument("--extended-edit", action="store_true", help="Commit Details transforms, F2 rename and dock resize")
    parser.add_argument("--play-trigger", choices=("keymap", "toolbar"), default="keymap")
    parser.add_argument("--clutter-count", type=int, default=0,
                        help="Add this many real Cubes before recording interaction phases")
    return OperatorRun(parser.parse_args()).run()


if __name__ == "__main__":
    raise SystemExit(main())
