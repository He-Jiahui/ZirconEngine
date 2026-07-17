# Minimum Viable Engine Foundation Acceptance

This document is the execution-priority policy for every active plan under
`docs/plans/`. It defines the smallest acceptable ZirconEngine product loop.
It has priority over new work on advanced rendering, complete text coverage,
AI, networking, or plugin expansion.

## Priority Rule

1. Every Session must classify its next milestone as either an MVP foundation
   gate, a direct blocker for one, or deferred work.
2. Work that is neither a foundation gate nor its direct blocker must not take
   a shared implementation or validation lane while any earlier foundation
   gate remains unaccepted.
3. Existing advanced work may repair a regression that blocks a foundation
   gate, but must not add new product scope before the MVP loop is accepted.
4. Record accepted gate evidence through the owning numbered plan and follow
   `milestone-validation-policy.md`; do not create per-slice progress records.

## MVP Product Loop

The engine is minimally usable only after this exact loop is accepted with
current-source evidence:

1. `zircon_runtime` starts a project and creates a basic scene.
2. The basic scene renders, receives keyboard and mouse input, and exits
   cleanly and reliably.
3. The project and scene save, reopen, and preserve the authored state.
4. `zircon_editor` opens the same project, selects one entity, modifies it,
   and saves the result.
5. This exact path passes one batched build in a clean environment and two
   consecutive product launches.

The gates below divide that loop into owner-scoped implementation and
acceptance work; no gate may be accepted from static evidence alone.

| Gate | Required user-visible outcome | Primary plan owners |
| --- | --- | --- |
| F0: Reproducible bootstrap | Supported `zircon_runtime` and `zircon_editor` profiles build and start from a clean product configuration, report actionable startup failures, and exit cleanly. | Runtime 02/14; Editor 01 |
| F1: Project and assets | A project can be created or opened through one supported route; its asset registry and project settings load without fallback-only state. | Runtime 04; Editor 01/02 |
| F2: Scene runtime | A persisted scene with one camera, one visible primitive, one light, and keyboard/mouse input renders and enters and leaves the `zircon_runtime` loop deterministically. | Runtime 08/09/12; Render foundations |
| F3: Persistence | The project and scene can be saved, reopened, and compared for the authored entity, transform, and referenced asset state. | Runtime 04/08; Editor 11 |
| F4: Basic authoring | `zircon_editor` opens the same project, selects one entity, changes a transform or property through the command path, saves it, and observes the change after reopening. | Editor 01/02/05/08/11 |
| F5: Acceptance wave | The F0-F4 path passes one clean-environment batched build, the focused contract/integration suite, and two consecutive product runs with captured diagnostics. | Dedicated validation lane |

Do not claim an MVP gate from static guards, source inventory, a single unit
test, or an unverified feature flag. The product path must exercise the
current executable and persisted project data.

## Explicit Deferrals

The following are valuable but outside the foundation critical path until F5
is accepted:

- Advanced rendering: temporal reconstruction, volumetrics, advanced lighting,
  RenderDoc optimization, shader permutation breadth, and high-end effects.
- Complete text: full BIDI and vertical-layout coverage, MSDF breadth, rich
  text expansion, IME platform specialization, and typography polish.
- AI, networking, multiplayer replication, and nonessential first-party
  plugin capabilities.
- New editor panels, advanced domain tools, export targets, and performance
  tuning that do not unblock F0-F5.

Foundation work may keep the smallest stable rendering and text path required
by F2-F4. It must not wait for feature-complete rendering or typography.

## Session Routing

Before starting a new implementation slice, a Session must:

1. Read this policy, the active plan, and the current status of the earliest
   unaccepted foundation gate.
2. Select the lowest unblocked F0-F5 gate and state the concrete product
   behavior it will unlock.
3. Escalate only a direct cross-plan blocker through the normal failure
   handoff; continue independent foundation work rather than expanding a
   deferred subsystem.
4. At each accepted gate, add one concise evidence record with the product
   path exercised, batched commands, diagnostics, and remaining accepted risk.

## Exit Condition

After F5 passes, advanced rendering, complete text, AI, networking, and plugin
expansion resume in dependency order. Any new scope must preserve the accepted
MVP product loop and rerun the smallest affected foundation regression batch.
