# Native Plugin Discovery Authority Architecture And Profiling Plan

- Date: 2026-08-27
- Owners: Runtime06 / Runtime11 / Editor12
- Status: `source_contract_implemented_editor_migration_managed_validation_and_profile_pending`

## Decision

Native plugin discovery is an application-process service, not a child of one
`CoreRuntime` generation. `ProductCompositionRequest::prepare` can discover and
load admitted native plugins before the Core runtime is composed, while loaded
library lifetime is owned separately by `NativePluginHostHandle`. Injecting a
weak pool from the first runtime into the process-static discovery authority
would therefore create a stale execution route after that runtime shuts down.

The process lifetime agrees with Unreal's architecture. `IPluginManager::Get`
is the global access point in
`dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h`.
`dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp`
constructs `FPluginManager` on demand as one function-static instance, states
that it is destroyed at program exit, and performs initial discovery in the
manager constructor. Zircon keeps the same lifetime separation: process-owned
discovery, product-generation host handles, and explicit activation.

This decision accepts the authority lifetime, not its current physical worker
budget. `NativePluginDiscoveryRefreshService` obtains the I/O route through
`TaskPools::process_default()`, which materializes the complete process
Compute/AsyncCompute/Io pool set. A product can later create a second explicit
three-domain `ExecutionRuntime`. Whether this is material in startup time,
resident threads, idle power, or shutdown behavior must be measured before the
execution topology changes.

## Current Structural Defect

The synchronous `discover` projection schedules a cold generation and then
waits for its terminal result. Editor plugin-management paths cannot use that
contract for an interactive refresh without risking filesystem and manifest
work on the request path. The internal refresh service already has the correct
bounded newest-generation publication model, but its ticket and immutable
snapshot were not available through the canonical
`plugin::native::discovery` namespace.

## Source Contract Implemented

The source now separates root preparation from the UI fast path:

1. `resolve_native_plugin_discovery_root(...)` canonicalizes and interns a root
   during project-open or another admitted setup phase. This operation may stat
   the filesystem and is deliberately explicit.
2. `request_native_plugin_discovery_refresh(&root)` admits a bounded refresh
   and returns its generation-bound ticket without path resolution, directory
   traversal, manifest reads, terminal waits, or dynamic-library activation.
3. `latest_native_plugin_discovery_snapshot(&root)` returns the immutable
   last-good `Arc` publication without starting work or touching the
   filesystem.
4. Tickets expose cancellation, completion, terminal observation, and a
   maximum of 32 terminal observers. Blocking `wait_terminal` remains internal
   to the loader's synchronous compatibility projection.
5. Async-only callers no longer leave completed tickets in the authority's
   in-flight map indefinitely; admission reclaims terminal entries before
   coalescing the next request.

Dynamic library loading remains only in the explicit load/host path. The
refresh terminal binds generation and snapshot in one value, so callers do not
pair a mutable report with a separate generation lookup.

## Bounds And Complexity

Let `R` be admitted root/input keys, `C` candidates, `D` diagnostics, and `O`
terminal observers. Current defaults enforce `R <= 16`, `C <= 4096`,
`D <= 128`, 128 MiB aggregate admitted reads per refresh, 64 MiB peak admitted
scratch, a 10 second deadline, and `O <= 32` per ticket. Root identities are
separately capped at 32.

- prepared-root refresh admission: `O(R)` terminal-entry reclamation followed
  by ordered-map lookup; `R` is fixed by the admission budget;
- latest-snapshot read: `O(log R)` lookup and one `Arc` clone;
- active queue state: at most one active and one latest pending generation per
  root/input key;
- published memory: `O(R * (C + D))` bounded immutable payload and manifest
  index state;
- UI fast path: no directory enumeration, manifest parsing, native ABI probe,
  library load, polling loop, or terminal wait.

These are source bounds, not measured latency, CPU, RSS, or power results.

## Editor Migration Architecture Gate

The remaining Editor migration is not a four-call local replacement. Current
source has two different consumer classes:

- enablement and manifest completion synchronously call discovery and only
  need a generation-bound catalog projection;
- native registration synchronously calls the explicit editor load path and
  needs activation results, contribution materialization, and retained native
  library lifetime rather than a discovery-only snapshot.

`ProjectPreflightReceipt` must retain its documented data-only guarantee: it
contains no plugin, runtime, filesystem-mutation, or live observation
capability. A refresh ticket must therefore not be inserted into that receipt.
Project preflight may resolve the native discovery root while filesystem setup
is admitted, but the project/session activation ledger must own the companion
live state: prepared root identity, current refresh ticket, last observed
terminal generation, selected activation generation, and product-generation
`NativePluginHostHandle`.

The dependency-ordered migration is:

1. Project preflight resolves the plugin directory once and passes the prepared
   root as companion activation input without weakening the receipt contract.
2. After project admission, the activation ledger requests one bounded refresh
   and owns ticket cancellation/terminal observation across session shutdown.
3. Status, manifest completion, and native-aware enablement consume only the
   latest complete immutable discovery snapshot associated with that ledger.
   A cold session with no publication remains explicitly pending; it must not
   fabricate an empty catalog or fall back to synchronous discovery.
4. Explicit native registration consumes the selected completed discovery
   generation through the load/activation transaction, publishes its load
   report separately, and retains the returned host handle for the product
   generation. A discovery snapshot is not evidence that a DLL is loaded.

Until that activation-ledger contract exists, changing any individual Editor
call site would either produce cold empty snapshots, split report/generation
identity, or bypass the activation transaction. The Editor migration therefore
remains architecture-blocked while Runtime11's nonblocking source contract is
available and independently usable.

## Profiling Gate Before Worker-Topology Optimization

Use one source-bound Windows product build and compare these cases:

- first native discovery before Core bootstrap;
- Core bootstrap without native discovery;
- native discovery followed by Core bootstrap;
- warm snapshot read and a 1/16/256 event refresh burst;
- one and two sequential product/runtime generations in the same process.

Record process thread count by Zircon thread name, worker creation count per
domain, startup wall/CPU time, refresh submit p50/p95/p99, cold and warm
publication latency, queue depth, peak working set, idle CPU, package power,
and shutdown receipts. WPR/ETW is the preferred Windows trace source. Compare
the same hardware and build profile; do not infer an Unreal-equivalent power or
latency result from source topology alone.

Only if the trace attributes a material regression to duplicate process pools
should the next architecture slice introduce a process application execution
owner or a dedicated bounded discovery I/O owner. It must not bind discovery
to one runtime generation or create an Editor-local pool.

## Remaining Work

- Editor12 adds a project/session activation-ledger companion to the data-only
  `ProjectPreflightReceipt`, stores the prepared root at project open, owns one
  refresh ticket, and observes its terminal without polling or UI-thread I/O.
- Status, manifest completion, and enablement render only the latest complete
  discovery snapshot; native registration instead consumes a selected
  generation through explicit activation and retains the product-generation
  native host handle.
- Managed Runtime/Editor Cargo validation covers burst coalescing, stale-result
  suppression, cancellation, shutdown, and the UI no-I/O fixture.
- Run the profiling matrix before changing the process task-pool topology.

No milestone acceptance, performance improvement, power improvement, or
shutdown-drain claim is made by this source slice.
