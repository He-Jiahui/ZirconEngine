# Runtime 15 chrome render owner split

## Scope

- Target: `zircon_runtime/src/ui/surface/render/chrome.rs`.
- Baseline: clean 702-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 15, and the existing runtime chrome behavior suite.
- This slice changes source ownership only. It does not alter chrome semantics, claim a render-time or power improvement, or close UI/runtime acceptance.

## Architecture review

The previous file mixed six independently changing responsibilities: chrome identity and authored metadata decoding, painter-state folding, design-token/color policy, metric and separator/content geometry, render-command encoding, and the concrete command sequence. The public extraction route only needs to classify a chrome component, reject invalid frame extents, resolve state/metrics, and dispatch to the concrete owner.

The primary local Unreal references were `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SBorder.h`, `Widgets/Docking/SDockTab.h`, and `Widgets/SViewport.h`. Slate keeps border appearance, foreground/background, padding/content, concrete dock content, and viewport content as separate widget/style concepts around the paint boundary. Zircon retains its immediate render-command model, but adopts the same responsibility boundaries.

The repository's retained-host renderer already owns workbench chrome through a folder-backed `workbench_chrome` family with `model`, `palette`, `fill`, `selection`, and `separators` leaves, while command streaming remains separate. The runtime split follows that established project direction without routing runtime rendering through editor code or adding a compatibility facade.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `chrome.rs` | Public extraction route, classification, frame admission, state/metric resolution, and dispatch | 62 |
| `chrome/metadata.rs` | Chrome kind, label/icon identity, authored colors/strings, and typed numeric decoding | 103 |
| `chrome/state.rs` | Painter-state folding and read-only state queries | 61 |
| `chrome/style.rs` | Cached design-token palette, authored overrides, and state-dependent surface/border/content colors | 212 |
| `chrome/metrics.rs` | Cached density/typography projection, separator parsing, and separator/content geometry | 175 |
| `chrome/commands.rs` | Surface/separator/text/icon command encoding | 135 |
| `chrome/content.rs` | Concrete surface, separator, icon, and label command sequence | 73 |

The already modified parent `ui/surface/render/mod.rs` and extraction consumer were not touched. External imports continue to use the same `chrome` module and the four existing `pub(super)` entry points.

## Behavior invariants

- Non-chrome nodes still exit before painter-state and metric resolution.
- Invalid frame extents still exit before dynamic state folding and command allocation.
- Component classification remains before dynamic state resolution, including the existing control-id fallback rules.
- `EditorDesignTokens`, authored `style_overrides`, and cached palette/metrics remain the visual authority.
- Disabled/loading, pressed, open, hovered, selected, and normal surface precedence is unchanged; focused chrome still keeps the normal surface while using its active border/content treatment.
- Default separator edges, frame arithmetic, surface/separator/icon/text command order, z-index, painter family/state, clipping, and opacity are unchanged.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the root, six child owners, and the adjusted behavior/source-contract test owner.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static migration comparison found all 32 old free-function definitions in the new owner family; the two new free functions name the concrete command assembly and separator color policy.
- Static source checks found zero lowercase-allocation patterns, all six child mounts, classification before state resolution, and all design-token/style hooks.
- Root size changed from 702 to 62 lines; all production owners are at or below 212 lines.
- Managed Cargo and live render verification were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because the rendering algorithm and command count were intentionally unchanged. Any later chrome hotpath optimization requires a focused CPU allocation/command-build baseline and rendered-output parity capture before implementation.
