# Plugins15 Immutable Compiled-Tree Generation Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, NAI-P1-015
- Status: implementation and focused static validation complete; managed release batch queued

## Problem

Every agent tick cloned every registered `CompiledBehaviorTree` into a new
vector before resolving implementation owners and evaluating the selected
tree. Tree registration is rare, but this deep catalog clone ran once per
agent tick and copied node arrays, child-index arrays, parameters, and strings.

## Change

- `AiRuntimeState` owns an immutable `Arc<[CompiledBehaviorTree]>` generation.
- Registration and owner revocation rebuild the generation after changing the
  authoritative registry.
- A steady tick clones one `Arc` and addresses its selected tree by the stable
  registry index; it does not clone the selected tree or the full catalog.
- Tree order, subtree lookup, owner fencing, revocation, and evaluation
  semantics remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| One tick with 256 registered trees, 32 nodes each | 256 deep tree clones | 1 `Arc` clone | 99.61% fewer top-level clone operations |
| Compiled nodes copied per tick | 8,192 | 0 | 100% |
| Catalog generation rebuild | every agent tick | registration/revocation only | removed from steady state |

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `b684ea3ed9304cf4a9f71e5787befa1a`, preview
  `258e1f8cc6ee4d7fa685b24a615b8096`, fingerprint
  `b9047be13003a7c46040170fe788f53d2c5dcc8722f2488ffecc97d56ebffbbc`.
- Current `manager/state.rs` SHA-256:
  `CC1C15BD9389D4E8D765AAFF6ECFB3DFC60AA2AFE4E490B7888208943DD31410`.
- Unified deterministic model manifest SHA-256:
  `93CF6BD9C2D374D1F4C81CF6776948372611820AAB048DB2EB499977E8493347`.
  Across 32 acquisitions it records top-level tree clones `8,192 -> 0`,
  compiled-node copies `262,144 -> 0`, and 32 candidate `Arc` handle clones.
- Focused source/model/validator contract passed locally `12/12`; managed
  static ticket `049d11366ae94ef38ddc58158d6e6b69` is queued.
- Four-benchmark Windows release batch ticket
  `bf5d08d9143849e189ac6e0fa1bb477c` uses validator SHA-256
  `A4696773FBB2145F4372101A18CD6F30D14DD741182A6020F77F19A7D28B65FB`.
  Its 21 alternating sample pairs are the only accepted P50/P95 source.

## Acceptance

- `compiled_tree_generation_rebuilds_in_registry_order` verifies cold-path
  generation order.
- `steady_tick_uses_compiled_tree_generation_without_deep_catalog_clone`
  rejects a restored per-tick compiled-tree collection.
- `immutable_compiled_tree_generation_release_benchmark_evidence` compares 21
  paired, alternating release samples over 256 trees of 32 nodes and 32
  acquisitions, then computes nearest-rank P50/P95.
- Timing gate: generation acquisition P95 must be no more than 10% of the
  legacy deep-clone P95.
- Exact-file Rustfmt, source assertions, and scoped `git diff --check`: passed.
- Cargo regression and release P50/P95: queued in one batched Windows
  coordinator validation with targeted debug projection.

## Remaining Scope

The cold path still deep-clones compiled trees when publishing a generation,
and implementation-owner resolution still scans every compiled node on each
tick. A future artifact generation can own `Arc<CompiledBehaviorTree>` entries
and precompute owner leases without restoring hot-path directory cloning.
