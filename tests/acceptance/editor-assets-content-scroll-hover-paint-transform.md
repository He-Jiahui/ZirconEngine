# Editor Assets Content Scroll/Hover Paint Transform Acceptance

## Scope

This record accepts only Editor Layout slices S15.4ox/S15.6nw, S15.4oy/S15.6nx, S15.4oz/S15.5n/S15.6ny, and S15.4pa/S15.5o/S15.6nz. It covers retained Activity/Browser interaction state, the generic template-node paint transform, Assets Activity source geometry/scroll/clip/hover projection, the shared scrollbar, production native pointer routing, move arbitration, and the scrolled/hovered screenshot route. It does not accept the wider Asset Browser painter, overlay drawers, all editor windows, or full Unreal visual parity.

## Baseline

Code review found that the first screenshot fixture injected host scroll/hover setters directly and that production routing never constructed `PanePointerTarget::AssetContent`. The routing RED returned whole-pane damage `(34,104,230,350)` instead of the content viewport `(34,168,230,170)`. After adding the production route, a second RED reported `template_control=WorkbenchScenePlayerItem` at the intended asset-row point, proving base Workbench template move handling still preempted pane semantics.

The accepted architecture resolves `AssetsActivityContentPanel` before generic template fallback and uses panel-local `AssetContent("activity")` coordinates. Top-level move order is native menu, Workbench popup row, pane target, then base Workbench template. This keeps popup priority while allowing asset content callback/writeback and paint projection to share the same relative geometry.

## Test Inventory

- Interaction setters: 2/2 passed.
- Generic template-node transform: 2/2 passed.
- Activity content projector/root/empty/scrollbar group: 7/7 passed.
- Short-drawer source geometry: 2/2 passed.
- Shared asset-content pointer bridge: 1/1 passed.
- Open Workbench dropdown/popup hit priority: 2/2 passed.
- Assets Activity group: 9/9 passed.
- Native wheel/move/content-only repaint/Preview-integrity regression: 1/1 passed in 55.30s.
- Ignored scrolled/hovered capture: 1/1 passed in 47.52s.

The full compiled editor test binary was also run with `--test-threads=1`: 2761 passed, 133 failed, 34 ignored, 0 filtered, in 1291.13s. Every test listed above passed inside that full run. The failures are broader current-worktree/baseline failures; representative fresh-process failures include an existing Workbench projection assertion (`0.0` versus `12.22`), a Scene context-menu hit resolving to `WorkbenchEffectAssetSearch`, runtime HUD glyph capture with zero changed pixels, and plugin optional-feature catalog drift.

## Tooling Evidence

- Official `validate-matrix.ps1` locked package build passed in 4m37s.
- The validator's default-parallel test process created thousands of threads and exited 101 without emitting a test result. It is not counted as a passing full-suite gate.
- WSL `cargo check -p zircon_editor --lib --no-default-features --locked --offline --jobs 1 --message-format short --color never` passed in 19m13s.
- Scoped Rustfmt and scoped `git diff --check` passed. Full workspace Rustfmt remains blocked by unrelated formatting deltas in current `zircon_runtime/src/tests/runtime_absorption` fixtures; the first reported owner is `core_spine_root_generated/split_layout.rs`, followed by several structure-convention production-file-budget tests.
- Added-production-line scans found no local raw RGB/hex colors, concrete font family, compatibility alias, or absolute-positioning addition. New/touched source owners remain below the repository large-file threshold.
- Plan-output audit reports 23 existing violations outside `docs/plans/zircon_editor/editor_layout` and zero new editor-layout violations.

## Results

The accepted artifact is `docs/tests/editor/editor-window-m3-assets-drawer-scrolled-hover-900x620.png`, 900x620, 74069 bytes, SHA256 `45359A656E5EEBADA47685E526C790C965BD2167724C3C85053A17056DC23533`. Manual inspection confirms later rows including `unit.zshader` and `player_start.prefab`, one hovered visible row, a non-zero shared scrollbar thumb position, and content clipped above Preview. The pixel contract requires more than 40 changed interior pixels in the target row, zero changed pixels in every other visible row, and zero changed pixels in Preview.

Matching-filename scans report zero copies in `E:\Git\ZirconEngine\target` and zero copies in the external validator target. The PNG exists only under `docs/tests/editor`.

## Acceptance Decision

Accepted for the four scoped Editor Layout slices. The production route, callback/writeback, relative paint transform, hover, clipping, scrollbar, and screenshot evidence are complete and pass their focused and full-context tests. The broader editor-layout goal remains active because Asset Browser paint projection, overlay drawer interaction, remaining composite/window adaptation, the 133 current full-suite failures, and complete Unreal-level visual parity are outside this acceptance boundary.
