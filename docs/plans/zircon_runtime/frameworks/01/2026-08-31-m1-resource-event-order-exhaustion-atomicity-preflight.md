# M1 Resource event order exhaustion and atomicity preflight

Status: `production_fix_implemented / focused_static_green / managed_build_green /
focused_and_full_behavior_pending /
editor_consumer_pending`.

## Scope and decision boundary

This preflight owns the correctness boundary between a committed Resource mutation and its bounded
event-log order. It covers sequence exhaustion, receiver cursor terminal state, commit-time sequence
admission, publisher lifetime, gap reporting, and the lock order needed to preserve an infallible
`PreparedResourceMutation::commit` after successful prepare.

It does not redesign the already profiled intrusive slot log, change event coalescing policy, add
event persistence or replay, or claim a runtime performance improvement. Production editing remains
held until the same-source ResourceManagement/readiness profile produces its current baseline.

The reviewed current-source fingerprints are:

- `zr_resource/src/event_stream.rs`
  `636941a86d044ee06c523b9c9c0daed3cb874d8ebcd4c9d1976d8707784e6015`;
- `zr_resource/src/event_stream/publication_index_tests.rs`
  `b3cd778337fa5df6216fbae0e22210963a4bafe60c85fbf9d284f3237fa63a02`;
- `zr_resource/src/manager/commit.rs`
  `10ce59851595e975b01b7a6b9f24451a1f4f4c5ec0fcff94e2c7100375eb38e5`;
- `zr_resource/src/manager/lease_ops.rs`
  `f713a8af183fadadfc65013b7223c792026937c6698ef0752779a7ab3475f624`;
- `zr_resource/src/manager/payload_ops.rs`
  `02f91c140762e49b93b2c3f98f38f66d7efa2c0aedb37be20526f74a464605cb`.

## Current defects and invariants

The current stream starts at sequence 1 and advances both publisher and receiver cursors with
`wrapping_add`. After `u64::MAX`, a new event reuses sequence 0 and an old cursor can compare equal
to a new lifecycle. The recent-sequence ring, ordered lookup, gap detection, and subscription cursor
all assume a strictly ordered scalar and cannot distinguish that reuse.

The current empty-log gap path also projects `state.next_sequence` as an always-valid
`oldest_available_sequence`. At a real terminal there may be no successor, so another integer is a
false sentinel. Changing the counter to `u128`, an epoch pair that can also wrap, or a saturating
counter would delay or hide the same correctness defect.

Publisher disconnection is tracked by an independent `AtomicUsize`. `Clone::clone` uses unchecked
`fetch_add`; `Drop` uses `fetch_sub`. That manual lifetime counter can overflow or underflow and is a
second identity authority beside the existing `Arc<ResourceEventHub>`.

The mutation path adds a stricter atomicity constraint:

1. `prepare_commit` obtains `commit_serial` and constructs exact staged before/after records;
2. `commit` mutates Resource authority under its write lock;
3. only after releasing Resource authority does it publish the derived events;
4. `commit_serial` remains held until all events are published.

Repository-wide call-site review found no second production event publisher: the only
`publish_event` call is the event loop in `manager/commit.rs`. Acquire and lease release can refresh
readiness, but do not publish Resource events. Therefore the commit serial is the valid sequence
admission authority. A fallible sequence increment after step 2 is forbidden because it would leave
a committed mutation without its event. Advancing the sequence during prepare is also forbidden
because a dropped prepared mutation would create a false receiver gap.

## Reference-engine review

The primary reference remains Unreal Asset Registry:

- `dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h`,
  SHA-256 `08c154ce35bced25eed8c415bbf0f6386905bcf978d35a92711fa1d1bf2c336f`;
- `dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp`,
  SHA-256 `071f916324ff4e714329416559b3dce37be493b9df22a3dcb98988c58a4ca9fc`.

Unreal accumulates exact changes in an `FEventContext` while registry mutation is protected, then
broadcasts after the registry lock is released. It does not invoke arbitrary subscribers while the
authoritative registry write lock is held. Zircon should preserve that separation: prepare exact
event data under the serial mutation boundary, mutate Resource authority, release that authority,
then publish the prepared batch.

Unreal does not provide Zircon's retained cursor/exhaustion contract, so its delegate machinery is
not copied. Zircon keeps its bounded slot log, coalescing boundaries, lag diagnostics, and independent
receivers.

## Locked terminal model

The order authority hard-cuts to an explicit terminal state:

```text
next_sequence: Option<u64>
receiver_cursor: Option<u64>
```

`Some(sequence)` means that exact sequence is the next publish/read position. `None` means sequence
space is terminal. Publishing `u64::MAX` is valid exactly once; its successor is `None`. No reset,
epoch rollover, compatibility cursor, or numeric sentinel survives.

