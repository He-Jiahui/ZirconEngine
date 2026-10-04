---
related_code:
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
---

# Protected plan routing: Workbench view generation and plugin lifecycle

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize01/06/13/50 and numbered owner plans
are protected or foreign dirty. The active session owns the Editor source tree and two focused tests
contain foreign changes. This record routes the 22/22-file current evidence without editing those
authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-view-descriptor-instance-generation-plugin-lifecycle-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-099 and PERF-MVP-106

Add descriptor and instance producer costs to the Workbench generation gate. Full reflection/chrome
deep-clones enabled descriptors, the complete layout and every live instance payload. Pointer-size
and main-close paths also call the aggregate instance clone API although they need one host or only
IDs. Required target: shared immutable descriptor/instance generations, O(1) narrow lookup and zero
unrelated payload clone bytes. Downstream model equality does not satisfy producer acceptance.

### PERF-MVP-102

Extend the pointer acceptance gate to `viewport_toolbar_surface_size(surface_key)`. At present a
pointer/geometry query clones every `ViewInstance`, including arbitrary JSON payload, before a
linear ID search. Require instance visits and payload/host clone bytes `=0` for unrelated entries,
with direct indexed host lookup and existing geometry/floating fallback parity.

### PERF-MVP-104

Count view instance and descriptor generations independently of editor/asset snapshots: builds,
visits, payload bytes, layout/host bytes, activity projection allocations and shell lock time. One
changed domain publishes at most one generation; stable or unrelated event batches do zero work.

### PERF-MVP-107

Extend plugin presentation acceptance beyond pane payload reuse. Plugin view descriptors currently
have add-only materialization with no unregister/replace/owner generation, while session and
registry retain separate full instance copies. Require transactional owner-generation reconcile,
zero revoked descriptors/callbacks and one shared artifact across main/native presenters.

## Requested Optimize and owner updates

### Optimize50 + Editor06

Make view descriptors a table in the shared `ExtensionOwnerGeneration`. Publish stable ordered
immutable descriptors and atomic add/replace/revoke deltas. Make session/workspace the sole mutable
`ViewInstance` authority; the registry keeps only descriptor generation, singleton ID index and
checked allocation counters. Remove the full instance map and never dual-write mutable metadata.

### Optimize01 + EditorUI08

Replace aggregate cloning with generation readers: O(1) instance-by-ID lookup, compact ID slice for
close, and typed owner-specific payload access. Cache activity/chrome projection by input generation
and publish at most once per changed frame/domain. Stable reflection, pointer and close paths must
perform zero descriptor/instance deep-copy work.

### Optimize06/50 + Editor12

Connect plugin Active/Disabled/Faulted/reload state to the exact view descriptor generation visible
to Workbench. Revoke must quiesce callbacks and resolve live/dirty instances before publishing
removal. Reload replaces the owner generation atomically; duplicate IDs, faults and rollback cannot
leave mixed old/new descriptors or live callbacks.

### Optimize13 + EditorLayout08

Persist compact view identity and versioned owner-specific state, not an unbounded arbitrary live
payload copied into multiple registries. Enforce instance/string/path/payload bytes-depth-node
bounds before materialization, stage restore before mutation, and retain unknown/disabled plugin
views as bounded placeholders eligible for later rehydration.

## Requested protected index state

- `pending.md`: add or retain one concise module row for
  `zircon_editor/src/ui/workbench/view/**`, 22/22 files, 578 lines, fingerprint
  `154c82ae4a17...`, `source_recheck_required=true`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require single instance authority, immutable descriptor and
  capability generation, plugin reconcile/quiescence, bounded staged restore, current-source
  managed Cargo/F4 and WPR/ETW CPU/allocation/lock/power evidence.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize50 + Editor06 | registry duplicate payload/resident bytes `=0`; singleton reopen never overwrites current metadata; deterministic descriptor generation and owner attribution |
| Optimize01 + EditorUI08 | stable full build `=0`; pointer/close unrelated visits and payload clone bytes `=0`; one projection per changed generation |
| Optimize06/50 + Editor12 | atomic 1/100/1k plugin view register/replace/revoke/reload; stale descriptors/callbacks/instances `=0`; dirty/fault rollback parity |
| Optimize13 + EditorLayout08 | bounds before materialization; staged current/N-1/future/corrupt/oversize restore; bounded unknown-plugin placeholder and rehydration parity |
| Performance01 | 31-run F4 WPR/ETW CPU, allocation, shell lock, input latency, RSS and package-power matrix on identical hardware/config; artifacts on D/E/F |

RenderDoc remains conditional on rendering-visible changes and proves GPU event/resource/pixel parity
only. It cannot replace WPR/ETW evidence for clone bytes, authority duplication, locks or power.
