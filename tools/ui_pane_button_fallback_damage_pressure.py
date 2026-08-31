#!/usr/bin/env python3
"""Model the damage amplification of the pane-button fallback path.

This is a deterministic geometry/source-contract model, not product timing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from dataclasses import dataclass
from pathlib import Path, PureWindowsPath


ENTRY_SOURCE = Path(
    "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
    "pane_button_damage/entry.rs"
)
CALLER_SOURCE = Path(
    "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
    "button_dispatch/pane_callbacks/fallback.rs"
)


@dataclass(frozen=True)
class Rect:
    x: float
    y: float
    width: float
    height: float

    @property
    def right(self) -> float:
        return self.x + self.width

    @property
    def bottom(self) -> float:
        return self.y + self.height

    @property
    def area(self) -> float:
        return self.width * self.height


def _positive_finite(name: str, value: float) -> float:
    if not math.isfinite(value) or value <= 0:
        raise ValueError(f"{name} must be positive and finite")
    return value


def _union(left: Rect, right: Rect) -> Rect:
    x = min(left.x, right.x)
    y = min(left.y, right.y)
    right_edge = max(left.right, right.right)
    bottom = max(left.bottom, right.bottom)
    return Rect(x, y, right_edge - x, bottom - y)


def _union_all(rects: list[Rect]) -> Rect:
    if not rects:
        raise ValueError("at least one rectangle is required")
    result = rects[0]
    for rect in rects[1:]:
        result = _union(result, rect)
    return result


def pressure_report(
    viewport_width: float = 1920.0,
    viewport_height: float = 1080.0,
    top_chrome_height: float = 48.0,
    status_height: float = 32.0,
    pane_width: float = 320.0,
    button_width: float = 32.0,
    button_height: float = 32.0,
) -> dict[str, object]:
    viewport_width = _positive_finite("viewport_width", viewport_width)
    viewport_height = _positive_finite("viewport_height", viewport_height)
    top_chrome_height = _positive_finite("top_chrome_height", top_chrome_height)
    status_height = _positive_finite("status_height", status_height)
    pane_width = _positive_finite("pane_width", pane_width)
    button_width = _positive_finite("button_width", button_width)
    button_height = _positive_finite("button_height", button_height)
    if top_chrome_height + status_height >= viewport_height:
        raise ValueError("top chrome and status bar must leave a positive center band")
    if pane_width > viewport_width:
        raise ValueError("pane_width must not exceed viewport_width")

    center_height = viewport_height - top_chrome_height - status_height
    pane = Rect(0.0, top_chrome_height, pane_width, center_height)
    center = Rect(0.0, top_chrome_height, viewport_width, center_height)
    status = Rect(
        0.0,
        viewport_height - status_height,
        viewport_width,
        status_height,
    )
    button = Rect(16.0, top_chrome_height + 16.0, button_width, button_height)

    # Current source contributes one pane, two center-band, and three status-bar
    # rectangles. Duplicate domains do not change the union, but still document
    # why the fallback is deliberately conservative.
    current_sources = [pane, center, center, status, status, status]
    current_bounds = _union_all(current_sources)
    typed_regions = [button, status]
    typed_bounds = _union_all(typed_regions)
    typed_represented_area = sum(rect.area for rect in typed_regions)

    return {
        "schema": "zircon.editor.ui_pane_button_fallback_damage_pressure.v1",
        "evidence_kind": "deterministic_geometry_and_source_contract",
        "is_product_timing": False,
        "inputs": {
            "viewport_width": viewport_width,
            "viewport_height": viewport_height,
            "top_chrome_height": top_chrome_height,
            "status_height": status_height,
            "pane_width": pane_width,
            "button_width": button_width,
            "button_height": button_height,
        },
        "current_fallback": {
            "source_union_count": len(current_sources),
            "damage_region_count": 1,
            "bounding_area_px": current_bounds.area,
            "represented_area_px": current_bounds.area,
            "viewport_area_ratio": current_bounds.area
            / (viewport_width * viewport_height),
        },
        "typed_action_target": {
            "damage_region_count": len(typed_regions),
            "bounding_area_px": typed_bounds.area,
            "represented_area_px": typed_represented_area,
            "viewport_area_ratio": typed_represented_area
            / (viewport_width * viewport_height),
            "required_receipts": [
                "affected control or pane region",
                "status text changed",
                "active/sibling pane semantic or chrome changed",
            ],
        },
        "comparison": {
            "avoided_represented_area_px": current_bounds.area
            - typed_represented_area,
            "represented_area_ratio": current_bounds.area / typed_represented_area,
            "semantic_correctness_policy": "fail_closed",
        },
        "interpretation": {
            "finding": (
                "the current unknown-effect Press fallback expands damage to the center "
                "band and status bar even when a typed action would affect only one button "
                "and status text"
            ),
            "implementation_gate": (
                "production damage may shrink only after callback dispatch publishes typed "
                "affected-region receipts; missing or ambiguous receipts retain the current "
                "conservative fallback"
            ),
            "excluded": [
                "CPU, allocator, RSS, GPU, and input-to-present timing",
                "actual callback action distribution",
                "occlusion, clip, draw-command, and backend batch costs",
                "proof that the representative geometry matches a product capture",
            ],
        },
    }


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def source_contract(repo_root: Path) -> dict[str, object]:
    entry_path = repo_root / ENTRY_SOURCE
    caller_path = repo_root / CALLER_SOURCE
    entry = entry_path.read_text(encoding="utf-8")
    caller = caller_path.read_text(encoding="utf-8")
    contract = {
        "pane_frame_unions": entry.count(
            "union_visible_frame(damage, pane_frame.clone())"
        ),
        "center_band_unions": entry.count("center_band_frame.clone()"),
        "status_bar_unions": entry.count("status_bar_frame.clone()"),
        "pressed_requests_frame_update": (
            "NativePointerDispatchResult::region_with_frame_update(damage)" in caller
        ),
        "release_is_pointer_local": (
            "state == NativePointerButtonState::Released" in caller
            and "NativePointerDispatchResult::region(pointer.frame.clone())" in caller
        ),
        "sources": {
            ENTRY_SOURCE.as_posix(): _sha256(entry_path),
            CALLER_SOURCE.as_posix(): _sha256(caller_path),
        },
    }
    expected = (1, 2, 3, True, True)
    observed = (
        contract["pane_frame_unions"],
        contract["center_band_unions"],
        contract["status_bar_unions"],
        contract["pressed_requests_frame_update"],
        contract["release_is_pointer_local"],
    )
    if observed != expected:
        raise RuntimeError(
            "pane-button fallback source contract drifted: "
            f"expected {expected}, observed {observed}"
        )
    return contract


def _git_head(repo_root: Path) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def build_payload(repo_root: Path) -> dict[str, object]:
    report = pressure_report()
    report["source_contract"] = source_contract(repo_root)
    report["git_head"] = _git_head(repo_root)
    return report


def validate_output_path(output: str) -> Path:
    windows_path = PureWindowsPath(output)
    if windows_path.drive.upper() == "C:":
        raise ValueError("performance artifacts must not be written to the C drive")
    path = Path(output).resolve()
    if PureWindowsPath(str(path)).drive.upper() == "C:":
        raise ValueError("performance artifacts must not be written to the C drive")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output")
    args = parser.parse_args()
    payload = json.dumps(build_payload(args.repo_root.resolve()), indent=2, sort_keys=True) + "\n"
    if args.output:
        output = validate_output_path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload, encoding="utf-8", newline="\n")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