The receiver contract is:

- an existing receiver at `Some(u64::MAX)` may consume the final retained event, then moves to
  `None`;
- a receiver at `None` returns typed `SequenceExhausted` from try, blocking, and timeout APIs;
- a receiver that lost events before the terminal first receives `Lagged`, then
  `SequenceExhausted` on its next read;
- a subscriber created after terminal subscribes at `None` and does not replay retained history,
  preserving the current tail-subscription contract;
- `Disconnected` remains distinct and is used only when publisher lifetime ends before order-space
  exhaustion.

`ResourceEventGap::oldest_available_sequence` becomes `Option<u64>`. `Some(sequence)` directs the
receiver to the oldest retained or future successor. `None` records that the expected event was
lost and no successor can ever exist. No compatibility constructor keeps the old mandatory scalar.

Diagnostics add an explicit `sequence_exhausted` flag and a saturating
`rejected_publish_count`. Rejected count is diagnostic only; it never authorizes a reset or changes
mutation semantics.

## Locked prepare/commit protocol

`prepare_commit` must establish event admission before Resource authority can change:

1. acquire `commit_serial`;
2. preflight the mutation against a Resource authority read guard;
3. derive the exact ordered `Vec<ResourceEvent>` from staged before/after records;
4. after dropping the Resource read guard, inspect event order under the event mutex;
5. if the exact batch does not fit before terminal, increment rejected diagnostics and return typed
   `ResourceRegistryError::EventSequenceExhausted` before any Resource mutation;
6. otherwise return a private, non-cloneable `ResourceEventPublishPermit` with the exact first
   sequence, event count, and optional successor, without changing `next_sequence`.

Holding `commit_serial` makes this non-mutating permit exclusive: no production path can publish a
sequence before it is consumed. Dropping `PreparedResourceMutation` drops the serial guard and the
permit without changing event state, so receivers observe neither a pending reservation nor a gap.

`PreparedResourceMutation::commit` remains infallible after prepare:

1. consume the staged mutation and exact event vector;
2. apply Resource registry, payload, runtime, management, and readiness changes under the Resource
   write lock;
3. release the Resource write lock;
4. consume the permit and insert the exact event vector under the event mutex;
5. advance `next_sequence` to the permit successor, possibly `None`;
6. release the event mutex, notify receivers, then release `commit_serial`.

No Resource and event mutex are held at the same time. The lock order stays
`commit_serial -> Resource authority`, followed later by `event state`; subscribe, receive, and
diagnostics take only event state. This preserves Unreal's mutate-then-broadcast separation and
avoids adding an event-to-Resource reverse edge.

An event-free prepared mutation is valid after sequence exhaustion because it has no event to lose.
An event-producing mutation is rejected before authority change. Raw `publish_event(event)` is
removed so production publication cannot bypass the permit.

## Publisher lifetime hard cut

The manual `publisher_count` is replaced by one shared publisher-lifetime object:

```text
ResourceEventPublisher -> Arc<ResourceEventPublisherLifetime>
ResourceEventReceiver  -> Weak<ResourceEventPublisherLifetime> + Arc<ResourceEventHub>
```

The lifetime object's `Drop` locks event state only to order the final notification, then wakes all
receivers. Receivers detect disconnection under that same event-state lock with the non-owning
`Weak::strong_count() == 0` observation. They must not call `Weak::upgrade()` there: a racing
temporary strong owner could become the final owner and re-enter the same mutex from lifetime
`Drop`. Publisher clone/drop no longer mutates a counter and cannot overflow. The hub remains alive
through receiver ownership after all publishers are gone. No second boolean or count is retained as
a compatibility authority.

## RED/GREEN matrix

Production implementation is authorized only with these focused cases:

1. the final valid event receives sequence `u64::MAX` exactly once and an existing receiver then
   reports `SequenceExhausted`;
2. a subsequent event-producing commit returns the typed exhaustion error and leaves registry,
   payload, management generation, readiness generation, receipt count, and event log unchanged;
3. preparing and dropping a mutation does not advance a subscriber cursor or produce a false gap;
4. a receiver waiting across prepare/commit receives the committed event, never an intermediate
   reservation;
5. a terminal lag reports `oldest_available_sequence=None`, followed by `SequenceExhausted`;
6. coalescing and entry/byte/TTL eviction near the terminal preserve existing gap behavior and never
   reuse a sequence;
7. event-free mutations remain valid after terminal while event-producing mutations fail closed;
8. cloned managers keep receivers connected, and dropping the final publisher wakes blocking and
   timeout receivers without an atomic count;
9. static guards reject `wrapping_add`, `publisher_count`, a raw production publish bypass, numeric
   terminal sentinels, and public compatibility overloads.

