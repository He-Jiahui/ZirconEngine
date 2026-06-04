---
related_code:
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/helpers.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
implementation_files:
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/helpers.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_state/*.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs zircon_runtime/src/scene/tests/mod.rs
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --message-format short
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: module-detail
---

# QueryState Owner Split

`zircon_runtime::scene::ecs::query::QueryState` owns ECS query access, matched entity cache state, and the `SystemParam` bridge used by runtime systems. The public type remains exported through `zircon_runtime::scene::ecs::QueryState`; only the source ownership changed.

The split follows the local query directory and Bevy's `bevy_ecs::query` precedent: keep query access, data, filters, iterators, and state in separate owner files instead of letting one state file own every read, mutable, and cached path.

## Owner Files

- `query_state/mod.rs` owns the `QueryState` struct, construction, access descriptors, cache rebuilds, cache counters, and cache metadata accessors.
- `query_state/cached_direct.rs` owns `CachedQueryData` and `CachedQueryFilter` paths that fetch directly from cached component storage locations.
- `query_state/read_only.rs` owns non-mutating `QueryData` iteration, `get`, `many`, `contains`, and combination APIs.
- `query_state/mutable.rs` owns `QueryMutData` access, mutable alias validation, mutable many/combination iteration, and the narrow unsafe fetch used after duplicate checks.
- `query_state/helpers.rs` owns shared fixed-size collection and cached entity-list filtering helpers.
- `query_state/system_param.rs` owns the `SystemParam` implementation that turns query state into `Query<'world, D, F>`.

## Boundary Rules

Do not recreate `query_state.rs`.

New query-state APIs should land by behavior family:

- cache construction and cache metrics in `mod.rs`;
- direct cached storage access in `cached_direct.rs`;
- read-only entity access in `read_only.rs`;
- mutable entity access and alias validation in `mutable.rs`;
- reusable fixed-size collection helpers in `helpers.rs`;
- scheduler/system parameter wiring in `system_param.rs`.

The structure guard in `scene::tests::ecs_query_structure` rejects a legacy `query_state.rs`, missing owner files, behavior impl families in the root file, and owner files above the current budget. The structural audit mirrors that contract as `ecs_query_state_boundary`, so CI or review automation can detect the same rollback without first running the Rust test binary. This keeps future ECS query/cache work from turning `QueryState` back into a mixed hot-path file.

## Validation Notes

This is a structural refactor. It should preserve query behavior and public export shape, then reduce the `runtime-other` large-file pressure reported by the runtime architecture audit. Focused validation should cover the structure guard and representative ECS query tests before broad runtime validation.
