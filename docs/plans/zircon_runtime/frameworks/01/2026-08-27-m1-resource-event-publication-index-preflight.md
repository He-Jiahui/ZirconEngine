# M1 Resource Event Publication Index Preflight

## Status

- Plan owner: Frameworks01
- Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`
- State: `source_reworked / ttl_concurrency_review_fixed / adversarial_profile_and_direct_tests_green / managed_cargo_foreign_blocked`
- Session base HEAD: `1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`
- Reviewed current HEAD: `ea35974cdf64068f6789010451d20bbf69e0a29d`
- Scope: `zircon_runtime/src/core/resource/event_stream.rs` publication lookup, coalescing, retention-index invariants, and focused tests
- Excluded: resource physical crate move, resource IO transactions, sequence exhaustion policy, editor projections, IBL implementation, replay/persistence, and a new public event API

This record follows the required order: review the whole current module, inspect reference-engine behavior, capture a
pre-change profile, lock the structural direction, then implement. It does not treat a standalone microbenchmark as
product frame-time, energy, or whole-engine proof.

## Current-Source Review

The current event stream is a bounded retained log with three independent limits:

- at most `4,096` retained events;
- at most `4 MiB` of approximate event payload;
- at most `60 s` of retained age.

Its externally observable invariants are more important than the container itself:

1. every publish reserves a monotonically advancing sequence number;
2. receivers observe retained events in sequence order;
3. a missing sequence is surfaced as `Lagged`, including a gap created by coalescing;
4. `Added` and `Updated` may replace the latest retained event for the same `(ResourceKind, ResourceId)` only when
   that latest event is also coalescable;
5. lifecycle edges such as `Removed`, `Renamed`, and `ReloadFailed` stop coalescing across the edge;
6. TTL, entry-capacity, and byte-capacity eviction update diagnostics and receiver gap behavior;
7. poisoned locks recover their inner state rather than changing the public error model.

The input shared blob was an uncommitted Runtime64 cursor optimization with coordinator SHA-256
`92fb55be67092f0a13247049faf762512478b3df7a8cbcef2f6d3ef24e89afb7`. Ownership transfer preview
`ced62c9ba3694d2bbbc055e2daa2bba1` and apply request `80565aea3fb14182880c82f3749ddfd1` transferred that exact blob
from archived session `root-runtime64-resource-event-cursor-20260824` to r12 before production editing. The first r12
implementation retained `VecDeque`, added a latest-identity `HashMap`, and passed its original profile. It is now an
explicitly rejected intermediate, not the accepted design, because the workload never exercised arbitrary-position
deque removal. Current source was edited only after lease request `d07220de0c044c7ea1b1199014ddd272` acquired the
production, test, and record paths. No offline attribution, direct Git staging, or foreign-file rewrite was used.

The remaining publication bottleneck is structural:

```rust
state.entries.iter().rposition(|existing| same_identity(existing, event))
```

Every `Added` or `Updated` publish scans backward through up to 4,096 retained events. If the matching event is
coalescable, `VecDeque::remove(index)` then moves the shorter side of the deque. Complexity is therefore:

- no matching retained identity: `O(N)` comparisons plus amortized `O(1)` append;
- matching identity: `O(N)` search plus worst-case `O(N)` contiguous movement;
- receiver cursor lookup and receiver `len()`: `O(log N)` after the Runtime64 source change;
- retained storage: bounded `O(N)` with `N <= 4,096` and a separate `4 MiB` byte cap.

The fixed cap prevents unbounded growth but does not make a full scan cheap inside the global resource mutation lock.
At a broad working set, the scan multiplies lock hold time for every publisher and delays all receivers.

## Reference-Engine Review

### Unreal Engine

Unreal's Asset Registry is not the same retained-stream contract, so its code is routing evidence rather than an API
copy target. The relevant design choices are nevertheless explicit:

- `AssetDataMap.cpp` stores asset pointers in stable slot-like storage and maintains an `AssetByObjectName` hashed
  lookup; `Add`, `Remove`, and `FindId` resolve the identity through `FindByHash`/`RemoveByHash` instead of scanning all
  assets;
- `AssetRegistryImpl.h` defines `FEventContext::AssetEvents` as transaction-local ordered event data collected while
  the registry write lock is held;
- `AssetRegistry.cpp` drops the lock before `Broadcast(EventContext)`, pre-counts event types, reserves batch buffers,
  preserves remove/non-remove ordering boundaries, and publishes batched and single events;
- public batch delegates are first-class (`OnAssetsAdded`, `OnAssetsRemoved`, `OnAssetsUpdated`, and
  `OnAssetsUpdatedOnDisk`).

The applicable Zircon principle is to use an identity index for authoritative resource lookup and keep event ordering
as a separate concern. Zircon additionally retains events for independent receivers, so it cannot replace the log with
Unreal's transaction-local array.

### Bevy

`dev/bevy/crates/bevy_asset/src/assets.rs` separates asset identity storage from event delivery:

- dense runtime IDs use dense storage while UUID assets use a `HashMap`;
- mutations append to `queued_events: Vec<AssetEvent<A>>`;
- the `asset_events` system updates `AssetChanges` through keyed insert/remove operations, then drains the queued
  events with one `write_batch` call;
- the system does not run when the queue is empty.

The applicable principle is again keyed state plus explicit event batching. Bevy's queue has a frame/system lifetime,
so it is not a substitute for Zircon's bounded sequence log and lag reporting.

## Pre-Change Profile

The benchmark directly `include!`s the current production `event_stream.rs` into a minimal interface-type harness and
drives the real `ResourceEventPublisher::publish` path. It was compiled with
`rustc +1.94.1 --edition 2021 -C opt-level=3 -C target-cpu=native -C debuginfo=1`. Each table value is a median; short
runs use 11 samples, medium runs 7, and 100,000-event runs 5. `black_box` prevents the result from being removed.

Environment and artifacts:

- Windows x86_64, AMD Ryzen 7 5800H, 8 cores / 16 logical processors;
- Rust `1.94.1` (`e408947bf`, LLVM `21.1.8`);
- source and outputs: `D:\zircon-frameworks01-r12-resource-event-publish-20260827`;
- harness source SHA-256: `8e3b4ffcaf23cc96019833371cb6250c40f42b102463a32b7a1a25cfc1734f72`;
- executable SHA-256: `5782cca1ad80c65b925d7f3ca79ec7f78ad3091dd15707ca8fb7f27ddd0d73f5`;
- pre-profile CSV SHA-256: `12aee7c967c89b59b6623acc88f4d449072f10622394ca35ce106728c6d05318`.

| Publishes | Active identity working set | Median total | Median per publish |
| ---: | ---: | ---: | ---: |
| 1,000 | 1 | 91.2 us | 91.200 ns |
| 1,000 | 64 | 140.7 us | 140.700 ns |
| 1,000 | 1,000 | 500.5 us | 500.500 ns |
| 10,000 | 1 | 926.9 us | 92.690 ns |
| 10,000 | 64 | 1.542 ms | 154.210 ns |
| 10,000 | 4,096 | 54.586 ms | 5,458.570 ns |
| 10,000 | 10,000 unique | 45.706 ms | 4,570.580 ns |
| 100,000 | 1 | 11.779 ms | 117.790 ns |
| 100,000 | 64 | 17.329 ms | 173.290 ns |
| 100,000 | 4,096 | 567.083 ms | 5,670.829 ns |
| 100,000 | 100,000 unique | 725.043 ms | 7,250.434 ns |

At 100,000 publishes, a 4,096-key working set is `48.14x` slower per publish than the one-key case; the all-unique
case is `61.55x` slower. The unique case never removes a matching entry, so its `7.25 us` result isolates the reverse
identity scan rather than deque movement. This is sufficient to reject implementation-level tuning of the current
loop.

The first post-profile was not sufficient to accept the indexed `VecDeque` replacement. Its 4,096-key case repeated
the same identity order every cycle, which means each previous event was at the deque front and `remove(0)` took the
best-case path; the unique case did not remove anything. A second workload first fills all 4,096 identities and then
uses a deterministic xorshift identity stream. Against the exact indexed-deque intermediate it measured:

| Publishes | Pattern | Median per publish |
| ---: | --- | ---: |
| 10,000 | random retained 4,096 | 5,866.950 ns |
| 100,000 | random retained 4,096 | 6,018.244 ns |

The same intermediate measured `270.692 ns` for the ordered 100,000/4,096 case, so the missing workload hid a
`22.23x` order sensitivity. Executable `resource_event_publish_profile_random.exe` has SHA-256
`67a060689ed8a8b79047456e5bc201dd8b12131d38dcee11d7eba1112b771e9b`. This evidence invalidates the earlier
`accepted r3` wording and confirms that indexed lookup alone did not remove the publication algorithm's O(N) payload
movement.

Windows Performance Recorder CPU sampling was attempted with the installed `CPU` profile and rejected before a
recording began with `0xc5585011` because this host lacks the required system-profile privilege. `wpr -status`
confirmed no active recording remained and no ETL was produced. No alternative user-mode sampling profiler or energy
counter is installed. Consequently this preflight claims source-level attribution and controlled latency only; it does
not claim call-stack percentages, product power reduction, frame-time reduction, or parity with another engine.

## Locked Design

Hard-cut the contiguous queue to one bounded intrusive slot log:

```text
slots: Vec<Option<Node { entry, previous_slot, next_slot }>>
latest_slot_by_identity: HashMap<(ResourceKind, ResourceId), usize>
recent_slot_by_sequence: [Option<(sequence, slot)>; 4096]
oldest_slot / newest_slot / free_slots
```

The slot array is the event-order authority. It reuses removed slots without moving any retained payload. The identity
map is a verified derived index for publication; the fixed recent-sequence ring is a verified derived index for the
normal receiver cursor path. A collision or stale derived entry fails closed to the bounded linked-order walk.

Publication algorithm:

1. evict expired head nodes and clear an identity mapping only when it still points to the removed slot;
2. compare the newest slot before probing the identity map, preserving the single-hot-key path;
3. verify the candidate slot's exact identity and lifecycle kind before unlinking it;
4. unlink through previous/next slot indices in O(1), update byte/coalesce diagnostics, and push the slot to the LIFO
   free list;
5. reserve the new sequence and immediately reuse the just-freed slot when coalescing;
6. retain the verified identity mapping when the same slot is reused, avoiding a redundant hash remove/insert on the
   hot path; otherwise publish the new mapping;
7. write the `(sequence, slot)` recent-ring entry, then enforce entry and byte capacity from the linked head.

Expected complexity after the change:

- tail coalesce: O(1) with no identity-map mutation;
- non-tail coalesce: expected O(1) identity lookup plus O(1) unlink/reuse;
- unmatched append and head eviction: amortized/expected O(1);
- normal exact receiver cursor: O(1) through the fixed recent-sequence ring;
- collided or older-than-ring lag recovery: O(N), bounded by the exact 4,096 retained-entry cap, after which normal
  exact reads return to O(1);
- `len()`: O(unread entries), also bounded by 4,096; current production has no `len()` caller, while all receiver event
  delivery paths use `take_next()`;
- storage: O(N), with one pre-sized recent ring, one bounded identity map, and no tree-node allocation per publish.

`BTreeMap<u64, Event>` was prototyped and rejected. It reduced the 100,000 random-retained case to `978.579 ns`, but
regressed the single-key case to `218.759 ns` (`+85.72%` from the original pre-profile) and the unique case to
`860.950 ns` because every event incurred tree removal/insertion and node allocation. Tombstones remain rejected
because they weaken physical byte/entry bounds or require periodic O(N) compaction. The slot log instead matches the
stable-slot plus keyed-lookup direction in Unreal's Asset Registry while preserving Zircon's retained receiver model.

## Implementation And Post-Profile Evidence

The hard cut is implemented without a compatibility path:

- `ResourceEventLogEntries` owns reusable linked slots, head/tail/free-slot state, and the fixed recent-sequence ring;
- `ResourceEventLogState` owns `latest_slot_by_identity`; publication contains no reverse scan, contiguous payload
  move, `VecDeque`, or `BTreeMap`;
- every derived slot is verified against the authoritative event before coalescing or receiver delivery;
- ring collisions fall back to linked sequence order, and removing the older colliding node cannot clear the newer
  mapping;
- test ownership was mechanically split through the existing leaf: production is 778 lines and
  `publication_index_tests.rs` is 462 lines, both below the 800-line soft budget.

Current source hashes:

- `event_stream.rs`: `a56ec7096ab1d4cc7330afd356855a0c9f3c5e7ebec16bc991cee2d7f47e13c4`;
- `publication_index_tests.rs`: `e93a89c65b94728d9432af4481d64ec408d148187c97c74b27029960cd080712`.

Independent review found one concurrency defect after the first exact snapshot: `publish` sampled `Instant::now()`
before acquiring the state mutex, so two publishers could append a newer timestamp followed by an older timestamp
while TTL eviction assumed head-to-tail timestamp order. The focused source-order regression failed before the fix and
passed after publication-time sampling moved inside the state lock. Sequence assignment, list insertion, and TTL time
now share the same serialization point; this is a correctness repair, not a compatibility path.

The final exact-current benchmark uses the same compiler, flags, sample counts, original cases, and the added
random-retained cases:

| Publishes | Active identity working set | Pre median per publish | Post median per publish | Change |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 1 | 91.200 ns | 128.500 ns | 40.90% higher |
| 1,000 | 64 | 140.700 ns | 145.400 ns | 3.34% higher |
| 1,000 | 1,000 | 500.500 ns | 131.500 ns | 73.73% lower |
| 10,000 | 4,096 | 5,458.570 ns | 146.090 ns | 97.32% lower (`37.36x`) |
| 100,000 | 1 | 117.790 ns | 142.676 ns | 21.13% higher |
| 100,000 | 4,096 | 5,670.829 ns | 162.991 ns | 97.13% lower (`34.79x`) |
| 100,000 | 100,000 unique | 7,250.434 ns | 243.694 ns | 96.64% lower (`29.75x`) |
| 100,000 | random retained 4,096 | 6,018.244 ns indexed-deque | 228.622 ns | 96.20% lower (`26.32x`) |

Post-profile artifacts:

- harness source: `resource_event_publish_profile.rs`, SHA-256
  `0ea00c3ac2b22795b771249da268ba0f6850723157d5bc512c30677bf6b56ae0`;
- executable: `resource_event_publish_profile_slot_ttl_final.exe`, SHA-256
  `eb6f778c0b8bcf2b868ac7e1d240b3786a82582855c07c069baf613100473e85`;
- CSV: `post-profile-slot-ttl-final.csv`, SHA-256
  `deda416bfe578dfbab7bac7b5d67ff024619280e9fbc34c38f96d083da979418`.

The 1,000/single-key case is short enough to remain noisy and is not the locked regression gate. The locked
100,000/single-key result is `+21.13%`, below the `25%` limit. The random/ordered 4,096-key ratio is now `1.40x`
rather than `22.23x`; order no longer exposes an O(N) payload move, although cache locality remains measurable.

A standalone `rustc --test` harness compiled the real current production file and its private test submodules. Result:
`20 passed / 0 failed / 1 declared managed-performance test ignored` in `0.50 s`. The dedicated test harness source
SHA-256 is `74f8bfaa99e55ef6fa2a8a03539016fffd190e49c3e4c9af3ec13631d5c9cb02`; executable SHA-256 is
`ef702cc9d390c37d9ef68d84432a2be53ac34a1e85575e5a2ec45da53cfa843d`. This proves focused behavior and compilation,
not full-package integration. Running the ignored cursor evidence separately also passed: 4,096 entries, 250,000
queries, `24.5235 ms` elapsed against a `500 ms` bound. It remains direct evidence, not the managed release gate.

## TDD And Acceptance Gates

Implementation and managed acceptance cover these invariants:

1. the state owns a latest-identity slot index and publish contains no reverse scan, `VecDeque`, `BTreeMap`, or
   arbitrary contiguous removal;
2. same-identity `Added`/`Updated` still coalesce and increment `coalesced_count`;
3. `Removed`, `Renamed`, and `ReloadFailed` prevent coalescing across the lifecycle edge;
4. entry-capacity, byte-capacity, and TTL eviction do not leave an index pointing at an evicted slot;
5. eviction of an older event must not erase a newer index mapping for the same identity;
6. sparse sequence gaps and receiver `len()`/`take_next()` retain behavior through the recent ring and bounded linked
   fallback;
7. mixed resource kinds sharing the same `ResourceId` never alias;
8. a differential test compares the indexed implementation with a simple scan-based semantic oracle over mixed
   publish/eviction sequences;
9. recent-ring sequence collisions return the correct linked successor and deleting the older collision does not clear
   the newer ring entry;
10. both production and focused test owners remain below the 800-line soft budget.

Post-change performance uses the exact same harness and environment. Acceptance requires:

- 100,000 publishes with working set 4,096 are at least `80%` below the `567.083 ms` baseline;
- 100,000 unique publishes are at least `80%` below the `725.043 ms` baseline;
- both broad-working-set cases remain below `1,500 ns` per publish median;
- working-set 1 does not regress by more than `25%` from `117.790 ns` per publish;
- 100,000 random-retained 4,096-key publishes are at least `90%` below the indexed-deque `6,018.244 ns` blind-spot
  baseline and remain below `1,500 ns` per publish;
- output reports exact source, executable, and CSV hashes and does not convert microbenchmark results into power or
  whole-product claims.

All five direct performance gates are met. Focused direct behavior and compilation are green. Managed package/product
integration is not green and therefore the milestone is not accepted.

The predecessor Tooling bootstrap waves exited naturally and the official coordinator artifact audit reported
`unmanaged: []`. Managed Windows job `433c40bf07154cbf9bc1d712f94dcf09` then ran the `zircon_runtime` `core-min`
package check in
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
It terminated with exit code `101` after approximately `409 s`: rustc reported `55` errors and `123` warnings. The
729,706-byte structured diagnostic artifact has SHA-256
`2f382d9f1dafec05d780c65eebff976cc8db244304e7bbe3e851facbcf622694`. None of its diagnostics names
`core/resource/event_stream.rs` or `publication_index_tests.rs`; this absence is not promoted to package acceptance.

Following support-first regression rules, the package-level failure was decomposed into the lowest visible current
source groups before considering any patch:

- asset render-manifest/material/shader imports, visibility, ownership, and move semantics: 7 diagnostics;
- builtin runtime-module assembly/composition visibility: 4 diagnostics;
- core framework lifecycle/render const, IBL range, frame-extract, and submission contracts: 11 diagnostics;
- runtime task diagnostics/pools/task-graph ownership and borrowing: 7 diagnostics;
- platform host/window-registry identity, visibility, const, and move semantics: 12 diagnostics;
- native plugin discovery-root contract: 3 diagnostics;
- scene schedule/removal/change-journal visibility, lifetime, and error conversion: 11 diagnostics.

The ownership matrix did not show an executable current owner for these exact compiler-error blobs: most have missing
attribution, while the remainder point to archived or cancelled stale attribution. They are outside this focused
ResourceEvent slice and several are outside its immutable write scope. Frameworks01 therefore did not add a facade
compatibility export, rewrite a foreign blob, or launch a second Cargo job. Upward package and product regression must
resume after the corresponding domain owners close their lowest shared migration layers. Direct `cargo` remains
forbidden, and all future build-producing validation must retain a non-C target directory.

## Ownership And Adjacent Boundaries

- Production input ownership was transferred exactly before editing. The final source/report attribution refresh and
  live ownership-matrix recheck remain required after this record update; current path leases were acquired through
  request `d07220de0c044c7ea1b1199014ddd272`.
- The broader zr_resource physical hard cut remains blocked by executable foreign attribution on 62 current inputs;
  this focused production optimization does not rewrite those files or claim physical-cut acceptance.
- `crate::core::resource::io::{atomic_write, atomic_write_new}` remains a supported curated public facade. Of UI12's
  three reported IBL fingerprints, `ibl_bake_artifact_asset_derived.rs` and `ibl_bake_artifact_cache.rs` remain valid
  consumers; `ibl_source_cubemap_staging.rs` no longer imports `atomic_write` and has a source guard rejecting it.
  Deleting the facade export would regress the two valid consumers; Frameworks01 did not rewrite any IBL blob.
- `zircon_editor/src/ui/host/editor_event_runtime_access.rs` and
  `zircon_editor/src/ui/workbench/project/asset_workspace_state.rs` remain untouched mixed blobs pending their Editor01
  and Editor09 ownership transfers.
- This preflight is not milestone acceptance and does not authorize a commit or WeCom completion notification.
