# Plugins14 reused native crowd state scratch optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`
- Finding: `NNAV-P1-039`
- Status: `validation_pending`

## Scope

- Allocate the Detour crowd FFI state buffer once when `RecastCrowd` is created and reuse it for every state read.
- Validate native `state_count` against the admitted crowd capacity before projecting Rust states.
- Reserve the active-state result exactly instead of allowing filtered collection to grow repeatedly.

## Contract

- The native scratch length remains equal to the admitted crowd capacity for the crowd lifetime.
- Reentrant scratch use and allocation failure return typed `NavigationError` values instead of panicking.
- Native state counts above the crowd capacity fail closed before a slice is formed.
- The existing owned `Vec<RecastCrowdAgentState>` API and state values remain unchanged.

## Performance Gate

- The release workload uses capacity 4,096, one active agent, and 64 reads per sample.
- Per-sample native scratch allocations fall from 64 to zero after crowd construction, a deterministic 100% reduction.
- Per-sample default initialization writes fall from 262,144 states to zero, a deterministic 100% reduction.
- The release gate uses 21 alternating legacy/optimized sample pairs and nearest-rank P95; optimized P95 must remain within 110% of legacy P95.
- Measured timings remain pending the grouped coordinator validation.

## Validation

- The behavior regression locks the scratch pointer and length across two state reads while checking returned agent cardinality.
- Existing crowd update, position sync, filter recycling, and area-mask behavior remain in the same native test group.
- The release marker is `PERF-MVP-PLUGINS14-CROWD-STATE-SCRATCH`.
- Cargo/native compilation and release measurements are queued in the multi-task Plugins aggregate; no standalone Cargo run is claimed here.

## Remaining Plan Work

- This slice removes the capacity-sized FFI allocation and initialization in `NNAV-P1-039` without touching the concurrently owned runtime agent loop.
- Reusing the high-level result buffer through a caller-owned scratch or borrowed state view remains open until the current Navigation runtime session has integrated.
- Configurable product capacity, generation-qualified slots, and overflow policy remain separate Plugins14 milestones.
