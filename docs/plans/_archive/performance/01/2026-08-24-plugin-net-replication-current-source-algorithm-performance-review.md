---
title: Plugin Net Replication Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_shared_changes_preserved_dynamic_pending
scope:
  - zircon_plugins/net/features/replication/runtime
canonical_owners:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Public/ReplicationGraph.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Private/ReplicationGraph.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationSystem/ReplicationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationState/ReplicationStateDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Private/Iris/ReplicationSystem/ReplicationWriter.h
---

# Plugin Net Replication Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Replication feature completed E3 current-source static review over **24/24 Rust files** at revision `080fefe6acd449beded4497dee4a474b9e1f7383`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/features/replication/runtime` | 24/24 | 1,697 / 1,521 | 59,589 | 12 / 2 | `abb960d7e19b94e98dec3c0549194fb4fb3910507d0662ae1d59d740a67095e3` |

The fingerprint hashes sorted repository-relative path plus file bytes. Shared changes in `manager/apply.rs` and `manager/state.rs` were preserved. All 24 files parse through standalone rustfmt; 23/24 pass `rustfmt --check --edition 2021 --config skip_children=true`. The sole mismatch is existing import ordering in `manager/schedule.rs`. The scope passes `git diff --check`.

Managed Windows Cargo is unavailable, so none of the 12 tests ran. Product-call search across the other 152 Net Rust files found zero external manager/schedule/collect/apply caller. No real World, Reflection, connection or transport workload exists for WPR/ETW, and RenderDoc is not a CPU replication profiler. Dynamic scale, frame, wire, memory and power acceptance remain pending.

This optional feature is an in-memory algorithm model, not an accepted multiplayer replication product.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Feature/package surface | `capability.rs`, `feature.rs`, `lib.rs`, `plugin.rs` | Declares NetManager dependency but creates a private manager without consuming the dependency. |
| Manager facade and state | `manager.rs`, `manager/state.rs` | One global mutex owns descriptor, snapshot, sequence, interest, per-session schedule-time and interpolation maps. There is no World/connection generation. |
| Collect/apply/lifecycle | `manager/{collect,snapshot,apply,lifecycle,registry,table}.rs` | Compares caller-materialized raw fields, applies string/raw-byte deltas and exposes two inconsistent despawn paths. It is not wired to ECS dirty state or Reflection codecs. |
| Interest/schedule/budget | `manager/{interest,schedule,budget}.rs` | Scans and sorts all snapshots for every session/tick, then filters and sends full snapshots under payload-only budgets. |
| Tests | `tests/{budget,collect_apply,delta_interest,feature_registration,interpolation,lifecycle,mod,schedule,table}.rs` plus inline modules | Direct in-memory examples only; no transport loss/ACK, baseline, schema migration, real World, multi-connection, scale or malformed input qualification. |

The framework contract in `zircon_runtime/src/core/framework/net/sync.rs` was reviewed with the feature. `SyncReplicationStrategy`, `delta_compressed`, field type and authority are stored but not enforced by collect, schedule or apply. A default budget uses zero to mean unlimited, and byte accounting covers only field payloads.

## 3. Current-source local optimization assessment

Two local improvements are present:

- `schedule.rs` builds sortable candidates from object/component keys and delays snapshot cloning until after interest, due and budget checks. Its ignored 8,192-snapshot/4 KiB benchmark models removal of **32 MiB** of eager payload clones per candidate-build sample.
- The shared interpolation index changed from `(object, String, String)` keys to component/object/field nested maps. Borrowed queries no longer allocate two strings per lookup; the ignored fixture models **524,288 string allocations** removed across 4,096 keys and 64 query repeats.

These are directionally correct, but unexecuted. Candidate construction still clones component strings, scans every snapshot, fully sorts, and holds the global mutex. Final selected snapshots still deep-clone payload. The interpolation API still locks globally once per field query. Neither improvement changes product complexity or wire behavior.

## 4. Structural algorithm findings

### P1: the feature is disconnected from World, Reflection and authenticated connections

The factory ignores its declared NetManager dependency. There is no World/session owner, ECS query/change tick, reflection schema/compiler, transport codec, authenticated role or connection generation. Callers must materialize strings and full field byte vectors before the manager can compare them.

The target is one replication system per World/session generation attached to canonical authenticated connections. World extraction publishes dense dirty-object/member masks once per simulation tick; connection workers consume immutable metadata/state views. Do not create another manager, polling thread or reflection copy.

