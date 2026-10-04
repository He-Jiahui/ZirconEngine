---
name: capture-hub-window-screenshot
description: Capture and verify the running Zircon Hub window on Windows for visual acceptance.
---

# Capture Hub Window

Use a Hub binary matching the changed source and built under an approved drive-root Cargo target. Reuse a matching build; obtain a scoped build through `tools/dev/local-cargo.ps1` only when needed.

Read [the capture workflow and options](guide.md) for the affected state. Verify that screenshot, profile, log, and child-process outputs fit the current write grants before launching a helper; repository `target` is not an approved fallback.

| Required evidence | Helper under `scripts/` |
| --- | --- |
| One page, popup, or feedback state | `capture-hub-window.ps1` |
| Projects navigation and detail actions | `capture-hub-project-pages.ps1` |
| A declared set of other UI states | `capture-hub-visual-state-matrix.ps1` |
| A required complete reference comparison | `compare-hub-tauri-references.ps1` |

Acceptance requires the real native window titled `Zircon Hub`, valid bounds, state-specific WebView text, and a rendered PNG from WebView2 DevTools. Open the returned image with `view_image` at original detail. A helper window, stale page, black WebView surface, or failed navigation delta does not satisfy the gate.

Capture only the states affected by the task unless its acceptance gate requires the complete matrix. Run capture helpers sequentially; they share foreground state. Let Hub own window sizing and use stable text actions for navigation.
