# Runtime layout slot authority review

Date: 2026-08-28

Status: static implementation candidate. Tree-owned edge lookup, mutation closure, and exact
child-membership patching are implemented; managed Rust, production counters, physical parent-edge
storage, and product profiling remain open.

## Outcome

The static candidate closes the workspace-wide scan defect without weakening mutation safety.
`UiTree::slots` is now a private serialized carrier. All Runtime and Interface construction and
mutation sites use `push_layout_slot`, `replace_layout_slots`, `retain_layout_slots`, or
`mutate_layout_slot`; an internal edge-to-index authority is updated by those APIs and rebuilt once
after deserialization. Missing-edge and kind lookup now query that authority directly.

`UiLayoutSlotIndex` no longer owns a second defensive edge map, performs same-cardinality repair, or
scans every slot while patching one parent. Rebinding a slot preserves original flat-slot precedence;
bulk removal reindexes once; a no-op retain does not rebuild. Child property changes patch one
dependency membership through parent-local position maps. Container or child-topology changes are
typed parent-local fallbacks that rebuild `C_parent`. A fully co-located `UiChildEdge`
representation remains later work.

## Current-source proof

### Closed split-authority bypass

- `zircon_runtime_interface/src/ui/tree/node/ui_tree.rs` keeps `slots: Vec<UiSlot>` private and owns
  `UiLayoutSlotAuthority` plus immutable lookup APIs.
- Runtime Surface, virtual-list, template, layout and test construction no longer access
  `tree.slots` directly. A repository source guard rejects that bypass.
- Deserialization starts with an empty skipped authority and the first lookup rebuilds it exactly
  once. Repeated authoritative misses do not revisit the flat carrier.
- Same-cardinality rebind updates the previous and destination edge without rebuilding; bulk
  retention rebuilds only when indices actually shift.

### Removed repeated global scans

The rejected source had two global scans. `rebuild_parent_size_dependencies` could scan all `M`
slots for each of `C_parent` missing edges, and `patch_layout_order_parents` filtered all `M` slots
for a one-parent change.

Current `free_child_depends_on_parent_size` uses `UiTree::layout_slot_index_for_edge_kind`, and
current `patch_layout_order_parents` rebuilds only the named parents' child projections. Production
`slot.rs` contains no `tree.slots`, `edge_indices`, `index_for_edge_matching`, or repair fallback.
The achieved child-property patch is `O(A log C_parent)` and preserves tree order through a
position-keyed dependency map. Parent container/topology changes remain `O(C_parent)` by design.

## Reference-engine boundary

Unreal Slate co-locates the child and its layout slot at the panel owner:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Children.h:454-470` defines
  `TPanelChildren<SlotType>` with `TArray<TUniquePtr<SlotType>> Children`; `GetSlotAt(index)` reads
  the same collection.
- Lines 485-512 expose child lookup and `AddSlot`, which adds and constructs the slot in that
  parent's `Children` array.
- Removal is likewise local to that array. Slot mutation invalidates the owning widget through the
  normal Slate invalidation contract.

The transferable point is ownership, not Unreal's pointer type. Layout traversal must obtain child
identity and placement from the same parent-local record; it must not search a workspace table to
discover whether an edge has a slot.

## Target runtime authority

Use a single parent-owned edge representation, conceptually:

```rust
struct UiChildEdge {
    child_id: UiNodeId,
    slots: SmallVec<[UiSlot; 1]>,
}