Tests belong in a new folder-backed event-order leaf rather than growing the existing 610-line
publication-index owner. Production and each test owner must remain below the 800-line structure
budget. Rustfmt, scoped whitespace checks, focused event tests, full `zr_resource` tests, current
ResourceManagement/readiness profile, and upward product validation are required before acceptance.

## Acceptance state

The owned production protocol is implemented. Event state and receiver cursors use `Option<u64>`;
`u64::MAX` is published once before terminal `SequenceExhausted`; terminal gaps carry
`oldest_available_sequence=None`. A private non-mutating publish permit is admitted while
`commit_serial` is held and is consumed only after Resource authority mutation, so an insufficient
sequence range rejects before any registry/projection change and a dropped prepare consumes no
sequence. Publisher connection now follows `Arc/Weak` lifetime instead of a manual count. Current
source SHA-256 includes:

- `event_stream.rs`
  `8b72d3e7a501d01f58c0df7db715de0ada40663a32f45eb6cae9330bec723a7a`;
- `event_stream/event_order_tests.rs`
  `1db9206210eaac5a7faa309286eda1252242169d586c04a261da7613d6ba60d4`;
- `event_stream/publication_index_tests.rs`
  `ced53f929a4609f08927fb66a4e13b20afa467f678a3cc16a8d4ef9c6cb1398c`;
- `manager/commit.rs`
  `e227c5b5b2f053430ac712a454489e1ec2a54f50755ac47a63b4479672d1be5a`;
- `error.rs`
  `e9be4161ea8132b26146ba8b315b6ce36238b93c704299858c6bd8facd606678`;
- Runtime dynamic-scene event projection
  `8a8c3967a4fc99d85f52bf27584399114db1636ee51178d8d6fc15b5b1c82e36`.

Focused boundary cases cover final sequence publication, terminal lag, all receive modes, exact
batch-range admission, event-free post-terminal commits, dropped prepared mutations, atomic
rejection, and final publisher wakeup. The Frameworks resource static suite is GREEN `14/14`.
The exhaustive Editor asset-refresh consumer is a mixed Editor blob and has been routed to its
legal Editor owner; Frameworks01 did not modify it.

The RuntimeInterface03 E0277 is fixed by its exact owner. Managed job
`6486a2ea6b664b2ba0130ab61193090b` completed a GREEN `zr_resource` production build and linked a
lib-test binary; after the stale two-argument `apply_staged` source guard was updated, follow-up job
`8295343708a64b3c91a2bf7feda1c96e` again built GREEN but full test exited `1`. Persistent direct
libtest output from current binary SHA-256
`361E13E4608E2A61E9F67E908F52E53E7353DD9DA366A50AD9EF1C6B286B9202` identified the exact stall as
`blocking_resource_event_receiver_wakes_when_the_last_publisher_is_dropped`.

Root cause is structural: `is_disconnected()` called `Weak::upgrade()` while holding the event-state
mutex. If publisher Drop raced that check, the receiver's temporary `Arc` became the final strong
owner and its destruction invoked `ResourceEventPublisherLifetime::drop`, which tried to re-lock the
same mutex before notifying the Condvar. The fix uses non-owning `Weak::strong_count() == 0` under
the state lock; final Drop still locks state before `notify_all`, preserving the no-lost-wakeup
handshake without a temporary owner. The regression now has a one-second result-channel bound, and
the static guard rejects a future locked `upgrade()`. The complete Frameworks resource boundary
suite is GREEN `14/14` in 13.640 seconds. The latest managed-storage helper/test hashes were verified
as `7D1EB4FE2BAD2FB7BC124EFCAC272C187226B9A6F52DBDF9C86E4CD5342F74D9` and
`4798293A9503186B1917AA5DC5074BBBC005DACD866868366F4EB529D1502CC9`; the four storage-related
artifact-governance cases are GREEN `4/4` in 14.366 seconds.

The exact focused managed rerun is GREEN: job `e3455c566edb4766a59d2d8e19c0de98`, build/test exit 0,
released at `11:48:47`, with successor session `...:e4922acede72474f92d1698e953ac702`. The exact full
rerun `7111bf72605e4181ab5e46ab695228c6` built successfully but reported libtest exit 101 without a
retained failing test line. Its exact binary was subsequently run from the same E-drive target in
one-thread and five default-concurrency repetitions, each returning `218 passed / 0 failed / 11 ignored`
(one-thread 13.38 seconds; parallel 2.30--4.12 seconds). This diagnostic evidence cannot replace a
coordinator-native full receipt, so the full result remains unresolved and must be rerun after the
foreign Cargo window. Editor integration, post-cut profile, RSS/power evidence and independent review
remain pending. No performance result, milestone commit or WeCom completion notification is claimed.