### P1: declared strategies and schema are inert metadata

`OnChange`, `Interval` and `Once` all pass through the same update-frequency schedule. `delta_compressed` is ignored. Authority, declared field names and value types are not validated on publish/apply. Unknown, duplicate or wrong-sized fields enter snapshots. Component/field identity is a repeated string; the table's alphabetical dense index changes when a lexically earlier component is added and has no schema/build version.

Editor26 must compile source descriptors into an immutable versioned artifact with stable component/field IDs, serializers, quantization/interpolation traits, authority/condition policy, default state, migration and compatibility hash. Runtime installs only a validated artifact for the active BuildSet.

### P1: collect performs full serialization and quadratic field matching

Each publish collects every field and linearly scans old fields for every new field, producing `O(F^2)` name comparisons. It increments sequence and returns a delta even when no field changed. Removing a field from the new full snapshot is not represented in `changed_fields`, so the replica retains the old field indefinitely.

Collect from Reflection/ECS dirty member masks and stable dense field IDs. Full capture is reserved for spawn/resync; update work is `O(dirty members)`. Empty changes produce no update, and collection/optional-field removal semantics are encoded explicitly.

### P1: schedule is `O(connections × snapshots log snapshots)` every tick

For every session and tick, the scheduler visits all snapshots, clones every key string, sorts all candidates by static priority/object/string, and only then checks interest and due state. Hidden/dormant/unchanged objects still pay candidate/sort cost. One global mutex covers the whole operation.

Maintain persistent global and per-connection replication state. Build candidate sets from dirty/new/lost/due bitsets plus spatial/owner/team/dormancy nodes; filter before prioritization. Use aging and bounded partial selection for the packet/object budget, and reuse scratch storage per connection. Parallel gather is allowed only after ownership and thresholds are explicit; the main thread must not wait on a global manager mutex.

### P1: scheduler sends full snapshots without baseline or delivery state

Although collect creates `SyncDelta`, schedule ignores it and clones full `SyncObjectSnapshot`. `last_replication_ms` is advanced when a snapshot enters a report, before encoding, transport admission or ACK. There is no per-connection known-object state, baseline, pending baseline, packet delivery notification, loss retry or resync.

Each connection needs spawn/create-confirmed/update/destroy state, acknowledged and pending baseline indices, dirty/lost masks, packet commit records and resync policy. Only ACKed delivery advances the usable baseline. Loss re-dirties the affected state with priority aging.

### P1: budget accounting can starve objects and undercounts wire cost

Budget counts only field payload bytes, excluding object/component/field IDs, framing, reliability, references and compression overhead. An oversized snapshot is deferred every tick forever with no split/huge-object path, terminal diagnostic or aging. Static priority can starve lower-priority state indefinitely. Zero means unlimited, so a default caller has no protection.

Budget the encoded bit/byte estimate plus packet/object limits. Add bounded huge-object fragmentation, fair priority aging and typed cannot-fit policy. Admission must report scheduled, encoded, transport-admitted, ACKed, lost, retried and dropped bytes separately.

### P1: lifecycle and cleanup are inconsistent and expensive

`collect_despawn_deltas` retains/increments sequence and emits tombstones; public `despawn_object` removes sequence and returns old snapshots without a tombstone. Respawning through the public path restarts at sequence 1, which a replica retaining a higher sequence rejects forever. Interest changes do not create per-connection spawn/despawn transitions.

Removing one object/component calls `last_replication_ms.retain`, scanning all session/object/component entries; despawning a multi-component object repeats that global scan. Session interests and last-replication rows have no connection teardown.

Use generation-qualified object handles and one authoritative lifecycle. Per-connection dense state makes remove/teardown proportional to that object's connections/components, not the global map. Re-entry after relevancy loss and respawn must have explicit create generation/baseline rules.

### P1: interpolation is inferred from names and arbitrary bytes

Any component string containing `transform` records every changed field with at least four bytes, interpreting the first four bytes as little-endian `f32` regardless of the field descriptor. A Vec3/Quat, integer, packed or dynamic field can therefore be interpolated incorrectly. `apply_delta` supplies receive time 0, and same-time samples may return an older value. Every render query locks the global state and performs nested hashes.

