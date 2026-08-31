---
title: Editor workbench-window compiled property projection performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/workbench_window_projection
priority: MVP-P0 editor workbench nodes, styles and sparse projection
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate attribute descriptors and invalidation roots
---

# Goal

Compile retained workbench node properties, style, typed-canvas data, resources and parent metadata
once per exact source generation. Full and sparse host projection must reuse unchanged compiled node
artifacts; per-node work must not deep-copy a generic property map, rebuild TOML or allocate temporary
cycle-detection structures.

## Reviewed source

- module files: 10 production child modules plus module tests
- Rust files: 11/11
- current lines: 1,809
- current bytes: 63,275
- joined current source-bytes SHA256:
  `6498feb0717c01ea4cab032db0ecdcbb45eca940fb8c9f52d820a01ea4fa256e`
- joined pre-M1 source-bytes SHA256:
  `7274fbd59c462dc935f2d12d539f1244aa38827b6d614b0163f3043dd29af64e`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `defaults.rs` | 220 | `f8425feabf07afb050b5d47f84f7df541db43e07bf7681730e99c7c703985e87` |
| `host_value_toml.rs` | 102 | `49dedfd01fc36b34b89ad0ada79b17753012c0f09db5914072e7ff52bd5b5e46` |
| `mount.rs` | 155 | `cc4d2b8d42ddfd773f022eebe0ed40e19bd4d43c3685a28c4610226bee7ff8c1` |
| `node_index.rs` | 183 | `7df746a5780dde303a916d2c46bd12212a9a2e82451545652b2785d8566713ce` |
| `notification_cache.rs` | 102 | `1f62f6c546610931244f0600dcfc51706bb815b4144e1db8272e144b81b7412c` |
| `previous_node_index.rs` | 54 | `254711dab0a18a8f0eed6ebde10b7a706882b24ce2c1bdb1423c8445bcc3bce0` |
| `properties.rs` | 178 | `466d99b21811ab91f52480c38fb3fa04e0a1b2477db8c5a575dbedcba73eff67` |
| `selection_style.rs` | 129 | `6545e0e0053ebeb8951b96be689accfb34e6695da6888988758a2ba128b0508a` |
| `status_right.rs` | 72 | `f280003ff31cbdf39e5c750f14081a5239d47aab7f716d6aea78a026f6c0ae93` |
| `tests.rs` | 590 | `676492ba066deba12e923aeafda668b1e7bba5941a1220d512adb059d6d4e35e` |
| `typed_canvas.rs` | 24 | `8db1cfbaee35931f9bae0961b774a23aaec883bbef0eae5962dbd97cf486f9c1` |

All files were read in full. The already-reviewed 572-line parent projection root was followed for
call frequency and ownership. `RetainedUiHostNodeModel`, button-style and typed-canvas consumers were
traced as needed but are not counted here.

## Existing foundations to retain

`ProjectionNodeIndex` memoizes visibility and nearest controlled parents in O(N), including cycle
rejection. `PreviousWorkbenchNodeIndex` attaches document/control-row identity as `ModelRc` metadata.
Sparse projection replaces only changed rows and preserves unchanged row identity. Notification rows
reuse their owned option models only when the complete presentation key matches. Mount/scale code
separates visual metrics from semantic slider values and has full/sparse parity tests.

## Structural findings

### P0: every projected node deep-converts its complete property map to TOML

`toml_values_from_host_properties` walks the entire `BTreeMap<String, RetainedUiHostValue>`, clones
every key/string, recursively copies arrays/tables, parses datetimes and then creates aliases. The root
does this for every visible node so generic style helpers can read it. Notifications avoid one large
array only on a valid previous-row cache hit; all other unrelated properties are still copied.

The same node then independently queries the original property map many times for strings, numerics,
colors, booleans, arrays, options and typed-canvas inputs. A P-property node therefore pays O(P) deep
conversion plus dozens of O(log P) lookups and output allocations on every rebuild. The property
owner must publish a typed compiled projection once, not reserialize a runtime value map into TOML.

### P0: previous-node identity is used narrowly rather than as a node artifact cache

The previous row index finds the exact prior control row, but the root redoes component-role,
property/style, options/menu, typed-canvas, route, resource and geometry projection. Only notification
option rows use a domain generation. Unchanged nodes in a full projection therefore receive new wide
host rows even when their property/resource/style generations match.

Publish a per-node receipt and immutable semantic artifact. Geometry/mount and interaction state must
be independent patches so changing hover or mount does not rebuild typed properties and media.

### P0: property values mix semantic data, style and large collections

The generic map carries style aliases, notification collections, command options, menus, typed canvas,
component values and state. Converting it wholesale makes it impossible to know which change should
invalidate layout, paint, input, collection rows or semantic text. This is the design source of both
overwork and imprecise invalidation; filtering a few keys would be a fragile local optimization.

Introduce compiled domains: semantic text/value, style, routes, collection models, typed canvas,
resource/media and geometry. Each domain has an exact generation and changed reason.

### P1: status-right inheritance allocates a HashSet per matching node

At most nine status controls walk toward `WorkbenchWindowStatusBar`. Each call creates a `HashSet` to
detect a parent cycle. `ProjectionNodeIndex` already owns all N nodes; a valid simple parent chain can
have at most N nodes. A traversal bounded to N lookups rejects cycles and malformed overlong chains
without allocation. M1 can apply this independently; M2 should cache the resolved status parent in the
generation index.

