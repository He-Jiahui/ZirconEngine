---
name: capture-hub-window-screenshot
description: Use when validating Zircon Hub visual output, comparing Hub UI against screenshots, or needing a real Windows desktop capture of the running Tauri Hub window instead of static source inspection.
---

# Capture Hub Window Screenshot

## Overview

Capture the actual `zircon_hub.exe` window on Windows for UI verification. Use the bundled PowerShell scripts so future checks reuse the verified window enumeration, title/bounds checks, WebView2 DevTools screenshot path, and popup/action text gates.

The scripts still launch and validate a real Tauri window titled `Zircon Hub`. Pixel capture uses the WebView's internal DevTools `Page.captureScreenshot` output because Windows desktop `CopyFromScreen` can return a black WebView2 composition surface in automated sessions. Treat a capture as valid only when the real native window gate and the WebView-rendered PNG both pass.

## Workflow

1. If UI code changed, rebuild the Hub binary first:

   ```powershell
   cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1
   ```

2. Run the capture script from the repository root:

   ```powershell
   .\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\capture-hub-window.ps1
   ```

3. Open the returned PNG path with `view_image` using `detail: original`, then compare against the reference image.

For Projects secondary-page checks, run the multi-page script instead:

```powershell
.\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\capture-hub-project-pages.ps1
```

It seeds an isolated Projects profile by default and writes stable dashboard, New Project, Project Browser, and Project Detail screenshots under `target/hub-visual-check`. The seeded recent-project paths and project metadata keys must point at the real isolated project directories created under the capture config root, so Project Detail actions that check the filesystem, including guarded delete confirmation, exercise the same existing-path path as a developer profile.
The multi-page script also writes stdout/stderr logs beside the screenshots, checks the Hub process after each navigation step, compares each post-action screenshot against the previous page, uses a harmless focus click before coordinate-based navigation, and validates the real Hub window before every WebView screenshot, so a Tauri startup failure, stale window handle, missed click, or foreground-window race fails with a concrete stage name instead of silently producing a misleading image. It captures Dashboard/New Project and Project Browser/Project Detail in separate Hub sessions, avoiding false captures caused by returning from one secondary page into another while Windows is still repainting or another topmost window overlaps the capture rectangle.
For Project Detail, the script opens the seeded Elysium row through a WebView action keyed by the visible project/detail text, requires a larger page-change difference than hover-only movement can satisfy, and retries through the project text path if the image did not change enough; this covers Windows frame/client-coordinate drift without accepting a stale Browser screenshot.
The script seeds the requested window size through Hub config and then only moves/topmosts the actual Tauri window. Do not resize the native window with Win32 after launch; that can desynchronize the outer frame from the WebView render surface and create misleading black bands.

For non-Projects state coverage, run the visual state matrix script:

```powershell
.\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\capture-hub-visual-state-matrix.ps1
```

It writes isolated Editor, Assets, Builds, Plugins, Cloud, Team, Learn, Settings, Source Engine popup, User menu, empty Project Browser, loading, and error-state screenshots under `target/hub-visual-check/tauri-visual-state-matrix`. Source Engine and user-menu captures click the topbar controls by WebView text and require popup-only text such as `Manage engines`/`管理引擎` or `Preferences`/`偏好设置` before saving, so a focused-but-closed topbar button cannot pass as popup evidence. Loading and error captures use the Hub runtime's `ZIRCON_HUB_VISUAL_TASK_STATE` diagnostic override, so the React UI still receives a real Tauri `hub_state` view-model and renders the shared `HubStatusBanner`/`HubSnackbar` feedback components instead of page-local mock markup.
The matrix seeds English runtime config and every capture must require state-specific WebView text before saving, such as `Launch Target`, `Assets Catalog`, `Build Workflow`, `Package Outputs`, `No projects found`, `Loading Hub state`, or the popup labels `Manage engines` and `Preferences`. This text gate prevents the initial localized fallback first paint from being accepted as a later backend-projected page state.

After the Project pages and visual state matrix are captured, run the Hub Tauri reference comparison script:

```powershell
.\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\compare-hub-tauri-references.ps1
```

It compares the real Tauri Dashboard plus all 19 exported reference pages/states against the HTML/CSS-finalized `docs/ui-and-layout` PNG references, writes `hub-tauri-reference-comparison.json` plus `hub-tauri-reference-comparison.md`, and checks that the matching AI draft PNGs are present where the manifest defines them. The comparison gate rejects missing captures, small non-Hub windows, mostly white captures, and low dynamic-range captures; similarity deltas are reported as mean/RMS/change-ratio metrics so visual drift can be reviewed without introducing page-by-page pixel patches.

## Script Options

