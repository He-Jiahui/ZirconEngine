---
title: Runtime Shader Dependency Direct Append
category: zircon_runtime
report_id: Runtime867-shader-dependency-direct-append-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime867 Shader Dependency Direct Append

## Finding

Full project dependency publication called `dependency_locators` for every
shader, materialized a complete temporary `Vec<AssetUri>`, and then moved each
locator into the retained `dependencies_by_id` vector. The provider lookup and
first-seen ID set were already indexed, but the full-generation caller still
paid for a second locator-slot buffer and allowed its retained destination to
grow geometrically.

## Optimization

- Share one `append_dependency_locators` owner between the returned-vector API
  and full-generation publication.
- Reserve the authored import count directly on the caller-owned destination
  and clone each admitted provider locator into its final owner.
- Preserve unique-owner lookup, ambiguous-provider rejection, first-provider
  order, provider-ID deduplication, and one runtime-owned occurrence when
  metadata already names the same locator.

The returned-vector API still has the same shape and now delegates through an
empty destination; targeted import callers therefore retain their existing
behavior while the full-generation path no longer materializes the temporary
locator vector.

## TDD and deterministic evidence

The Runtime867 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. The lower regression seeds a metadata-owned dependency, uses repeated
provider imports, and verifies the retained duplicate boundary plus exact
first-provider order.

For 4,096 admitted providers, the retired full-generation path owns 4,096
temporary locator slots and the direct append owns zero. Provider locator clone
count is unchanged because the retained dependency vector remains the owner.
The ignored 101-pair Release marker
`RUNTIME867_SHADER_DEPENDENCY_DIRECT_APPEND_BENCH_V1` emits alternating
p50/p95/p99 samples and gates direct-append p95 at or below the temporary
projection p95.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Runtime866–868 plus the adjacent glTF snapshot contract pass `19/19` in one
  local source/model batch.
- Runtime867 is grouped with Runtime866 and Runtime868 for one managed
  current-source lane; it is not submitted alone.
- Local evidence does not establish Windows compilation, allocator behavior,
  or full project-import product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs` | `3C037FFADB7F7A94F8AC2B578FD66EC5444831C54FDE388974692A5926823B87` |
| `zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/optimization_batch_runtime867_shader_dependency_direct_append_tests.rs` | `EBA53B5EAA8FEEE7EB934B59B81A3BA96EFF3BC9EB583CC25DAB428364AFA9DC` |
| `tools/tests/test_runtime867_shader_dependency_direct_append_performance_contract.py` | `5FF0B49E047EE2C556A3EC6DD68171C2970D62D02DE319F546D4FD324F5E6F68` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the combined Windows lane compiles Runtime, executes the lower regression and
ignored Release marker, and supplies allocator plus project-import product
p50/p95/p99 evidence. The deterministic slot reduction is not product
acceptance.
