---
related_code:
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
tests:
  - zircon_editor/src/tests/workbench/registry/instance_policy.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/workspace_restore.rs
  - zircon_editor/src/tests/host/manager/document_toolkit_lifecycle.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor Workbench view descriptor, instance generation and plugin lifecycle review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for mutable instance authority and aggregate clone removal; P1 for descriptor,
  capability and plugin lifecycle generations.
- Accounting: retain `zircon_editor/src/ui/workbench/view/**` in `pending.md`. Do not add it to
  `review.md` before the cutovers and product traces below pass.
- Code disposition: no Rust source changed. The fix crosses Workbench session, extension/plugin and
  persistence owners, the Editor source tree is held by an active session, and two focused tests
  contain foreign changes. The implementation owner must re-read and re-hash current source.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/view/**` | 22/22 | 578 | 0 in-module | 18,289 | `154c82ae4a1773f9bb1dfaf1382f2956ce9e379965ddedcb507684c11eeac5ef` |
| focused registry/host/restore/toolkit tests | 5/5 | 1,175 | 21 | 44,570 | `7875be69d4ba0ec2175f825612c364edd665d009f808e23920afc198c8da82e6` |

The fingerprint is SHA256 over each sorted normalized path, LF, then raw file bytes. All 22
production files and all five focused tests were read in full. Callers were traced through view
open/attach/close, session restore/persistence, full reflection/chrome construction, pane payload
collection, main-window close and viewport toolbar pointer sizing.

## Module acceptance record

| module | current-source performance verdict |
|---|---|
| descriptor model/builder | Descriptors own multiple strings, vectors and templates. Capability lists are neither normalized nor compiled; every filtered listing clones enabled descriptors. |
| instance model | A full mutable `ViewInstance`, including arbitrary JSON payload and host path, is stored in both registry and session. Deep copies are the public read contract. |
| registry lookup/open/restore | ID lookup is hashed, but singleton reopen returns the registry's stale full instance copy. Register supports add-only duplicate rejection; no replace, unregister, owner or generation contract exists. |
| host/reflection consumers | Full reflection/chrome clones descriptor and instance collections. Narrow close and pointer-size paths also clone every instance although they need IDs or one host only. |
| focused tests | Tests cover small descriptor metadata, singleton IDs, restore and toolkit close. They do not cover metadata coherence, payload clone bytes, scale, deterministic ordering, capability churn or plugin revoke/reload. |

## Structural bottlenecks

### P0: registry and session are conflicting mutable instance authorities

`ViewRegistry::open_descriptor()` stores a complete `ViewInstance` and returns another clone.
`EditorUiHost::attach_instance()` then clones that object into
`EditorSessionState::open_view_instances`. Metadata edits update only the session copy. A later open
of a single-instance descriptor returns the old registry copy, and `attach_instance()` inserts it
back into the session, so a focus/open action can overwrite newer title, dirty state, payload and
host data with stale values.

This is both a correctness defect and an amplification source. Every payload byte can remain in two
authorities and is copied again by read APIs. Optimize50 and Editor06 must define one
`ViewInstanceGeneration`: session/workspace owns mutable live metadata; the descriptor/extension
runtime owns only descriptor generation, singleton identity and allocation counters. Reopen returns
an ID/lease to the live session entry, never a second mutable object. A hard cut must remove the
registry's full instance map rather than synchronize two copies.

### P0: aggregate clone APIs sit on full and narrow interaction paths

`current_view_instances()` clones every instance, including the complete `serde_json::Value` tree
and host path. Full chrome/reflection consumes that vector together with a cloned layout and cloned
descriptor vector. `activity_descriptors_from_views()` then allocates new IDs, titles, icons and
formatted reflection paths for each activity descriptor.

The same aggregate API appears on narrow paths. Main-window close clones all instances and then
keeps only IDs. `viewport_toolbar_surface_size(surface_key)` clones all instances and linearly scans
them to find one host during pointer/geometry handling. Pane payload collection also requests the
complete vector when only a small set of active document kinds is relevant. A downstream equality
cache cannot recover these producer-side bytes.

EditorUI08/Optimize01 must consume immutable generation handles and indexed views: O(1) lookup by
instance ID, an ID slice for close, and typed bounded payload snapshots only for the owning pane.
Stable full reflection reads a shared descriptor/instance generation; it does not recreate owned
collections. Arbitrary document payload must not ride along with unrelated chrome or pointer work.

### P1: descriptor capability filtering recompiles and allocates on every list

`list_descriptors()` visits every descriptor, calls `descriptor_capability_error()` as a boolean
predicate, clones every missing capability into a temporary vector, and formats an error when the
descriptor is unavailable. It then deep-clones every available descriptor. Duplicate required
capabilities are accepted by the builder. The backing `HashMap::values()` order is not stable, and
that order feeds activity descriptor projection.

Compile one deterministic descriptor generation whenever contributions or capability generation
change. Normalize/deduplicate capability requirements at admission, separate cheap availability
testing from on-demand error formatting, and publish stable ordered IDs plus indexed immutable
descriptors. Unchanged reflection performs zero capability probes, string/error allocations, sorts
or descriptor deep clones.

### P1: add-only view registration cannot implement plugin revoke or reload

The registry can register a descriptor and reject a duplicate, but it cannot unregister, replace,
attribute an owner generation, quiesce instances or publish a delta. This contradicts the existing
Editor06/12/Optimize50 direction in which plugin contributions are ticket-owned and revocable.
Plugin disable/reload can therefore leave a materialized view descriptor or live instance outside
the authoritative extension generation, while a replacement descriptor cannot be installed
atomically under the same ID.

Optimize50 must make view materialization part of the shared `ExtensionOwnerGeneration`
prepare/commit/publish/quiesce/revoke transaction. Existing instances are either migrated under an
explicit schema contract, closed after dirty/save admission, or represented by a bounded unknown-
plugin placeholder. No callback, descriptor or payload owner may survive a revoked generation.

### P1: workspace restore admits unbounded instance identity and JSON payload before live mutation

`ViewInstance` persistence contains owned IDs, title, arbitrary JSON, host paths and dirty state.
Restore clones each complete instance into registry and session before later UI/toolkit repair. The
view module imposes no payload bytes/depth/node, string, instance-count or path-length bounds.
Optimize13 must validate a versioned bounded workspace DTO before live mutation, resolve descriptor
owner/schema versions, and stage unknown or disabled plugin views as bounded placeholders.

## Reference-engine evidence

- Unreal `TabManager.h:72-136` gives tabs a compact `FTabId`; persisted `FTab` at `485-514` stores
  the ID and tab state rather than a duplicate arbitrary live editor payload. This supports compact
  layout identity while document/toolkit state remains with its owner.
- Unreal `TabManager.h:1040-1052` and `TabManager.cpp:1197-1218` expose explicit register,
  unregister and unregister-all spawner lifecycle. `TabManager.cpp:3620-3641` replaces an existing
  global spawner before publishing the new one, supporting atomic reload rather than add-only drift.
- Unreal `TabManager.cpp:2475-2484` and `3295-3320` use indexed local/global spawner lookup.
  `2634-2669` spawns/reuses a live tab and retains only a weak spawned-tab pointer in the spawner,
  rather than storing a second full mutable tab model.
- Unreal `TabManager.cpp:2678-2694` can return an explicit unrecognized tab or retain an unknown tab
  in layout restoration. This supports bounded plugin placeholders and later rehydration instead of
  silently deleting layout identity or keeping revoked live callbacks.

These references establish lifecycle, identity and ownership patterns. They do not prove Zircon
timing, allocation, power or interaction parity; current-product WPR/ETW evidence remains required.

## Required architecture cutover

1. Editor06/Optimize50 publishes one immutable, stable-order descriptor generation attributed to
   extension owner generations, with atomic add/replace/revoke deltas.
2. Capability changes compile availability once per generation. Error text is produced only for an
   explicit failed open, never while filtering a presentation list.
3. Session/workspace becomes the only mutable live-instance authority. Registry retains only
   descriptor generation, singleton ID index and checked allocation counters.
4. Public access splits into O(1) instance lookup, compact ID iteration and owner-specific bounded
   payload access. Remove aggregate full-payload cloning from pointer, close, chrome and reflection.
5. EditorUI08 reads shared descriptor/instance generations and derives activity/chrome artifacts at
   most once per changed generation; unchanged frames and unrelated domain changes do zero work.
6. Editor12/Optimize50 reconcile plugin generations transactionally. Revoke quiesces callbacks and
   resolves dirty/live instances before publishing removal; reload cannot expose mixed generations.
7. Optimize13 validates bounded versioned instance DTOs and stages restore before mutation, with
   explicit unknown/disabled-plugin placeholders and schema migration.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters for descriptor/instance visits, capability probes, deep-clone payload/host bytes, duplicate resident bytes, stale singleton overwrites, lock wait/hold and plugin generation transitions. | current source re-read |
| M1 | Single mutable instance authority plus O(1) lookup/ID iteration; remove registry full-instance copies and stale singleton reopen. | Editor06 + Optimize50 |
| M2 | Stable immutable descriptor/capability generation and generation-cached activity/chrome projection. | EditorUI08 + Optimize01 |
| M3 | Transactional owner-generation register/replace/revoke/reload with quiescence and placeholder policy. | Editor12 + Optimize06/50 |
| M4 | Bounded versioned workspace instance schema and staged restore/migration. | Optimize13 |
| M5 | Current-source managed Cargo/F4 plus WPR/ETW CPU, allocation, lock and package-power matrix. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| stable/read paths | descriptors/instances `1/1k/100k`, payload `0/1KiB/1MiB`, full reflection, pointer size, main close | unchanged capability/descriptor/instance builds `=0`; narrow path unrelated visits and payload clone bytes `=0`; lookup near O(1) |
| authority | metadata title/dirty/payload/host mutation followed by singleton reopen, attach, detach, save and close | registry duplicate payload/resident bytes `=0`; reopen cannot overwrite current metadata; exactly one mutable generation owner |
| capabilities/order | capabilities `0/1/100/10k`, duplicate requirements, capability add/remove, repeated process runs | one compile per changed generation; unchanged probes/error allocations/clones `=0`; deterministic descriptor/activity order and error parity |
| plugin lifecycle | plugins/views `1/100/1k`, disable/reload/fault, duplicate ID, dirty/live view, unknown workspace view | atomic generation publication; stale descriptors/callbacks/instances `=0`; bounded quiescence; placeholder/migration/dirty-save semantics pass |
| workspace | instances `1/1k/100k`, payload current/N-1/future/corrupt/deep/wide/oversize | bounds enforced before full materialization/live mutation; one staged commit; no partial registry/session state; unknown plugin identity preserved safely |
| product | F4 cold/warm/idle, open/focus/drag/close/reload/restore storms, 31 runs | WPR/ETW CPU, allocation, lock hold/wait, input-to-pixel p50/p95/p99, RSS and package power on identical hardware/config; artifacts only on D/E/F |

RenderDoc is required only if the cutover changes draw resources, ordering or pixels. It cannot
measure JSON clone bytes, authority duplication, capability filtering, locks or package power.

## Static gates executed

- Read 22/22 production files and five focused test files in full; reproduced current line, byte,
  test counts and both fingerprints above.
- Traced register/open/restore/attach/update/close, session/workspace duplication, full reflection,
  pane payload, main-window close and pointer-size consumers.
- Read the cited Unreal compact tab identity, explicit spawner lifecycle, indexed lookup, weak live
  tab tracking, replacement and unrecognized-tab source ranges.
- Preserved foreign changes in `workspace_restore.rs` and `document_toolkit_lifecycle.rs`.
- No Cargo lane, F4 launch, WPR/ETW, package-power or RenderDoc capture was run. Dynamic acceptance
  remains pending; RenderDoc is not applicable because no rendering-visible source changed.

## Completion rule

This module remains pending until M0-M5 pass against a current source fingerprint. Static review,
small semantic tests, hash-map lookup in isolation or a downstream equality cache is not acceptance.
No milestone commit or WeCom completion message is permitted before quantified product evidence.
