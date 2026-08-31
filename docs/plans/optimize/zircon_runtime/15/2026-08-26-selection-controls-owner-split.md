# Runtime 15 selection controls render owner split

## Scope

- Target: `zircon_runtime/src/ui/surface/render/selection_controls.rs`.
- Baseline: clean 829-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 15, and the existing runtime selection-control behavior suite.
- This slice changes source ownership only. It does not alter checkbox/radio/toggle behavior, claim a render-time or power improvement, or close UI/runtime acceptance.

## Architecture review

The previous file combined component identity and metadata decoding, dynamic state folding, design-token/style projection, shared geometry, command encoding and labels, and three concrete control renderers. The top-level extraction route only needs to classify the control, resolve its visual and dynamic state, reject invalid frame extents, and dispatch to the concrete owner.

The primary local Unreal reference was `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SCheckBox.h`. Unreal exposes checkbox type, checked state, style, content layout, and concrete visual rebuilding as distinct concepts. Zircon's checkbox/radio/toggle controls share a state/style contract but have different marker, dot, track, and thumb construction, so each concrete renderer now has an independent leaf owner.

The repository's retained-host selection renderer already separates identity, style, commands/labels, and checkbox/radio/toggle owners. The runtime split follows that project-established shape without routing runtime rendering through editor code or introducing a compatibility facade.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `selection_controls.rs` | Classification-first public extraction route and concrete dispatch | 63 |
| `selection_controls/metadata.rs` | Component identity, painter family, label, metric, boolean, and CSS color parsing | 120 |
| `selection_controls/state.rs` | Checked/selected/disabled/painter-state folding and read-only state queries | 92 |
| `selection_controls/style.rs` | Cached design-token projection, authored overrides, and state-dependent colors | 268 |
| `selection_controls/geometry.rs` | Shared mark/label/track/thumb/dot frame calculations | 83 |
| `selection_controls/commands.rs` | Label emission, quad/text command encoding, and RGBA serialization | 123 |
| `selection_controls/checkbox.rs` | Checkbox surface, tick marker, and label command construction | 97 |
| `selection_controls/radio.rs` | Radio surface, dot marker, and label command construction | 62 |
| `selection_controls/toggle.rs` | Toggle label, track, and thumb command construction | 62 |

The already modified parent `ui/surface/render/mod.rs` and `extract.rs` were not touched. External imports continue to use the same module and two existing `pub(super)` entry points.

## Behavior invariants

- Non-selection nodes still exit before visual and painter-state resolution.
- Invalid frame extents still exit before dynamic state folding and command allocation.
- Checked/selected/disabled precedence and painter-family resolution remain unchanged.
- Focused-only controls keep neutral surfaces; hover/drag/drop surface-hot input only affects the prior eligible paths.
- Active checkbox/toggle visuals remain dominant over pressed/hover visuals, and active radio fill/border/dot semantics are unchanged.
- Existing mark/label/track/thumb geometry, tick construction, z-order, clipping, opacity, and command styles are unchanged.
- `EditorDesignTokens`, `EditorTypographyTokens`, authored `style_overrides`, validated CSS colors, and finite metrics remain the visual authority.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the root, eight child owners, and the adjusted behavior/source-contract test owner.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static migration comparison found all 43 old function definitions in the new owner family; two read-only state accessors were added for sibling ownership.
- Root classification-before-state, design-token/parser hooks, legacy-constant exclusions, and the 90-line root budget passed their static checks.
- Root size changed from 829 to 63 lines; all production owners are at or below 268 lines.
- Production `panic!`, `unwrap`, `expect`, and `allow(dead_code)` anchors remain absent.
- Managed Cargo and live render verification were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because the rendering algorithm and command sequence were intentionally unchanged. Any later selection-control hotpath optimization requires a focused CPU allocation/command-build baseline and rendered-output parity capture before implementation.
