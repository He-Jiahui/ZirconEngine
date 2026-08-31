# Runtime 15 button render owner split

## Scope

- Target: `zircon_runtime/src/ui/surface/render/buttons.rs`.
- Baseline: clean 772-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 15, and the existing runtime button behavior suite.
- This slice changes source ownership only. It does not alter control semantics, claim a render-time or power improvement, or close UI/runtime acceptance.

## Architecture review

The previous file mixed six independently changing responsibilities: component identity and authored metadata decoding, painter-state folding, design-token/style projection, render-command encoding, ordinary button layout, and icon-button layout. The public extraction route only needs to reject unrelated components, resolve visual/state inputs, reject invalid frame extents, and dispatch to the concrete control owner.

The primary local Unreal reference was `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SButton.h`. Unreal keeps button style, content, padding, interaction state, and paint responsibilities as explicit concepts rather than one render helper. Zircon retains its immediate render-command model, but adopts the same responsibility boundaries.

The repository's retained-host renderer already separates ordinary template buttons into command, identity/content, style, and surface owners, with icon-button geometry and presentation owned separately. The runtime split follows that established project direction without routing runtime rendering through editor code or adding a compatibility facade.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `buttons.rs` | Public extraction route, frame admission, state/visual resolution, and concrete dispatch | 63 |
| `buttons/metadata.rs` | Component identity, variant/label/icon metadata, typed metrics, and CSS color parsing | 160 |
| `buttons/state.rs` | Component/painter state folding and read-only state queries | 104 |
| `buttons/style.rs` | Cached design-token projection, authored overrides, and state-dependent colors | 292 |
| `buttons/commands.rs` | Quad/text/icon command encoding and RGBA serialization | 116 |
| `buttons/button.rs` | Ordinary button icon/text geometry and command construction | 75 |
| `buttons/icon_button.rs` | Icon-button geometry and command construction | 44 |

The already modified parent `ui/surface/render/mod.rs` and extraction consumer were not touched. External imports continue to use the same `buttons` module and the three existing `pub(super)` entry points.

## Behavior invariants

- Non-button nodes still exit before visual and painter-state resolution.
- Invalid frame extents still exit before dynamic state folding and command allocation.
- Button variant classification still runs once per admitted control and uses borrowed ASCII-insensitive matching without a joined lowercase allocation.
- `EditorDesignTokens`, `EditorTypographyTokens`, authored `style_overrides`, and the cached default visual remain the visual authority.
- Unavailable, icon selected/pressed/hot, ordinary pressed/selected/hot, and normal color precedence is unchanged.
- Existing z-order, frame arithmetic, icon/label extraction, painter family/state, clipping, and opacity are unchanged.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the root, six child owners, and the adjusted behavior/source-contract test owner.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static migration comparison found all 27 old free-function definitions in the new owner family with no missing or duplicate definition.
- Static source checks found one production `button_kind(metadata)` call, zero lowercase/join allocation patterns, all six child mounts, and all design-token/style-parser hooks.
- Root size changed from 772 to 63 lines; all production owners are at or below 292 lines.
- Managed Cargo and live render verification were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because the rendering algorithm and command count were intentionally unchanged. Any later button hotpath optimization requires a focused CPU allocation/command-build baseline and rendered-output parity capture before implementation.
