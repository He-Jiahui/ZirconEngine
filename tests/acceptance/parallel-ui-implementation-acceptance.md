# Parallel UI Implementation Acceptance Standard

## Purpose

This standard is the shared gate for parallel Zircon UI implementation tasks. It prevents a task from claiming completion from one local happy path while another active session still owns a lower shared layer, editor bridge, runtime renderer, or debug surface needed by the same feature family.

Use this file for the active UI roadmap around Slate-style surface frames, Material `.ui.toml`, editor host cutover, runtime UI rendering, input events, text, invalidation/performance, and Widget Reflector diagnostics.

## Required State Before Any Task Claims Acceptance

- The task has a current coordination note under `.codex/sessions/` while active, or it explicitly references the active note that owns the milestone.
- The task lists its exact owned modules and active non-owned modules it avoided.
- The task states the milestone and feature inventory it claims to finish. "UI works" is not an acceptable scope.
- The task records the current lower-to-upper validation path: interface/contract, runtime shared behavior, asset/template, editor host, runtime renderer, visual/debug/performance, and package/export if applicable.
- The task records whether validation was package-scoped or workspace-wide. Do not imply workspace green unless workspace validation actually ran.

## Layered Acceptance Gates

### Gate 0: Coordination And Scope

- Fresh coordination context was scanned within 4 hours before editing or validating shared UI files.
- Active sessions touching the same files were read before edits.
- The task records ownership boundaries, including files it intentionally did not modify.
- Any failing test outside the task's scope is mapped to a recent plan/session before it is fixed or used as evidence.
- No unrelated dirty work is reverted, reformatted, or silently folded into the task.

### Gate 1: Shared Contract

- Public DTOs in `zircon_runtime_interface::ui` construct, serialize, and preserve defaults through contract tests.
- Any new cross-layer type has an interface test that proves the wire/debug shape, not only runtime behavior.
- Contract additions are neutral: editor and runtime can consume them without importing each other's implementation crates.
- Old and new names are not kept alive as compatibility aliases unless the user explicitly requested temporary coexistence.

### Gate 2: Runtime Shared Behavior

- The lowest shared behavior has focused runtime tests before editor or renderer acceptance is claimed.
- Layout, render extract, hit testing, event routing, text measurement, dirty rebuilds, and diagnostics consume `UiSurfaceFrame` or the declared shared authority for the task.
- Visibility, clip, z/paint order, disabled/input policy, focus/capture/hover/pressed state, scroll/virtual range, and dirty flags have boundary coverage when the task touches those areas.
- A task cannot pass by adding editor-specific coordinate tables, component-name string checks, or one-off fallback branches in shared paths.

### Gate 3: `.ui.toml` And Material Assets

- `.ui.toml` remains the source for layout/component structure. Generated `.slint` UI must not re-enter the shipping path.
- Material/editor surfaces import the Material theme or record an explicit chrome-only/runtime exception.
- Interactive controls use Material meta components or equivalent descriptor/style/layout metadata.
- New bindings survive meta-component expansion and reach the dispatch envelope with stable binding ids.
- Fixed sizes are justified as chrome rails, icon squares, status bars, bounded dialogs, or test fixtures; main panels and lists use responsive constraints, scroll, stretch, or min/preferred/max sizing.

### Gate 4: Editor And Runtime Integration

- Editor host behavior uses shared surface frames, routes, bindings, and debug DTOs instead of local coordinate or widget-family tables.
- Runtime behavior uses the same `.ui.toml`/template/runtime UI chain and does not depend on Slint host types.
- If both editor and runtime should support the feature, there is at least one parity test or acceptance record proving the same authored surface enters both paths.
- If runtime support is intentionally deferred, the acceptance record says so and names the interface that was reserved.

### Gate 5: Rendering, Text, Input, And Debug Evidence

- Rendering changes prove visible pixels or render-command output for normal, clipped, missing-asset, and overlap cases relevant to the task.
- Text changes prove shared measurement before painter clipping, plus wrap/ellipsis/caret/selection/IME boundaries when in scope.
- Input changes prove route, reply/effect, focus/capture, release-inside click, keyboard/text/navigation, drag/drop, popup/menu, and failure diagnostics for the subset claimed.
- Debug/performance changes expose machine-readable evidence: reflector snapshot, hit stack/reject reasons, drawcall/material batch counters, overdraw samples, invalidation/damage counters, or exported replay/snapshot data as applicable.

### Gate 6: Validation Stage

