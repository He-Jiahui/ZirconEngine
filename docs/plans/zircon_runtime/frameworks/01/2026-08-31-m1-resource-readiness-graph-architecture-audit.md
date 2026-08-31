# Frameworks01 M1 Resource Readiness Graph Architecture Audit

Status: `architecture_review_iteration_2_complete / behavior_matrix_source_complete / paired_profile_matrix_source_complete / managed_execution_fifo_blocked / production_change_blocked`

Owner Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`

Plan owner: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`

Date: 2026-08-31

## Purpose

This record reopens the readiness graph as an engine-foundation algorithm, not as a local
micro-optimization. The current implementation publishes immutable generations and maintains a
reverse dependency index, but its cycle contract, graph input contract, evaluation algorithm, and
consumer admission contract are not yet strong enough for M1 acceptance.

No production algorithm change is authorized by this record. Behavior RED cases and a reproducible
current-source profile must exist first. Historical timings, current intuition, and reference-engine
shape are design inputs, not acceptance evidence.

## Current physical source

The review is tied to these exact SHA-256 fingerprints:

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `zircon_runtime/crates/zr_resource/src/readiness_generation.rs` | 135 | `4830e32d4b730ee5ad4f435c1239a3bdc6d397250b3dfa6c0011880cda0045f8` |
| `zircon_runtime/crates/zr_resource/src/manager/readiness_projection.rs` | 357 | `9aac00f12030fff53fef7a23e388579c2b8a0099e9787cf300a4bfe4d8c525c9` |
| `zircon_runtime_interface/src/resource/resource_record.rs` | 105 | `f6eb941bf0931a9f3c99c55f8cfb652cad23f7c4a7ca42fe85f011ce838c366e` |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager.rs` | 747 | `8a155161c91a125654781be06111130750f94d5e9d93cd2308d05bd509e10b54` |

## Current architecture

1. `ResourceReadinessProjection` owns mutable `sources` and
   `reverse_dependencies: dependency -> referencers` authorities.
2. A changed source updates reverse edges, then an iterative `VecDeque` reverse closure finds all
   referencers that may need a new result.
3. `compute_aggregate` recursively walks outgoing dependencies for every affected root and memoizes
   direct state, recursive state, and a 64-bit `DefaultHasher` fingerprint.
4. Publication clones each touched one of 64 `HashMap` shards, preserves equal row `Arc`s, and
   advances generation only when a row changes.
5. Runtime asset load-state queries read the published direct and recursive states. Render residency
   ticket creation reads the generation sequence and dependency revision.

The reverse-closure and immutable-publication split is worth preserving. The recursive aggregate and
implicit graph contract are the parts under review.

## Proven structural findings

### A1. Graph input is not canonical

`ResourceRecord::with_dependency_ids` stores the caller's `Vec<ResourceId>` unchanged. Public fields
and derived `Deserialize` also accept duplicate, self, and order-variant edges without validation.
The reverse index de-duplicates referencers through `HashSet`, while aggregate evaluation still walks
every duplicate edge and hashes input order. Equivalent dependency sets can therefore produce
different edge counts, fingerprints, dependency revisions, and publication work.

Required decision: the authority must define dependencies as either an ordered multigraph or a
canonical set. Asset dependency readiness is set-like today, so the candidate contract is a sorted,
unique list that rejects self edges and reports non-trivial cycles separately. This is a hard cut; no
compatibility normalization branch may survive beside the canonical constructor/deserializer.

### A2. Deep graphs consume the native call stack

Reverse closure is iterative, but `compute_aggregate` performs one Rust call per dependency depth. A
deep but valid import chain can overflow the process stack before it produces readiness. No deep-chain
test currently bounds this behavior. Production evaluation must use an explicit work stack or an
incremental queue; increasing thread stack size is not an architectural fix.

### A3. A cycle back-edge is silently synthesized as `Loaded`

When `visiting.insert(id)` fails, current code returns direct `Loaded`, recursive `Loaded`, and a
synthetic cycle hash. No row or diagnostic identifies the cycle. A fully resident self-cycle or
multi-node cycle can consequently publish `Loaded` recursive readiness. This is an undocumented SCC
rule, not a validated engine contract.

M1 default is fail closed: a cyclic dependency closure must not advertise recursively loaded. The
eventual implementation must return typed cycle evidence with stable member IDs. If a future asset
kind explicitly allows SCC loading, that policy must be declared by the owning asset contract and
evaluated as an SCC, never inferred from a DFS back-edge.

### A4. Fingerprint equality is used as semantic equality

Nested dependency changes advance `dependency_revision` only when a 64-bit hash changes. A collision
can suppress consumer invalidation. The hash also couples the result to dependency iteration order.
The replacement design must make semantic change detection structural: an exact canonical dependency
stamp containing outgoing IDs, dependency source revisions, dependency readiness, and child semantic
revisions, or another collision-independent equality check. A hash may accelerate a negative compare
or remain as diagnostics, but a hash match must be confirmed by exact structural equality before a
consumer-visible dependency revision is preserved. Replacing `DefaultHasher` with a larger digest
alone is not the hard cut.

### A5. Publication cost is coupled to fixed hash shards

Every touched shard clones its complete `HashMap`. With 100,000 rows, a sparse update copies roughly
one sixty-fourth of the table before allocator/hash variance; dense closures can copy most shards.
This may or may not be the current bottleneck. Shard/page layout changes remain blocked until the
profile reports sparse and dense allocation/time distributions on the current physical source.

### A6. Consumer admission is weaker than the readiness contract

`resolve_ticket_seed` verifies catalog presence, kind, readiness-row presence, and revision, but does
not itself require root/direct/recursive `Loaded`. This may be valid only if every caller has a proven
earlier readiness gate. The render owner must either prove that invariant or add typed admission
rejection. Frameworks01 will not edit the render path from this plan, but M1 cannot call a cyclic or
failed readiness closure safe merely because it has a revision.

### A7. Generation counters silently wrap

Both generation sequence and per-row dependency revision use `wrapping_add`. These values are public
cache identities, so wrapping violates the monotonic-generation contract and can make an ancient
prepared shader/material entry compare equal after rollover. The hard cut must use a checked
monotonic policy with a typed terminal/reseed outcome; saturating or wrap-to-zero behavior is not an
accepted compatibility path. The rollover RED may construct a near-terminal generation through the
crate-private assembly boundary; it does not need to execute `u64::MAX` real updates.

## Local reference-engine evidence

### Unreal Engine, primary reference

Reviewed sources:

| File | SHA-256 | Relevant structure |
| --- | --- | --- |
| `dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/DependsNode.h` | `56ed2db89d0f692d3d2703a1e8d6acbac9d92dea399388263d13d6875bb3c4ce` | Explicit dependency and referencer lists; sorted-state flags; duplicate-edge visibility; reservation/scratch structures. |
| `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp` | `1cbf8131ed811392911d83c7f8c302d6ccd8088a792d84606772fbbfc78f9ba9` | Explicit load event nodes, dependency arcs, barrier counts, work queues, graph allocation counters, and circular-dependency handling. |

The relevant standard is not to copy Unreal types. `FDependsNode` keeps dependency and referencer
lists first-class, exposes duplicate-edge information, and records whether those lists are sorted.
`FEventLoadNode2` separates dependency arcs from an atomic barrier count and releases ready nodes into
`FAsyncLoadEventQueue2`; the loader waits for dependencies to reach explicit phases before releasing
the next package node. Zircon should likewise keep topology authority separate from queued readiness
transitions and expose cycle handling rather than turning a recursion sentinel into success. Unreal
also reserves and accounts graph storage for batch work, which is relevant only after Zircon has a
measured current-source workload.

### Bevy, secondary behavioral reference

`dev/bevy/crates/bevy_asset/src/server/info.rs`, SHA-256
`6c6d4d79051bd8abf92d8f9a25548f7a89ebea7df384b9f1d41091b3ddb42b65`, stores per-asset direct and
recursive load state plus sets of loading/failed dependencies and waiting dependents. Completion and
failure propagate to dependents as state transitions instead of recomputing an arbitrary recursive
closure for every public query. Bevy's recursive helper calls are not adopted as a deep-chain
solution; its useful evidence is the persistent per-node dependency state and explicit failure
propagation.

## Candidate target architecture

The candidate to test is a single readiness graph authority with four layers:

1. **Canonical topology**: sorted unique outgoing edges and explicit reverse referencers, both updated
   atomically with the source record.
2. **Graph validity**: iterative SCC/cycle detection for the changed weak component. Invalid SCCs
   publish typed fail-closed readiness and stable diagnostics.
3. **Exact semantic stamp**: each node retains a collision-independent canonical dependency stamp;
   hashes are accelerators and diagnostics, never the authority that preserves a cache revision.
4. **Incremental state**: per-node counts or sets for non-loaded direct dependencies and failed
   direct dependencies; a queue propagates only actual state/revision transitions to referencers.
5. **Immutable publication**: readers retain lock-free generation snapshots and stable row identity;
   sequence and dependency revisions advance through a checked monotonic policy, and storage
   granularity is selected only after sparse/dense profile evidence.

The target complexity is `O(V_affected + E_affected)` evaluation with `O(1)` native call-stack depth.
A sparse leaf transition should visit the reverse closure once, not recursively reevaluate the same
diamond subgraphs. Cycle analysis may be linear in the changed component, but must be iterative and
observable.

## Mandatory RED matrix

Before production edits, focused tests must demonstrate current behavior for:

- self-cycle, two-node cycle, and multi-node cycle fail-closed semantics with stable member evidence;
- a 10,000-node chain without stack overflow, followed by a larger profile-only chain;
- diamond and high-fanout graphs with one evaluation per affected node/edge bound;
- missing dependency, removal, replacement, and later arrival;
- duplicate and reordered dependency input under the selected canonical-set contract;
- direct failure/reload and nested failure/reload propagation;
- unchanged update preserving generation and row `Arc` identity;
- dependency revision advancing exactly once for one semantic nested transition;
- exact-stamp collision defense and checked sequence/dependency-revision rollover rejection;
- renderer-facing proof that non-ready/cyclic closures cannot produce an admissible residency ticket,
  owned by the render plan if its boundary must change.

## Mandatory current-source profile

The profile is an ignored release-only test, single test thread, with at least 31 measured samples and
explicit E-drive output. It must emit raw and summary CSV plus exact source fingerprints for:

- chain lengths 1,000, 10,000, and 100,000;
- fanout 64, 4,096, and 100,000;
- diamond/shared-dependency graphs at 4,096 and 100,000 nodes;
- sparse leaf state transition, root edge replacement, missing dependency arrival, and dense state
  transition;
- self/two-node/large SCC detection;
- initial build and unchanged-update controls.

Each mutable scenario requires two paired measurement scopes over the same topology and sample count:

1. manager end-to-end, including registry-to-update `ResourceRecord` snapshot construction;
2. evaluator-only, with owned updates prepared before timing/allocation counters begin.

The current lazy `prepared.updates.iter().cloned()` occurs inside `apply_updates`, so the existing
worker is an end-to-end-shaped sample and must not be labeled evaluator-only. The paired scopes must
report wall p50/p95/MAD, allocation count, requested bytes, peak live bytes, affected nodes, visited
edges, queue pushes, and touched publication pages/shards. Build time is not a runtime sample.
RSS/CPU sampling and Windows power/energy evidence require a separate WPR or equivalent run with its
ETL and reports explicitly placed on E or D; unavailable counters must be marked `unavailable`, never
inferred from elapsed time.

## Decision and execution order

1. Finish and record the current ResourceManagement profile already in flight; it must not be mixed
   into readiness timing.
2. Add the readiness behavior RED cases and reproducible profile harness without changing production
   behavior.
3. Run the current algorithm and publish raw evidence.
4. Choose between incremental queue plus iterative SCC validation and a smaller iterative evaluator
   only from the measured workload and correctness matrix.
5. Implement the hard cut, rerun the identical profile, and compare distributions and asymptotic
   counts.
6. Request independent architecture/code review and only then update M1 acceptance status.

## 2026-08-31 behavior/profile infrastructure result

Step 2 produced the first source harness without a production algorithm change:

- the two existing inline tests moved mechanically from the production root into the folder-backed
  `readiness_projection/tests.rs` owner; the production file fell from 432 to 357 lines;
- `behavior_red.rs` contains explicit ignored probes for self/multi-node cycle fail-closed behavior,
  duplicate/order canonicalization, and a 10,000-node stack-boundary chain;
- the initial source snapshot of `profile.rs` contained a 580-line release-only orchestrator and
  isolated worker. The orchestrator
  launches one child test process per scenario, so a current 100,000-node recursive stack failure is
  recorded without losing fanout, diamond, dense, cycle, and no-change samples;
- every successful worker writes at least 31 raw samples plus p50/p95/MAD, allocation count,
  requested bytes, peak live bytes, changed/affected row counts, edge visits, and exact BLAKE3 source
  fingerprints to an explicit non-C directory;
- queue pushes, touched shards, RSS, and power remain explicitly unavailable until production
  diagnostics and an external E/D-drive WPR run exist;
- the existing management profile and readiness profile share the single 116-line test-only counting
  allocator in `src/test_profile.rs`; no second global allocator exists.

Iteration-2 review found that the worker's measured iterator clones each prepared update lazily inside
`apply_updates`. This is useful as a proxy for the production manager path, which clones records out
of the registry, but it cannot isolate graph evaluation/publication cost. The harness therefore needs
paired end-to-end and evaluator-only modes before Step 2 is accepted. No existing result is discarded;
the measurement label and missing control are corrected before any sample exists.

The paired revision is now source-complete. `ProfileMeasurementScope` runs
`manager_end_to_end` and `evaluator_only` in separate child processes for every topology. The former
materializes each update inside the measured boundary through prepared registry/runtime/payload lookup
tables matching `ResourceAuthority::readiness_source_update`; the latter prepares an owned update
vector before allocation counters and the timer start. Orchestration, raw CSV, summary CSV, and
metadata all carry `measurement_scope`; schema versions advanced to v2. The exact current owner hash
is recorded in the mandatory matrix correction below.
The focused static contract passed 1/1, Rustfmt parsing and scoped whitespace checks are green. This
closes the Step-2 harness-shape correction only; neither scope has emitted a runtime sample.
The complete Resource crate-boundary plus conditional-write static batch then passed 22/22 in
37.657 seconds; the paired-scope requirement extends an existing test method, so the case count
remains 22.

The Resource crate-boundary plus conditional-write static suite passed 22/22 in 45.343 seconds.
Rustfmt and scoped diff checks are green. Managed Rust execution is not yet evidence: job
`64107e1527764083b78888c199cccf5f` stopped in the foreign Runtime Interface host-request bridge on
E0026/E0027 before compiling `zr_resource`. The canonical handoff is
`failure-2026-08-31-runtime-interface-ui-activate-link-field-mismatch.md`.

Step 3 therefore remains pending. Production readiness code remains intentionally unchanged and this
work item is not a milestone candidate until the current worker matrix executes and the resulting
data selects the hard-cut algorithm.

The subsequent managed ResourceManagement retry also produced no readiness or management sample.
Job `28eb6b1ee6a649e79a8cac8c19dc5c21` cleared the prior typed-link bridge diagnostics, then stopped
before `zr_resource` on the active Runtime Interface schema-catalog E0502 borrow conflict. The
foreign owner has since changed that source to an immutable-validation-then-sort shape, but no
managed integration receipt exists yet. Readiness Step 3 and every production algorithm edit remain
blocked; this architecture audit is not promoted by a compile that never reached the crate.

R4 still did not reach Step 3. Managed job `84f3507f1dee480184e94f5cbaf9fdb2` stopped while sccache
compiled the Runtime Interface dependency because the job-scoped temporary directory vanished before
rustc could publish `deps.d` (OS error 3). The job released cleanly and left no Cargo/rustc process,
but it produced no ResourceManagement or readiness worker sample. The unique handoff is
`failure-2026-08-31-managed-cargo-sccache-temporary-path-lifecycle.md`, assigned to the App08 managed
storage owner. Frameworks01 will continue non-validation architecture work, but production SCC/queue,
semantic-stamp, checked-generation, and publication-layout changes remain prohibited until current
algorithm evidence actually executes.

R5 rejected the first tooling return. Fresh managed job
`680c28eeb45f44ada781073ea28a3e50` reused R4, but persistent sccache PID 1660 still allocated below
the deleted R4 `scratch\84f3507f...\temporary` authority and failed before `zr_resource` with Cargo
101. Client-side mode therefore did not repair the already-running server's startup environment; no
readiness worker executed. The canonical Failure remains open, Step 3 remains pending, and non-Cargo
architecture review may continue without converting unmeasured generation/identity risks into
production edits.

## 2026-08-31 mandatory matrix correction and R6 FIFO attempt

The mandatory scenario list is now enforced by source and the Frameworks01 static guard rather than
only by this plan prose:

- `profile.rs` adds fanout 64, a two-node cycle, root-edge replacement on a 4,096-node chain, and
  missing-dependency arrival on a 4,096-node chain. The missing-arrival baseline intentionally omits
  the terminal source while retaining its incoming edge; the replacement case moves the root edge
  from node 1 to node 2 so old-edge removal and new-edge propagation share one measured operation;
- the canonical dependency-set RED was corrected to compare reordered/duplicated forms of the same
  `{first, second}` set. Its earlier second mutation accidentally changed the set to `{first}` and
  therefore could not prove duplicate canonicalization;
- one non-ignored behavior regression now covers missing dependency, later arrival, edge replacement,
  edge removal, and a former dependency arriving after detachment. The final assertion requires the
  parent row `Arc` identity to remain unchanged, proving the obsolete reverse edge no longer admits the
  former parent into the affected closure;
- `test_frameworks_01_resource_crate_boundary.py` now names all four added scenarios, so a later
  harness simplification cannot silently drop plan-required coverage;
- every worker summary and metadata record now fingerprints the profile harness and shared counting
  allocator in addition to the production projection/generation. This prevents allocator or harness
  drift from being compared as if it were an algorithm-only result;
- `manager_end_to_end` no longer times only `ResourceReadinessSourceUpdate` cloning. It now performs
  record clone plus runtime-state and payload-type lookups from prepared maps and allocates the update
  vector inside the measured boundary; `evaluator_only` still completes that work before timing.

Exact current validation-source fingerprints are:

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `manager/readiness_projection/tests/behavior_red.rs` | 162 | `6f5db8c2fd644c71b826876257f21983a9ee271a8b39daa5a8fc062019d9fec6` |
| `manager/readiness_projection/tests/profile.rs` | 726 | `4efc195d4c2fff45bdff3aedf7d573626c2b90c372356fcceb67ff9faef671c4` |
| `tools/tests/test_frameworks_01_resource_crate_boundary.py` | 430 | `6cb6c2b3b39c4d7daea462562fb302c6369bc1cad93802eede6598b6535dc138` |

Rust 2024 formatting, scoped whitespace checks, and the complete Resource crate boundary batch pass;
the latter remains 14/14 because the scenario assertions extend the existing profile-contract test.
Production `readiness_projection.rs` and `readiness_generation.rs` remain byte-identical at SHA-256
`6d45f0e2bbf093080e436dd3394ed8103be439f8bd881ba420f89acf492f2fd8` and
`8441fc693344202651576d4828715f595ca99d370594f6cf26544fc01b3e762d` respectively.

The exact managed Windows release origin was attempted with an explicit
`E:\cargo-targets\frameworks01-readiness-current-r1` target and profile report directory, release
profile, `-SkipBuild -LibTests`, filter `resource_readiness_profile_orchestrator`, and ignored-test
mode. Coordinator request `a6cd317007b5425d820aa2b7dba4fc77` reached terminal `failed` without
creating a Frameworks job: code `cargo_cpu_lane_reserved`, then-current FIFO owner
`root-runtime-editor-optimize-20260829-r5`, reservation `6e1dd0d43a22406d8b15a55f3d86219c`.
Subsequent read-only reconciliation showed other legitimate running/pending reservations, so the
origin has not been replayed while the lane is occupied. No profile directory or runtime sample was
materialized. Step 3 and every production algorithm change remain blocked until the same origin runs
after FIFO release.

The non-C validation-copy owner remains independently green: `python -m unittest
tools.session_coordinator.tests.test_validation_copies` completed 19/19 in 117.660 seconds on the
current source. This verifies the existing baseline path/include closure behavior only; it does not
resolve the separate requirement for a pinned source-root Cargo metadata runner, which remains owned
by `root-coordinator-validation-baseline-r3-20260831` in `tools/session_coordinator/validation_copies.py`
(current SHA-256 `5b59767ef62ee840d415b59a84a2575af3ef74ad5948a76ebb9f8777806a6404`).

At the latest reconciliation, RuntimeEditor reservation `fc9d4d3be30e4f20a7b584ba51ca59c5` had
completed and successor reservation `7db204816dd3486982717a5578dd195a` was running job
`37ab738bf5c947d88421e5ecb2135ff8`, with another successor pending. Cargo/rustc remained active;
Frameworks01 did not cancel, consume, or bypass those reservations and did not replay the failed
origin request. Status therefore remains `managed_execution_fifo_blocked / production_change_blocked`.

## 2026-08-31 current-source managed baseline and algorithm decision

The exact Windows release origin subsequently reached the crate after the FIFO drained. Managed
job `b36edb71b2b04e65bb66d4f6441bac24` used target
`E:\cargo-targets\frameworks01-readiness-current-r1`, sccache endpoint `127.0.0.1:42261`, and the
paired `manager_end_to_end` / `evaluator_only` worker matrix. Cargo finished successfully with exit
code 0; the complete command took 337.7 seconds, including 3 minutes 46 seconds of release
compilation. The orchestration emitted 31 samples after 3 warmups per successful worker. RSS and
power remain unavailable, so this evidence must not be used to claim power parity.

The current algorithm is structurally invalid for ordinary deep graphs. Both 10,000-node leaf
reload workers and both 100,000-node initial-build workers terminated with Windows status
`0xC00000FD` (`-1073741571`, stack overflow) before producing samples. The 1,000-node chain still
completed, with evaluator-only p50/p95 3.067/6.033 ms for initial build and 1.859/2.514 ms for leaf
reload. A larger native thread stack is not an accepted repair: the graph depth is input data and
must not consume the process call stack.

Successful large topologies show that the memoized traversal is nominally `O(V + E)`, but also
quantify its publication and lookup constants:

| Scenario / scope | p50 | p95 | p50 allocations | requested bytes | affected / edge visits |
| --- | ---: | ---: | ---: | ---: | ---: |
| fanout 100,000 / evaluator | 99.954 ms | 113.901 ms | 18 | 0.100 MiB | 2 / 99,999 |
| diamond 100,000 / evaluator | 429.876 ms | 480.143 ms | 100,252 | 28.039 MiB | 100,000 / 199,996 |
| diamond 100,000 / manager | 429.002 ms | 500.499 ms | 100,254 | 28.039 MiB | 100,000 / 199,996 |
| no change 100,000 / evaluator | 0.005 ms | 0.006 ms | 0 | 0 | 0 / 0 |
| no change 100,000 / manager | 0.015 ms | 0.018 ms | 2 | approximately 0 | 0 / 0 |
| cycle 4,096 / evaluator | 13.229 ms | 16.474 ms | 12,864 | 5.171 MiB | 4,096 / 4,096 |

The manager/evaluator pairing shows update materialization is not the dominant cost for the large
diamond; graph evaluation and immutable-row publication are. Fanout invalidates only two rows but
must still inspect 99,999 outgoing edges, while diamond invalidates and republishes the complete
reverse closure. The current 64-shard publication clones only touched shard maps, but a graph-wide
change still allocates one candidate row per affected source. This is a measured follow-up target,
not justification to replace immutable snapshots before the evaluator is correct.

The cycle baseline also confirms the semantic defect described above: the current DFS returns a
synthetic `Loaded` aggregate on a visiting edge, so cycle state/fingerprint depends on traversal
order and advertises a closed dependency loop as recursively ready. The hard cut selected from this
baseline is therefore:

1. canonicalize every source dependency list as a sorted unique set before reverse-edge mutation,
   so order and duplicates cannot create false source changes or fingerprints;
2. replace recursive DFS with an explicit-stack graph evaluation that uses bounded heap storage and
   computes deterministic strongly connected components for the affected dependency subgraph;
3. fail every cyclic component closed and propagate that failure to its reverse dependants, while
   preserving cached generation rows for dependencies outside the exact affected closure;
4. preserve immutable generation/row identity and the no-change zero-allocation path; defer a
   publication-layout replacement until the identical post-change profile proves it remains the
   leading cost.

This closes the profile gate and opens production implementation. It does not complete M1: the
ignored REDs must turn green, ordinary contract tests must pass, the identical release matrix must
be rerun, and an independent reviewer must accept the exact attributed snapshot before coordinator
submission.
