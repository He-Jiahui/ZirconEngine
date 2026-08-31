---
related_code:
  - zircon_editor/src/core/gateway/**
  - zircon_editor/src/tests/gateway/**
  - zircon_editor/src/core/runtime_event_consumer/**
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_event_consumers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_runtime_host/src/foreign_output/**
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
write_scope: []
status: pending
---

# Editor gateway closure

This is a current-source static closure for the editor/runtime gateway. It
remains pending: the gateway is active in retained Play ticks, world sync,
plugin-event delivery and authoring inspection, but current-source Cargo,
scale, lock and F4 product gates are not green. No Rust source was changed and
no gateway scope is accepted into `review.md`.

## Scope and source state

- `zircon_editor/src/core/gateway/**`: 25 Rust files, 4,351 physical lines,
  3,904 nonempty lines, 145,278 bytes, 28 tests, two ignored tests and eight
  include sites. The tree includes the current session/route split and the
  foreign optimization test module.
- `zircon_editor/src/tests/gateway/**`: 11 Rust files, 2,332 physical lines,
  2,058 nonempty lines, 76,235 bytes, 53 tests and four include sites.
- The current foreign work is preserved. ArcSwap gateway generations, opaque
  gateway origins, checked replacement generations, operation/viewport-pick
  routes, foreign-output ownership/fuse handling, bounded frame demand and
  plugin-page validation are directionally positive.
- Focused rustfmt is not an acceptance gate: the current foreign tree has
  formatting failures in the known modified route/session files. The broader
  workspace remains blocked by the known UI/text compiler failures, SDF
  cfg(test) mismatch, stale compiled-scene/OIT source guards and graphics
  feature reexport issue. No current executable, WPR/Tracy or RenderDoc result
  is claimed.

## Product boundary and positive behavior

The gateway is a shared editor/runtime transport owner. Retained Play calls
`tick_frame` and the runtime-event consumer drains plugin pages through it;
authoring and inspection paths use borrowed world access and world queries.
Capture, operations, viewport picks, profiling and surface binding are
explicit or selected paths.

Current behavior worth preserving:

1. `EditorRuntimeGatewayHandle` publishes an immutable `GatewayGeneration`
   through `ArcSwap`; leases retain the origin that created opaque resources,
   and replacement uses checked generations.
2. Operation and viewport-pick routes pin calls to one gateway identity and
   validate request/ticket/result identity. Do not regress these route-local
   protections while generalizing the lease model.
3. `SessionGateway::new_with_identity` checks the ABI session handle and exact
   V8 table shape before exposure. Required callbacks and output release are
   validated up front.
4. Foreign output owners retain provider storage until explicit release/drop,
   validate encoded/item budgets, and release through one shared state. Frame
   data can be borrowed without a mandatory RGBA clone.
5. Frame demand maps to typed `OnDemand`, bounded `SleepUntil` and
   `Continuous`; plugin pages cap 64 deliveries and 256 KiB encoded bytes.
6. Highlight sets use one sort/dedup projection. In-process borrowed-world
   reentry is rejected, and visible surface transitions retain the previous
   binding on failure.

## Findings

### P0: identity-pinned routes do not cover the full child lifecycle

The handle loads the current lease independently for world watches,
invalidation drains, capture, surface calls, plugin subscriptions and most
public operations. A watch token or async result created by an old gateway can
therefore be revoked or drained through a successor gateway after replacement.
`query_world_at_identity` has a post-call identity check, while the ordinary
world/capture methods do not. `capture_frame_at_identity` only prechecks, so a
replacement can race completion unless the returned frame carries its origin.
Old origins intentionally remain callable, which is useful for opaque cleanup,
but there is no universal child terminal/rebase/cancel ledger.

Every multi-step request needs a `GatewaySessionGeneration` lease that carries
session, gateway, play-instance and capability generations. Watch,
subscription, surface, capture, operation, pick and world-invalidation child
resources must either stay on that lease or return a typed `Rebased`,
`Cancelled`, `Stale` or `Fault` outcome. Replacement and shutdown must retire
each child exactly once rather than silently redirecting a later call.

### P0/P1: request and output work is not one admitted proposal

Session world queries, watches, profile control, operations and plugin
subscriptions serialize public requests with `serde_json::to_vec` before any
host-side request byte, item, nesting or deadline proposal. World and plugin
outputs are bounded by the lower foreign-output owner only after the foreign
call; decoded objects, nested strings, host projections and callback/result
storage are not one end-to-end capacity reservation. Public world filters,
selectors, watch names and event/schema strings are unbounded at this boundary.

Compile a typed `GatewayRequestProposal` before serialization/FFI. It must
admit request bytes/items/depth, expected output bytes/items, decode and
projection peak, callback/result capacity, gateway generation and deadline.
An inadmissible query or subscription returns typed backpressure with zero
foreign call and no destructive output drain.

### P0/P1: in-process callbacks hold the world lock across arbitrary work

`InProcessGateway::with_world` and `with_world_mut` invoke caller callbacks
while `LevelSystem` owns its world lock. The thread-local guard prevents
same-thread reentry, but it does not bound callback duration or prevent
cross-thread contention. Inspection/query projection can allocate and scan
while world mutation waits, and there is no affinity/deadline contract for the
borrowed callback ABI.

Keep a short, explicitly borrowed lock scope for small authoring mutations.
Move large snapshots and projections to generation-qualified immutable data,
or expose a bounded query/snapshot proposal that releases the world lock
before expensive consumer work. Main-thread-affine callbacks need explicit
deadline, cancellation and quarantine accounting.

### P1: capability and lease work repeats after generation construction

Runtime capabilities are sorted/deduplicated into vectors at construction,
which is appropriate for a cold generation, but overlay submission still
linearly scans core capability strings. Public handle methods repeatedly load
and clone an `Arc` lease for chains that could borrow one origin. A cached
capability snapshot also stays stale if provider composition changes without a
full gateway replacement. The compatibility `SessionGateway::new` constructor
creates a zero-valued identity, weakening the stronger identity contract for
callers that do not migrate to `new_with_identity`.

Publish one immutable capability generation with dense IDs/bitsets and a
composition fingerprint. Require explicit identity construction, and let
multi-call routes borrow one lease instead of resolving the current pointer
per method. Capability changes must create a new generation or return a typed
stale/reconfigure result.

### P1: selected capture and output terminal provenance remains split

Gateway-owned frame storage and lower foreign-output release are safer than the
old clone path, but capture completion, surface replacement and plugin/output
drain do not share one end-to-end request identity. A replacement can leave an
old child alive through an opaque origin while the public handle reports only a
generic gateway error. The lower global output fuse also makes a protocol fault
in one output kind reject later unrelated output kinds; this safety policy is
not exposed as a scoped generation receipt.

Use one typed child request ledger for capture, output, subscription and
operation results. Preserve accepted frame/output receipts separately from
sideband faults, and make fuse scope and terminal reason visible in the
generation-qualified diagnostics snapshot.

## Architecture hard cut

M0 adds RED tests for replacement during watch/query/capture/drain, stale
unwatch, request/output cap+1, world-lock callback stalls, provider capability
change, old-constructor identity, child cancellation and terminal receipts.

M1 seals `GatewaySessionGeneration` with session/gateway/play identity,
capability/composition fingerprint, provider/device epoch, callback affinity,
request/output limits and dense capability IDs. Every route and retained child
borrows this generation.

M2 compiles a `GatewayRequestProposal` before JSON/FFI work. It reserves wire,
decoded, projection, callback and result capacity with one deadline and returns
`Ready`, `Backpressured`, `Invalid`, `Stale` or `Fault` before a foreign call.

M3 separates lock-scoped in-process reads/writes from immutable snapshot/query
work. Large projections run after the world lock is released; callbacks declare
main-thread or worker affinity and have an explicit cancellation/quarantine
owner.

M4 unifies watch/subscription/capture/surface/operation/pick lifetimes under a
terminal child ledger. Replacement, session loss and shutdown publish exactly
one outcome per child, while old origins remain available only for the cleanup
they own.

M5 makes capability checks dense and generation-qualified. Stable chains use
one retained lease, overlay lookup is O(1), and capability/composition changes
cannot silently reuse an old snapshot.

M6 gates diagnostics explicitly: Disabled performs no capability scans,
snapshot clones, empty drains or owned labels; Counters/Sampled/Full report
request/output bytes, route waits, callback time, fuse scope and terminal
reasons from the accepted generation.

## Evidence and acceptance

Unreal's `MessageRouter.cpp` dispatches work according to recipient affinity,
queuing a graph task for non-`AnyThread` recipients; `TickTaskManager.cpp`
models prerequisites, parallel queues and explicit waits. These are evidence
for declared callback affinity and task ownership, not a definition of the
Zircon ABI or quota values. Unreal RDG extraction and trace sources likewise
support terminal publication and diagnostics-disabled no-work without proving
this gateway's recoverable error contract.

Acceptance covers gateway/session generations 0/1/16/64/cap+1; replacement,
rebase, shutdown and device/session loss; world filters/selectors and plugin
requests at zero, 1 KiB, 64 KiB, cap and cap+1; output rows/bytes and decode
peaks at the same boundaries; callback latency 0/1/16 ms and timeout; one and
many concurrent routes; old-token revocation; and diagnostics
Disabled/Counters/Sampled/Full.

Hard gates are: every accepted child terminalizes exactly once; replacement
cannot redirect an old token to a successor; request, decoded, projection and
callback bytes are pre-admitted; no arbitrary callback owns the world lock
past its declared scope; stable capabilities and leases do not rebuild or
deep-clone; unsupported/stale operations are typed; diagnostics match actual
work and Disabled adds zero gateway overhead.

No benchmark artifact or production micro-fix is warranted before the shared
generation, request proposal and child-lifecycle owner are unified.
