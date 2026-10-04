---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-09-10-config-manager-caller-hardening.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
---

# Runtime Config Manager Caller Hardening

Persistent runtime settings now use the generation-aware Foundation ConfigManager at their
production caller boundary. Built-in and external animation managers, plus the Physics runtime
manager, declare the Foundation dependency and route typed JSON writes through `set_value` before
updating their in-memory projection. The remaining raw store calls are explicitly startup/session
or test-seeding paths and stay owned by the pending schema/layer migration.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime03/P1-5 | Route persistent animation and physics settings through ConfigManager handles; add dependency and stale-generation coverage | implemented_pending_validation | Foundation animation persistence roundtrip, module-resolution tests, source guards, focused access-boundary batch `31/31`, bounded Runtime/Editor contract batch `70/70`, scoped Rustfmt, and merged Runtime/Editor contract batch `1723/1723`; managed Cargo/WGPU and release p50/p95/p99 remain pending |