### P1: resource and collection resolution repeat on full projection

The root calls preview-image resolution for every node and rebuilds options text, option models,
structured options, menu rows and collection rows unless one notification-specific cache hits. Exact
resource and collection generations must reuse immutable outputs. Stable media must do zero path,
decode/load or size work.

### P2: defaults and mount arithmetic are bounded

Default component role/radius/border decisions are fixed match tables. Mount projection scales a
fixed field set once per changed node. These constants are not priority bottlenecks; their main need is
receipt separation so mount-only changes reuse the semantic artifact.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeMetaData.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeDescriptor.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal registers attributes in persistent widget metadata, retains getter items and orders them by
descriptor/prerequisite. Attribute descriptors own the value-change callback and exact
`EInvalidateWidgetReason`; registration/value changes invalidate the owning widget with that reason
instead of converting every attribute into a generic map (`SlateAttributeMetaData.cpp:36-57`,
`105-196`; `SlateAttributeDescriptor.cpp:11-30`). The invalidation root then rebuilds only for the
slow-path reason or updates the fast invalid list (`SlateInvalidationRoot.cpp:356-424`).

The transferable invariant is typed persistent attributes plus precise invalidation ownership, not a
per-frame generic value-to-TOML translation layer.

## Target architecture

1. Compile `RetainedUiHostNodeModel` into immutable semantic, style, route, collection, typed-canvas,
   media and geometry artifacts at the template/property owner.
2. Attach exact document, node, property-domain, style/theme, resource/text and geometry generations.
   A host node receipt references those owners.
3. Use previous-node identity to reuse every unchanged artifact. Full projection creates output rows
   only for changed receipts; sparse work remains a persistent row overlay.
4. Replace TOML conversion in the hot projection path with typed style inputs. TOML remains only at
   authored document parsing/config boundaries.
5. Publish collection/notification/menu/option models separately and virtualize large collections.
   Text joins are diagnostics or small presentation fields, not cache keys.
6. Cache parent visibility, nearest control and status-owner resolution together in one generation
   index. Parent queries allocate nothing.
7. Separate mount/scale geometry patches from semantic rows and resource handles.

Complexity targets:

- unchanged node: O(1) receipt comparison, zero TOML/property/style/resource/collection work;
- changed property domain: O(changed properties/output), unrelated domains zero;
- full generation: O(N + changed property bytes), not O(all property bytes every time);
- parent/status queries: O(1) after O(N) index build, zero allocation;
- mount-only change: O(changed geometry rows), semantic artifacts retained.

## M1 result

`ProjectionNodeIndex` now exposes its node count, and `status_right` uses a parent traversal bounded to
that count instead of a per-call `HashSet`. Valid chains return the same status-bar parent; missing
parents, cycles and paths that cannot terminate within N nodes return `None`. Existing cycle behavior
remains.

This removes one temporary hash table per matching status control. The source contract moved RED 1/1
to GREEN 1/1 and requires the explicit N-node bound. The two generation-build `HashSet`s in
`ProjectionNodeIndex` remain because they memoize all-node visibility/control ancestry once; M1 only
removes repeated per-status-node allocation. This is a bounded constant win, not a substitute for the
compiled typed-property architecture.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| host-value/TOML keys and bytes copied | stable = 0; hot-path conversion removed |
| property lookups/parses by domain | changed domain only |
| host node rows/artifacts rebuilt | changed receipts only |
| preview/resource and collection builds | unchanged = 0 |
| parent/status temporary allocations | 0 |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: nodes 0/1/1,000/10,000; properties per node 0/8/64/1,000; nested values 0/1KiB/1MiB;
depth 1/64/1,000 plus cycles; collections 0/10/10,000; stable projections 1/1,000; semantic, style,
route, collection, resource, mount, scale, state and render-only changes. Capture keys/bytes copied,
lookups/parses, artifact builds, allocations, CPU, latency, RSS and package energy on one fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc validates
current-source pixel/draw parity after a launchable editor exists; it cannot prove property-copy cost.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add property key/byte, TOML/style, resource/collection, node-artifact and allocation counters; capture baseline. | source-bound node/property scale evidence |
| M1 | Remove status parent-walk HashSet allocation. | RED-to-GREEN contract and cycle parity |
| M2 | Publish typed compiled property/style/route/collection/media artifacts and receipts. | stable property/TOML work = 0 |
| M3 | Reuse unchanged node artifacts and split mount/state patches. | changed receipts only; stable rows retained |
| M4 | Cache all parent-domain resolutions and virtualize large collections. | parent O(1); visible/budgeted collections |
| M5 | Delete hot-path TOML conversion and full-row reconstruction paths. | one typed attribute authority |
| M6 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full child-module source review: passed, 11/11 Rust files.
- Parent projection root, property consumers, persistent rows and Unreal references: read.
- M1 source implementation: complete. Its static contract moved RED 1/1 to GREEN 1/1.
- Related pane/scene/chrome/shell/apply/workbench source contracts: passed, 16/16.
- Changed Rust `rustfmt`, scoped diff check and plan-record audit self-test: passed.
- M0 and M2-M6 implementation and dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module remains in `pending.md` until M0-M6 pass on one source/executable fingerprint.