struct UiTreeNode {
    // other node state
    children: Vec<UiChildEdge>,
}
```

Multiple slot kinds for one edge remain representable. The active container kind selects from the
edge-local small collection. If the implementation keeps child IDs and slots in separate arrays
for serialization or cache locality, they must be private aligned fields owned by one parent API;
callers cannot mutate either independently.

The current candidate is an intermediate compatible form: the flat serialized array is private and
the runtime edge lookup is mutation-closed, but child identity and slots are not yet physically
co-located in `UiTreeNode`. Documentation and pressure evidence therefore call it tree-owned edge
authority, not completed parent-local edge storage.

Required mutation journal:

```rust
enum UiLayoutEdgeMutation {
    Insert { parent_id, child_id, position },
    Remove { parent_id, child_id },
    Reorder { parent_id, child_id, position },
    PatchSlot { parent_id, child_id, kind, dirty_domains },
}
```

The journal advances one layout-edge generation and records exact parents/children. Deserialization
converts legacy flat records into this authority, validates endpoints/kinds/duplicates and then
discards the flat input. Same-cardinality replacement is an explicit mutation, not something the
lookup cache discovers later.

Derived state becomes parent-local:

- ordered child positions for containers that use slot order;
- parent-size-dependent child positions;
- optional child-id-to-position lookup for mutation and pointer contracts;
- virtual-list materialized edge ranges.

A property change that can alter parent-size dependency patches only that child's membership.
Parent child-order/topology change may rebuild that parent's `O(C_parent)` projections. No path
scans slots belonging to another parent. Missing-slot lookup is an authoritative edge-local miss.

## Complexity contract

Let `E` be retained child edges, `M` legacy serialized slots, `C_parent` affected parent degree,
`A` changed children and `K_parent` slots on the affected parent.

| Operation | Rejected source | Static candidate | Remaining target |
| --- | --- | --- | --- |
| initial index build with unslotted edges | `O(E*M)` | `O(E + M)` | parent-local storage/migration |
| one child dependency change | `O(C_parent*M)` | `O(A log C_parent)` | direct edge-local `O(A)` |
| parent container/topology change | `O(C_parent*M)` | `O(C_parent)` | same parent-local bound |
| one parent order change | `O(M + C_parent*M + C_parent log C_parent)` | `O(C_parent + C_parent log C_parent)` | same bound with direct edge records |
| missing slot lookup | defensive `O(M)` repair | indexed edge lookup | direct edge-local lookup |

The ordered-child sort is shared work when order actually changes and is not claimed as eliminated.
The key requirement is independence from unrelated workspace slots.

## Pressure evidence

`tools/runtime_ui_layout_slot_index_pressure.py` counts child/slot row visits for the exact
no-matching-slot worst case. It is not CPU timing. Tests bind the model to current Zircon sources
and Unreal's parent-owned storage.

Artifact:
`E:\zircon-profiles\runtime-ui-layout-slot-index-pressure-20260831-current.json`

SHA-256:
`C34588A044FC01D3DF86378E034BF1358E37E064EA8EF46788E6351E05525834`

All scenarios use 10,000 global slots owned by other edges and one changed child/parent:

| children | rejected full | achieved full | rejected dependency | exact child | topology fallback | rejected order | achieved order |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 650,128 | 10,128 | 640,064 | 1 | 64 | 650,064 | 64 |
| 1,000 | 10,012,000 | 12,000 | 10,001,000 | 1 | 1,000 | 10,011,000 | 1,000 |
| 10,000 | 100,030,000 | 30,000 | 100,010,000 | 1 | 10,000 | 100,020,000 | 10,000 |

These are deterministic worst-case accesses, not current tree cardinalities, average slot
distribution, CPU cycles or speedup measurements. The v2 artifact binds HEAD
`b2e76ff33cc298ad76f7b801a1d06d1e2faa046d` and exact hashes for `UiTree`, `slot.rs`, the
virtual-list pool, and Unreal `Children.h`. The model excludes BTree constants, sorting,
serialization, allocator, measurement, arrangement, hit testing, paint and GPU work. Focused
model/source contracts pass 7/7. Tool SHA-256 is
`F57806C5073BE2BA2397D920BF395528CBE13C34EC35D5E049A9B50D883A512F`; test SHA-256 is
`9F981CCF37012AFCC4C058CE55B3ABA687A73828D12F94C0820719C06389F1AC`.

## Dynamic acceptance evidence

`tools/runtime_ui_layout_edge_evidence.py` now turns the target authority into a source-bound,
fail-closed gate instead of treating absent telemetry as zero. It accepts four explicit scenarios:
one-time legacy migration, full parent-edge projection, an exact child dependency patch, and one
parent order patch. Each measured run must publish operation/duration samples plus edge, journal,
workspace-scan, fallback, parity and allocation counters under `ui.layout_edge.*`.

The gate rejects all of the following:

- retained runtime flat-slot authority or non-migration legacy slots;
- a missing-edge lookup, child patch, or parent order patch that visits workspace slots;
- child work beyond the exact changed-child set or order work beyond affected parent edges;
- structure mutations that do not conserve exactly with the edge journal;
- defensive repair, layout/frame/clip/hit parity mismatch, missing/invalid counters, or a missing
  source manifest;
- local visit count, p95 duration, or allocation bytes that grow by more than 10 percent when
  unrelated workspace slots scale from 64 to 1,000 and 10,000.

Focused tests are 12/12 GREEN. Tool SHA-256 is
`63BA7D35D01E17D6B1545473C3A5DF5322018FD05E6A600B73ADF2320CA09B46`; test SHA-256 is
`32508B967C98690CBC5E4287E2006B541494FE65545892300279F418F04E00E1`.

The historical product trace
`E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch\20260811-201002-click-dock-patch-spaced\timeline.zrtrace.json`
was replayed only as a diagnostic. It exits 2 and returns `ready=false` with 20
`missing_counter` blockers and one `missing_source_manifest`; therefore it is not accepted as
evidence for the current layout authority. Rejection artifact
`E:\zircon-profiles\runtime-ui-layout-edge-evidence-20260829-historical-regression.json` has
SHA-256 `98C54729D6B460C50BA3CBAD079E636D58B8BB3B4623C57FFBF8484022374D12`.

The static candidate closes the public mutation bypass, global scan shape, and exact child
membership algorithm. Production paths do not yet publish the dynamic counters, physical slot
storage is not parent-local, and no managed Rust or product CPU/RSS/input-to-present measurement has
passed.

## Test-first migration plan

1. `[done-static]` Make runtime `slots` private, route Runtime/Interface callers through tree APIs,
   and guard against direct field access.
2. `[done-static]` Add one-time deserialize rebuild, repeated miss, same-cardinality rebind,
   precedence, bulk removal and no-op retention regressions.
3. `[done-static]` Remove defensive/global edge repair and workspace slot scans from layout lookup,
   order patching and dependency rebuild.
4. `[pending-managed]` Execute the lower Rust regressions and existing container parity suites.
5. `[done-static]` Patch parent-local dependency membership by exact changed child, with explicit
   `C_parent` fallback for container/topology changes.
6. `[pending-structure]` Co-locate child identity and slot payload or implement a validated load-time
   migration that makes the parent edge the physical runtime record.
7. `[pending-managed]` Preserve layout/frame/clip parity for Free, Canvas, Overlay, linear, wrap,
   grid, masonry, Taffy and virtual-list paths against a forced full layout oracle.
8. `[pending-product]` Run 10,000-node stress cases for initial load, one child property patch, one
   reparent, one order change and virtual-list pool growth. Record edge/slot visits, layout probes,
   allocations, CPU, RSS and p50/p95/p99.

## Acceptance gates

- no public mutable runtime flat-slot authority;
- no production direct mutation bypassing the journal;
- no `tree.slots.iter()` in incremental layout lookup/patch code;
- missing slot lookup performs zero workspace-wide slot visits;
- one child dependency patch visits no unrelated parent slot; the final gate also visits no clean
  sibling payload;
- one parent order patch visits only that parent's edges/slots plus required sort work;
- structure mutation count equals journal count, with no defensive-repair fallback;
- full and incremental layout/frame/clip/hit results match for every supported container;
- product resize/button/popup scenarios show no regression in CPU/RSS or matching-generation
  latency.

## Ownership and sequencing

The static candidate was implemented as one mutation-closed slice: private carrier and APIs first,
call-site migration second, edge authority regressions third, and only then global fallback removal.
It remains unaccepted until managed lower parity and current-source product profiling pass; no Cargo
or commit is claimed by this report.

## Current-source static revalidation (2026-08-31)

The pressure, fail-closed edge-evidence and incremental-layout source-contract
suites pass 38/38 on current HEAD
`14c89f9776bed828cc85e05e4b9914b3f8d1e784`; the shared Taffy opportunity
suite adds seven independently passing cases. Python bytecode compilation for
all three tools passes. The regenerated slot-index artifact above binds exact
current hashes for `UiTree`, `slot.rs`, the virtual-list prototype pool and
Unreal `Children.h`.

The 10,000-child / 10,000-unrelated-slot fixture still distinguishes achieved
and pending work correctly: full tree-edge projection is 30,000 modeled units,
an exact child dependency patch is one changed-child visit, and parent topology
or order fallback is 10,000 parent-local child visits with zero workspace slot
visits. Physical co-location of child identity and slot payload plus an exact
edge mutation journal remain pending. This is current-source operation-count
evidence, not CPU/RSS or latency evidence.