- Default output: `target/hub-visual-check/hub-actual-<timestamp>.png`.
- Default config mode: `Isolated`, which redirects `LOCALAPPDATA`, `APPDATA`, and `ZIRCON_CONFIG_PATH` under `target/hub-visual-check/config`.
- Use `-ConfigMode Current` when the visual check must use the developer's real Hub profile or existing recent projects.
- Use `-OutputPath <path>` for a stable screenshot filename.
- Use `-ClickX <x> -ClickY <y>` with `capture-hub-window.ps1` to click a window-relative point before capture, useful for dropdowns and popups.
- Use `-SecondClickX <x> -SecondClickY <y>` only for short two-step state captures where the first click navigates to the target surface and the second click triggers the state, such as Builds loading/error screenshots. Longer Projects page flows still belong in the Projects multi-page script.
- Use `-WebViewClickText <text>` with `capture-hub-window.ps1` when a topbar/menu/dialog action has stable visible text or an accessible label. Use `-RequireWebViewText <text>` to require specific rendered text before the screenshot; separate language alternatives with `|||`, for example `Preferences|||偏好设置`.
- Use `-WindowWidth <w> -WindowHeight <h>` with isolated `capture-hub-window.ps1` runs when a page requires a taller or wider viewport. The script writes those dimensions into the isolated Hub TOML config before launch, keeps the Editor JSON config separate, redirects stdout/stderr beside the PNG, and only moves the native window before capture.
- Use `-VisualTaskState loading|running|warning|error|success` with `capture-hub-window.ps1` when a stable feedback screenshot is needed. This sets `ZIRCON_HUB_VISUAL_TASK_STATE` only for the launched Hub process and restores the previous environment value afterward.
- Use `-RequireWindowTitle "Zircon Hub"` when a script must reject helper/console windows. The visual state matrix enables this by default.
- Use `-PinnedProjectCount <n>` with `capture-hub-project-pages.ps1` to seed pinned Hub metadata in the isolated profile and verify the Pinned Projects projection.
- Use `-CapturePendingDelete` with `capture-hub-project-pages.ps1` to click the Project Detail delete action once and capture the guarded confirmation state as `hub-projects-detail-delete-confirm.png`. The script scrolls the visible `Delete Project`/`删除项目` action into view through WebView text matching, saves the scrolled baseline as `hub-projects-detail-delete-ready.png`, then clicks the same localized action and compares the confirmation capture against that baseline so scroll movement alone cannot pass the assertion. Confirmation must produce a stronger image delta than a hover/selection-only change. Keep `-DeleteClickX/-DeleteClickY` and `-DeleteScrollNotches` only for legacy coordinate diagnostics; the accepted path should use the text-gated WebView action.
- Use `-CaptureBrowserMenus` with `capture-hub-project-pages.ps1` to open Project Browser's filter and sort select menus in separate stable Hub sessions and capture `hub-projects-browser-filter-menu.png` plus `hub-projects-browser-sort-menu.png`. These sessions seed the runtime directly into Project Browser/list mode before opening the menu, so dropdown verification does not depend on re-clicking the Dashboard list-view affordance. Override the `-BrowserFilterMenuClickX/Y` or `-BrowserSortMenuClickX/Y` values only when the Browser toolbar moves enough that the defaults no longer land inside the select controls.
- Project Browser menu captures now reject any saved window whose title is not `Zircon Hub` or whose dimensions are less than 90% of the requested Hub window size. This catches native file pickers, popup-only windows, and stale helper windows before they can be accepted as browser-menu evidence.
- Browser menu captures use a small but nonzero screenshot-difference gate because the dropdown only covers a narrow part of the full 1568x1003 frame. A stale capture must still fail at zero difference, while a visible opened menu can pass without requiring unrelated page movement.
- Use `-NewProjectClickX/-NewProjectClickY` and `-BrowserClickX/-BrowserClickY` only when the Projects layout has moved enough that the responsive defaults no longer land on the intended controls. The multi-page script still records `-DetailClickX/-DetailClickY` for diagnostics, but the accepted Project Detail path opens the row through WebView text matching instead of a fragile table-coordinate click.
- Use `-LeaveOpen` only when manual interaction is needed after capture.

Example:

```powershell
.\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\capture-hub-window.ps1 `
  -ConfigMode Current `
  -OutputPath target\hub-visual-check\hub-current-profile.png
```

## Rules

- Do not rely on `MainWindowHandle`; Hub can expose a tiny helper window before the real Slint surface.
- Prefer the top-level window titled `Zircon Hub`; only use the largest visible fallback for explicit exploratory captures. The matrix and Projects scripts should fail instead of accepting a helper/console fallback.
- Do not use a PowerShell variable named `$PID`; it is read-only.
- Move the window to a known visible coordinate and make it topmost before WebView screenshot capture.
- Reapply foreground/topmost state before every capture, not only once after launch; Windows can place another window above the Hub between navigation steps.
- Validate captured window bounds before allocating the screenshot bitmap; zero or negative window sizes should be reported as script/window state failures.
- Repeat only clicks that remain on the same intended control after navigation. Do not double-click dashboard controls that open a secondary page, because the second click can land on the new page and open focus-driven controls such as the Material `SearchBar`.
- Do not run multiple Hub screenshot scripts in parallel at the same screen coordinates. They will compete for foreground/topmost state and can produce misleading navigation captures.
- Clamp only the window position to the visible screen before capture; let Hub/Slint own the actual window size.
- Never treat black bands from native-window/render-surface desynchronization as UI evidence.
- Never treat a desktop-copied black WebView2 surface as UI evidence; use the DevTools screenshot helper after the real Tauri window title and bounds have been validated.
- Treat an isolated capture that shows no recent projects as valid evidence of empty-state rendering, not evidence that the reference project-card layout is correct.
- Treat a multi-page script failure that says the capture did not change enough as a real navigation validation failure. Fix the click target or UI route before trusting the labeled PNG.
- The Projects multi-page script computes default navigation clicks from the requested window size. Current Material-layout defaults target the New Project button around 84% of width on compact/medium captures and around 82% on wide captures, the dashboard list-view affordance for Project Browser navigation at each breakpoint, and the first browser row's trailing detail action in the table column. Compact captures keep the detail target near the right edge because the side panels stack; medium and wide captures bias the detail target to the left table action column so the click does not hit the right-side Quick Actions panel. Browser row body clicks are reserved for row selection, matching the Hub project-browser interaction plan. If the first detail click lands on an unstable edge, the script retries once farther inside that same hit zone before failing. Adjust the matching `*-ClickX/Y` parameter only if a future layout moves those targets again.
