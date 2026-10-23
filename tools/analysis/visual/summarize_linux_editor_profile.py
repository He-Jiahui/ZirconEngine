#!/usr/bin/env python3
"""Summarize an operator smoke's retained timing evidence without hiding truncation.

Zone times are inclusive wall times, not additive CPU percentages. Present gaps
are reported only inside explicit workload phases; idle gaps are not UI stalls.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import re
import statistics


def distribution(values):
    ordered = sorted(values)
    if not ordered:
        return {"count": 0}
    return {"count": len(ordered), "sum": sum(ordered),
            "median": statistics.median(ordered),
            "p95": ordered[min(len(ordered) - 1, int(len(ordered) * .95))],
            "max": ordered[-1]}


def summarize(root: Path):
    metrics = json.loads((root / "metrics.json").read_text())
    traces = list((root / "metrics").glob("*/timeline.zrtrace.json"))
    if len(traces) != 1:
        raise ValueError(f"expected one editor timeline, found {len(traces)}")
    trace = json.loads(traces[0].read_text())
    retention = trace.get("recorder_retention", [])
    if not retention or any(r.get("frames", {}).get("overwritten", 0) for r in retention):
        raise ValueError("first-tick alignment requires a complete retained frame stream")
    # frame_update_enter immediately precedes the first retained-host tick.
    boundary = next(b for b in metrics["boundaries"]
                    if "marker=frame_update_enter unix_ms=" in b["text"])
    unix_us = int(re.search(r"unix_ms=(\d+)", boundary["text"])[1]) * 1000
    tick = next(f for f in trace["frames"] if f["name"] == "retained_host_tick")
    epoch_us = unix_us - tick["start_us"]
    counters = trace["counters"]
    counter_overwritten = any(r.get("counters", {}).get("overwritten", 0) for r in retention)
    first_counter = min((c["timestamp_us"] for c in counters), default=None)
    activity = []
    pattern = re.compile(r"ui_activity unix_ns=(\d+) counter=(\w+) value=([0-9.eE+-]+)")
    for line in (root / "stderr.log").read_text(errors="replace").splitlines():
        if matched := pattern.search(line):
            activity.append((int(matched[1]), matched[2], float(matched[3])))
    # CLOCK_MONOTONIC and Rust Instant both exclude VM suspension. Prefer the
    # observed monotonic anchor; retain wall-clock differences for honest receipts.
    monotonic_offset_us = tick["start_us"] - boundary["seconds"] * 1e6
    unix_samples = [s for s in metrics.get("samples", []) if "unix_ns" in s]
    def elapsed_for_unix(timestamp):
        if not unix_samples:
            return None
        sample = min(unix_samples, key=lambda s: abs(s["unix_ns"] - timestamp))
        return sample["seconds"] + (timestamp - sample["unix_ns"]) / 1e9

    result = {"source": str(traces[0]), "alignment":
              "first retained_host_tick aligned to observed frame_update_enter; monotonic phase bounds when available; reader scheduling adds uncertainty",
              "realtime_minus_monotonic_seconds": metrics.get("realtime_minus_monotonic_seconds"),
              "recorder_retention": retention, "phases": []}
    phases = list(metrics["phases"])
    if metrics.get("play_requested_unix_ns") and metrics.get("play_ready_unix_ns"):
        phases.append({"name": "play_start", "start_unix_ns": metrics["play_requested_unix_ns"],
                       "end_unix_ns": metrics["play_ready_unix_ns"]})
    if metrics.get("play_requested_unix_ns") and metrics.get("game_visible_unix_ns"):
        phases.append({"name": "play_first_game", "start_unix_ns": metrics["play_requested_unix_ns"],
                       "end_unix_ns": metrics["game_visible_unix_ns"]})
    if metrics.get("stop_requested_unix_ns") and metrics.get("stop_ready_unix_ns"):
        phases.append({"name": "play_stop", "start_unix_ns": metrics["stop_requested_unix_ns"],
                       "end_unix_ns": metrics["stop_ready_unix_ns"]})
    phase_markers = {"play_start": ("play_requested", "play_ready"),
                     "play_first_game": ("play_requested", "game_visible"),
                     "play_stop": ("stop_requested", "stop_ready")}
    for phase in phases:
        if phase["name"] in phase_markers:
            begin, finish = phase_markers[phase["name"]]
            phase["start_seconds"] = metrics.get(begin + "_seconds")
            phase["end_seconds"] = metrics.get(finish + "_seconds")
        monotonic = phase.get("start_seconds") is not None and phase.get("end_seconds") is not None
        start = phase["start_seconds"] * 1e6 + monotonic_offset_us if monotonic else phase["start_unix_ns"] / 1000 - epoch_us
        end = phase["end_seconds"] * 1e6 + monotonic_offset_us if monotonic else phase["end_unix_ns"] / 1000 - epoch_us
        seconds = (end - start) / 1e6
        spans = [s for s in trace["spans"] if start <= s["start_us"] < end]
        phase_counters = [c for c in counters if start <= c["timestamp_us"] < end]
        zones = defaultdict(list)
        for span in spans:
            zones[(span["category"], span["name"])].append(span["duration_us"] / 1000)
        totals = defaultdict(float)
        for counter in phase_counters:
            totals[counter["name"]] += counter["value"]
        presents = sorted(s["start_us"] + s["duration_us"] for s in spans
                          if (s["category"], s["name"]) == ("render", "native_submit_present"))
        selected = {name: {"sum": value, "per_second": value / seconds}
                    for name, value in totals.items()
                    if any(word in name for word in (
                        "layout_pass_count", "redraw_full_frame", "present_count",
                        "submit_count", "submitted_count", "copy_bytes", "runtime_tick_count",
                        "frame_update_count", "coalesced_present_count", "damage"))}
        activity_values = defaultdict(list)
        activity_presents = []
        for timestamp, name, value in activity:
            elapsed = elapsed_for_unix(timestamp) if monotonic else None
            in_phase = phase["start_seconds"] <= elapsed < phase["end_seconds"] if elapsed is not None else phase["start_unix_ns"] <= timestamp < phase["end_unix_ns"]
            if in_phase:
                activity_values[name].append(value)
                if name == "PresentCount" and value > 0:
                    activity_presents.append(timestamp)
        activity_presents.sort()
        activity_counts = {
            name: {"sum": sum(values), "per_second": sum(values) / seconds,
                   "observations": len(values)}
            for name, values in activity_values.items() if not name.endswith("Us")}
        input_latency = {name: distribution([value / 1000 for value in values])
                         for name, values in activity_values.items() if name.endswith("Us")}
        span_overwritten = any(r.get("spans", {}).get("overwritten", 0) for r in retention)
        first_span = min((s["start_us"] for s in trace["spans"]), default=None)
        result["phases"].append({"name": phase["name"], "seconds": seconds,
            "phase_clock": "monotonic" if monotonic else "Unix timestamp",
            "wall_minus_monotonic_seconds": (phase["end_unix_ns"] - phase["start_unix_ns"]) / 1e9 - seconds,
            "span_coverage_complete": not span_overwritten or (first_span is not None and start >= first_span),
            "counter_coverage_complete": not counter_overwritten or
                (first_counter is not None and start >= first_counter),
            "span_count": len(spans), "present_attempts_from_zones": len(presents),
            "present_end_gap_ms": distribution([(b - a) / 1000 for a, b in zip(presents, presents[1:])])
                if phase["name"] not in ("idle", "interaction") else None,
            "counters": selected,
            "activity_trace_available": bool(activity),
            "present_end_gap_ms_from_activity": distribution(
                [(b - a) / 1e6 for a, b in zip(activity_presents, activity_presents[1:])])
                if phase["name"] not in ("idle", "interaction") else None,
            "activity_counters": activity_counts,
            "input_latency_ms": input_latency,
            "top_inclusive_zones_ms": [{"category": category, "name": name,
                **distribution(values)} for (category, name), values in
                sorted(zones.items(), key=lambda item: sum(item[1]), reverse=True)[:25]],
            "native_dispatch_ms": distribution(zones.get(("input", "native_event_dispatch"), [])),
            "layout_passes_from_zones": len(zones.get(("retained_host", "workbench_surface_compute_layout"), []))})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    result = summarize(args.evidence.resolve())
    destination = args.evidence / "profile-summary.json"
    destination.write_text(json.dumps(result, indent=2) + "\n")
    print(destination)


if __name__ == "__main__":
    main()