Interpolation policy belongs to the compiled field serializer/trait: type, quantization, time domain, interpolation/extrapolation mode, discontinuity/teleport and sample limit. Apply produces a generation-stamped presentation buffer once; rendering reads a lock-free/immutable snapshot in batches, not one locked string query per field.

### P1: retained state and observation have no product bounds

Snapshots, sequences, interests, session timing and per-field interpolation maps have no object/session/field/byte/age caps. `visible_snapshots` and late join deep-clone all matching payload under the mutex in nondeterministic map order. There are no high-water, clone-byte, dirty, candidate, cull, encode, ACK/loss or stale-generation diagnostics.

All stores need World/connection owner budgets and deterministic teardown. Late join uses the same incremental/budgeted create stream as normal replication. Observation publishes bounded counters and trace records without snapshot payload clones.

## 5. Unreal evidence and adopted policy

Unreal is the primary structural reference:

- `ReplicationGraph.h:14-27` explicitly maintains persistent actor lists plus global and per-connection actor information so work can be shared instead of scaling as a naive actor-by-connection scan. Lists are culled by distance/frequency, merged and prioritized.
- `ReplicationGraph.h:98-101,186-240,322-420` separates once-per-frame preparation from per-connection gather, keeps persistent node/frequency buckets and supports per-frame bit budgets, nearest limits and reusable sorted scratch storage.
- `ReplicationStateDescriptor.h:75-80,162-183,194-218,255-274` compiles member change-mask positions, serializers, traits, conditions, stable descriptor identity and dirty-mask storage rather than comparing free-form strings and bytes every tick.
- `ReplicationWriter.h:62-96,119-160` retains a compact state machine and dirty/ACKed/pending-baseline data per object per connection.
- `ReplicationWriter.h:184-210,263-310` schedules only dirty/new/resend work, processes packet delivery, retries objects that did not fit, partially sorts a bounded prefix and persists write context across packets.
- `ReplicationSystem.h:126-163,243-319,321-374` gives the system explicit object/connection limits, connection lifecycle, dirty/filter/prioritization update phases and a no-connection minimal-update path.

Zircon should adopt these boundaries, not copy Unreal types. The minimum viable algorithm is metadata-first: compiled schema -> World dirty set -> per-connection filter/relevancy -> priority/frequency/dormancy -> baseline/delta encode -> transport commit -> ACK/loss update -> presentation buffer.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Truth and regression freeze | Keep feature unavailable as product; preserve local clone/allocation improvements and add failing lifecycle/strategy tests. | Tests expose unchanged publish, field removal, Once/OnChange behavior, respawn sequence rejection, loss before ACK and oversized starvation. |
| M1 Compiled schema | Stable component/field IDs, serializers, change masks, authority/condition/interpolation traits and BuildSet migration rules. | Golden artifact/codec compatibility across builds; malformed/type/authority input rejected before state mutation. |
| M2 World extraction | Per-World object generations and ECS/Reflection dirty-member capture once per simulation tick. | Unchanged work is `O(0)` after fixed frame overhead; update work is `O(dirty objects + dirty members)`. |
| M3 Connection graph | Persistent per-connection known/relevant/dormant state and spatial/owner/team/shared filters. | Candidate work is proportional to changed/due/relevant sets, not all objects; connection teardown leaves zero rows. |
| M4 Baseline and lifecycle | Spawn/update/despawn, ACKed/pending baseline, loss retry, resync, late join and generation-safe respawn. | Loss/reorder/duplicate/interest churn preserves eventual state and never accepts a stale generation. |
| M5 Scheduling and encode | Aging priority, partial selection, encoded-bit budgets, huge-object path and reusable scratch outside main-thread locks. | No admitted object starves; encoded wire bytes obey policy; frame/main-thread blocked wall is zero. |
| M6 Presentation and observation | Typed interpolation buffers plus bounded per-connection/object trace and metrics. | No name heuristics/raw-first-f32 decoding; render queries allocate zero and take no global replication mutex. |
| M7 Dynamic qualification | 1/32/128 clients, 1K/10K/100K objects, churn/loss/late join/soak with current-source WPR/ETW. | Publish BuildSet-bound P50/P95/P99 frame/encode/ACK latency, CPU, RSS, candidate/dirty ratio, payload/wire bytes, wakeups and joules/useful update. |

Static current-source review is complete. Dynamic/product acceptance is pending, shared source work is preserved, and no Git milestone commit or quantified WeCom notification is warranted.
