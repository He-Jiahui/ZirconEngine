# Runtime 15 segmented controls render owner split

## Scope

- Target: `zircon_runtime/src/ui/surface/render/segmented_controls.rs`.
- Baseline: clean 838-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 15, and the existing runtime segmented-control behavior suite.
- This slice changes source ownership only. It does not alter control semantics, claim a render-time or power improvement, or close UI/runtime acceptance.

## Architecture review

The previous file mixed six independently changing responsibilities: component identity and metadata decoding, dynamic painter-state folding, design-token/style projection, render-command encoding, segmented-group rendering, and tab rendering. The public extraction route only needs to classify the component, resolve its visual/state inputs, reject invalid frame extents, and dispatch to the concrete owner.

The primary local Unreal reference was `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SSegmentedControl.h`. Unreal keeps style input, option/selection state, child rebuilding, and per-segment checked presentation as explicit concepts rather than one painter helper. `SDockTab.h` similarly keeps tab behavior/style ownership separate from the surrounding docking route. Zircon retains its immediate render-command model, but adopts the same responsibility boundaries.

The repository's retained-host segmented control already uses the corresponding `identity/options/style/segments/tabs/commands` owner shape. The runtime split follows that established project pattern without routing runtime rendering through editor code or introducing a compatibility facade.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `segmented_controls.rs` | Classification-first public extraction route and concrete dispatch | 57 |
| `segmented_controls/metadata.rs` | Component identity, options/selection/labels, typed metadata and CSS color parsing | 169 |
| `segmented_controls/state.rs` | Component/painter state folding and read-only state queries | 78 |
| `segmented_controls/style.rs` | Cached design-token projection, authored overrides, and state-dependent colors | 292 |
| `segmented_controls/commands.rs` | Quad/text command encoding and RGBA serialization | 86 |
| `segmented_controls/segments.rs` | Segmented body, option geometry, selection surface, underline, divider, and text commands | 211 |
| `segmented_controls/tabs.rs` | Tab surface, selected underline, and label commands | 77 |

The already modified parent `ui/surface/render/mod.rs` was not touched. External imports continue to use the same `segmented_controls` module and the two existing `pub(super)` entry points.

## Behavior invariants

- Non-segmented nodes still exit before visual and painter-state resolution.
- Invalid frame extents still exit before dynamic state folding and command allocation.
- Option matching remains borrowed `eq_ignore_ascii_case` and does not allocate a lowercase copy.
- `EditorDesignTokens`, `EditorTypographyTokens`, authored `style_overrides`, and the cached `default_segmented_visual` remain the visual authority.
- Focused-only surfaces remain neutral until a real hover/open/drag/drop surface-hot state is present.
- Existing z-order, frame arithmetic, labels, dividers, selected surface/border/underline, painter family/state, clipping, and opacity are unchanged.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the root, six child owners, and the adjusted behavior/source-contract test owner.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static migration comparison found all 42 old function definitions in the new owner family; three read-only state accessors were added for sibling ownership.
- Root classification-before-state, no-lowercase-copy, and four design-token hooks passed their static checks.
- Root size changed from 838 to 57 lines; all production owners are at or below 292 lines.
- Managed Cargo and live render verification were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because the rendering algorithm and command count were intentionally unchanged. Any later segmented-control hotpath optimization requires a focused CPU allocation/command-build baseline and rendered-output parity capture before implementation.
