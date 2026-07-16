# Layout15 Material State Priority Convergence Design

## Status

- Date: 2026-07-16
- Decision: approved approach A
- Owner plan: `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- Target slices: S15.4ke/S15.6jf, S15.4kf/S15.6jg, S15.4kg/S15.6jh
- Implementation status: design only; production code remains unchanged until this specification is reviewed

## Problem

Layout15 already establishes the visual priority
`disabled > pressed > drag > focus > hover > default`, but the contract is expressed
through several independent condition chains:

- retained Material state-layer opacity is selected directly in
  `material_state_layer/state.rs`;
- runtime Material Button boolean fallback in `zircon_runtime/src/ui/style.rs`
  skips `dragging` and falls from `pressed` directly to `focused`;
- shared `UiPainterState` already resolves focused plus dragging Button input to
  the existing `Hover` interaction bucket because `ButtonInteractionState` has no
  public `Dragging` variant.

The result is an avoidable boundary drift: retained painting and the shared painter
preserve drag-over-focus semantics, while runtime style extraction may still resolve
the same boolean input as `Focused`.

## Goals

1. Make the interaction priority explicit and independently testable instead of
   relying on the order of unrelated `if` branches.
2. Keep retained Material state opacity compatible with the existing Slint-derived
   values: hover `0.08`, focus `0.10`, press `0.10`, and drag `0.16`.
3. Converge runtime Button boolean fallback with the shared `UiPainter` contract:
   focused plus dragging resolves to `ButtonInteractionState::Hover`, never
   `Focused`.
4. Preserve the existing Runtime Text measurement and glyph interfaces; this slice
   changes interaction state selection, not text shaping or rasterization.
5. Produce a dedicated four-state visual artifact only at
   `docs/tests/editor/editor-components-material-state-layer-900x360.png`.

## Non-goals

- Do not add a public `Dragging` variant to `ButtonInteractionState`.
- Do not introduce a compatibility facade, re-export shim, root painter branch, or
  Workbench-window special case.
- Do not change Runtime Text, Render18, M1.1 TextField/Dialog files, or unrelated
  palette tokens.
- Do not position production UI using screenshot coordinates or fixed window-size
  branches.
- Do not treat the new screenshot as proof of complete Layout15 or complete Material
  component coverage.

## Chosen Architecture

### Retained state resolver

Add a private `MaterialStateLayerResolvedState` enum beside the existing retained
state-layer implementation. Its variants are:

- `Disabled`
- `Pressed`
- `Dragging`
- `Focused`
- `Hovered`

Absence of a state is represented by `None`, which keeps the state layer disabled for
the default interaction. A private resolver maps `TemplatePaneNodeData` to the enum in
one explicit priority chain:

1. state layer disabled -> `None`;
2. disabled -> `Disabled`;
3. pressed or enter-pressed -> `Pressed`;
4. dragging -> `Dragging`;
5. focused, selected, or checked -> `Focused`;
6. hovered, drop-hovered, or active drag target -> `Hovered`;
7. otherwise -> `None`.

Opacity selection consumes this enum. The resolver owns priority; opacity mapping owns
only visual intensity. Disabled retains the current focus-strength opacity so this
slice does not silently redesign disabled Material visuals.

### Runtime Button fallback

Keep explicit authored string values as the highest-priority style input. Preserve the
existing loading, disabled, and pressed order. Add boolean `dragging` immediately after
pressed and map it to `ButtonInteractionState::Hover`, then continue with focused and
hovered.

This intentionally follows the shared `UiPainterState::button_interaction_state()`
folding contract. It does not expand the public interface solely to represent a visual
state that Button already renders through its hover bucket.

### Visual fixture

Add a dedicated retained component fixture that renders four comparable Material
state-layer samples from the real painter path:

- default/hovered;
- focused;
- pressed plus focused;
- dragging plus focused.

The fixture uses existing palette tokens, component metrics, runtime text measurement,
and relative row/column layout. The 900x360 output size is an artifact viewport only;
individual component positions derive from measured content, container insets, gaps,
and available width. Production layout code receives no screenshot-size condition.

## Data Flow

1. Authored `.zui`/runtime state supplies boolean interaction properties.
2. `TemplatePaneNodeData` carries the retained interaction flags.
3. `MaterialStateLayerResolvedState::resolve` selects one semantic state.
4. The opacity mapper combines that state with the existing host palette color.
5. Runtime stylesheet extraction applies the same priority and folds Button dragging
   to `Hover`.
6. The retained screenshot fixture exercises real text measurement, layout, painter,
   and image export, writing only under `docs/tests/editor`.

## Boundary and Error Handling

- Unknown or absent interaction booleans remain the default state and do not paint a
  state layer.
- Explicit string interaction values retain existing parsing behavior and override
  boolean fallback.
- A disabled state layer returns `None` before any interaction flag is considered.
- Screenshot export must fail rather than redirect to `target`; post-capture validation
  scans repository and managed target roots for the exact artifact name.
- Cross-plan compile errors are recorded as their owning plan's failure and do not
  justify a retained painter bypass.

## Testing Strategy

### Red tests first

1. Enum-resolution unit tests for each state and exact mixed-state priority:
   disabled over pressed, pressed over dragging, dragging over focus, and focus over
   hover.
2. Runtime style test with `focused=true`, `dragging=true`, and `hovered=true`, expecting
   `ButtonInteractionState::Hover`.
3. Retained component visual guard that samples the four fixture states and proves the
   drag/press samples are not rendered as focused-only.

### Green verification

- Run `rustfmt --edition 2021 --check` on touched Rust files.
- Run focused retained state-layer tests.
- Run focused runtime Material Button style tests.
- Run the component visual guard and ignored capture from a current-source managed
  Windows test binary.
- Verify the PNG SHA-256 and visually inspect state separation, text clarity, spacing,
  and rounded surfaces.
- Confirm the exact PNG name has zero matches under repository and managed target
  roots.
- Run scoped `git diff --check` and independent review before milestone closeout.

## Delivery and Milestone Boundary

This work is a new Layout15 slice after M1.1. It must use an exact manifest containing
only the resolver, runtime fallback, focused tests, component fixture, dedicated PNG,
module documentation, and plan/output records required by those changes. M1.1
TextField/Dialog artifacts remain a separate accepted candidate and Render18 paths are
explicitly excluded.

The implementation is complete only when current-source managed validation, the
dedicated screenshot, target-name scans, plan status, documentation, exact review, and
coordinator milestone commit all agree on the same manifest.

## Alternatives Considered

### B. Preserve condition chains and add one runtime branch

This is smaller but leaves priority encoded implicitly in branch order across multiple
modules. It is easy for later focus or drag changes to drift again.

### C. Add public `ButtonInteractionState::Dragging`

This is semantically explicit but expands a shared public contract and all consumers
for a visual distinction that Button currently folds intentionally into Hover. It is
too broad for this Layout15 correction.

Approach A is selected because it makes retained priority explicit while preserving the
existing public Button interface and shared painter behavior.