- Validation runs from lower to upper layers: interface contract, runtime shared tests, editor/runtime integration, visual/debug/performance tests, then package/export or workspace checks when in scope.
- Commands include `--locked` by default and record `--target-dir`, package, filter, and whether the run was scoped.
- Timeouts, broken pipes, cold-build tool interruptions, and dependency-compilation exits without Rust diagnostics are recorded as inconclusive, not pass evidence.
- Warnings can be left only if they are pre-existing or out of scope; the acceptance record must not hide new warnings introduced by the task.

### Gate 7: Documentation And Handoff

- Code-facing docs under `docs/` are updated when behavior or public contracts change.
- Acceptance evidence is written under `tests/acceptance/` with exact commands, results, deferred risks, and owner notes.
- The active `.codex/sessions/` note is deleted or archived with a completion summary after the task ends.
- Parallel sessions are warned when the accepted change alters their assumptions or test baseline.

## Domain-Specific Required Evidence

### Slate Surface / Hit Testing

- Prove `layout -> arranged tree -> render extract -> hit grid -> input route` uses one spatial authority.
- Include tests for visibility variants, clip chains, overlap ordering, disabled nodes, `HitTestInvisible`, `SelfHitTestInvisible`, scroll virtualization, and debug reject reasons.
- Editor hit tests must consume submitted `UiSurfaceFrame` data, not locally rebuilt per-widget coordinates.

### Material Layout / `.ui.toml`

- Prove Material meta components produce stable intrinsic sizes and arranged frames.
- Prove Component Showcase or another representative `.ui.toml` surface carries Material bindings and layout metadata through template expansion.
- Prove text affects desired size before native painter clipping.
- Global surface conformance must inventory all `.ui.toml` files in scope, not just the edited screen.

### Rendering / Painter / Runtime UI

- Prove render extract commands and visible pixels for quad, text, image/icon/SVG, clip, fallback asset, and overlap cases in scope.
- Prove editor native painter and runtime renderer consume equivalent command semantics when the feature is shared.
- Prove drawcall/material batch/overdraw diagnostics are either backend-confirmed or clearly marked as deterministic estimates.

### Input / Widget Behavior

- Prove low-level route and semantic component event separately.
- Prove default actions come from descriptor/binding metadata, not control-name string checks.
- Include negative cases: disabled/hidden targets, unsupported event, missing binding, release outside press target, lost capture, popup outside press, rejected drop payload.
- Keyboard/text/IME/navigation/gamepad acceptance must state which subset is implemented and which remains reserved.

### Text

- Prove shared desired-size measurement and renderer/painter output stay coherent for short labels, long labels, wrapping, clipping, and empty text.
- Editable text must prove value, cursor, selection, commit, focus lost, and composition boundaries before it is accepted as complete.
- If shaping/BiDi/IME is deferred, acceptance must say the feature is not text-system complete.

### Invalidation / Performance / Debug UI

- Prove local damage for hover/press/text cursor/viewport image/popup/counter changes where claimed.
- Prove repeated same-target hover or idle move avoids presentation rebuild.
- Prove full rebuild is reserved for layout/data/window-metric changes.
- Widget Reflector acceptance requires live or snapshot tree, path, geometry, visibility/input policy, render/hit/debug counters, and exportable diagnostic data.

## Acceptance Record Template

Copy this into the task-specific acceptance file:

```markdown
# <Feature Or Milestone> Acceptance

## Scope
- Milestone:
- Owned modules:
- Non-owned active modules avoided:
- In-scope inventory:
- Out-of-scope/deferred:

## Coordination
- Coordination command:
- Timestamp:
- Active sessions/plans read:
- Cross-session warnings:

## Implementation Evidence
- Contract files changed:
- Runtime behavior files changed:
- Editor/runtime integration files changed:
- Assets/docs changed:
- Compatibility paths removed:

## Validation
- Interface/contract commands and results:
- Runtime shared commands and results:
- Editor integration commands and results:
- Runtime renderer/package/export commands and results:
- Visual/debug/performance evidence:
- Inconclusive commands:

## Acceptance Decision
- Status: accepted | blocked | accepted-with-deferred-risk | not-accepted
- Reason:
- Remaining risks:
- Follow-up owner/session:
```

## Non-Acceptance Conditions

- The task only passes one demo, one constructor test, or one happy-path integration test.
- The task skips lower shared support tests while claiming editor/runtime behavior.
- The task relies on a new host coordinate table, stringly control-name behavior, or compatibility shim that the roadmap says should be cut over.
- The task does not update docs/acceptance records after public contracts or behavior change.
- The task reports a timed-out or broken-pipe run as passing.
- The task says "workspace green" without workspace validation.
- The task leaves an active session note in `.codex/sessions/` after completion.
