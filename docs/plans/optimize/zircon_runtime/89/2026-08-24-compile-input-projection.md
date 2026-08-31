---
title: Runtime89 Render Graph Compile Input Projection Optimization
category: zircon_runtime
report_id: Runtime89-compile-input-projection-2026-08-24
date: 2026-08-24
session_id: root-runtime89-compile-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime89 Render Graph Compile Input Projection Optimization

## Scope

This slice removes two redundant owned projections from RenderGraph compilation. It advances the
Runtime09A P1-6 String/clone hot-path finding under the Runtime89 compiler owner and adds a 10,000
pass qualification point. It does not close Runtime89's 100,000-access scale gate, compiler IR,
barrier, queue, physical allocation, or execution-packet milestones.

## Implementation

The resource identity map now borrows names from the builder as `&str`. Owned names are created
only for the compiled graph DTO or typed error paths, rather than cloning every resource into a
temporary map before compilation.

The manual dependency projection created for the first topological order is now moved into resource
dependency inference. Execution and culling still retain their required independent adjacency
owners, but the compiler no longer rebuilds the same pass dependency vectors before creating them.

Regression coverage preserves manual dependency order, culling reachability, and dependency counts.
A source allocation contract rejects restoration of an owned resource-name map or a second pass
dependency projection.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Temporary owned resource-name clones for 10,000 resources | 10,000 | 0 | 100% reduction |
| Manual dependency edge copies for a 10,000-pass chain | 29,997 | 19,998 | 33.3333% reduction |
| 10,000-pass / 10,000-resource compile | dynamic evidence pending | <= 5 s | coordinator release gate |

The ignored Windows-native release evidence prints `RUNTIME89_COMPILE_PROJECTION_BENCH_V1` with the
exact elapsed nanoseconds, target, pass/resource counts, and both deterministic copy counts. Dynamic
elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and seven compile-projection source/evidence
  contracts: passed.
- RenderGraph regressions plus the ignored release performance evidence: pending a shared
  coordinator-managed Runtime batch.
- No local Cargo lane is launched, and no coordinator compilation is monitored in real time.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

The compiler still owns String-heavy compiled DTOs, rebuilds broader maps and adjacency structures,
and has not met Runtime89 G43's 100,000-access profile. Sparse-resource admission, typed storage
textures, barrier/queue packets, physical allocation identity, and execution diagnostics remain
separate work.
