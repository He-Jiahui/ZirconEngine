# Runtime367 List Row Command Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-367-313-20260830-v2`

## Scope

List-row rendering reserves the three commands used by the optional background, label, and
trailing icon, avoiding vector growth during row rebuilds without changing command order.

## Evidence

The focused test checks output shape and the production source contract. The ignored Windows
release benchmark compares zero-capacity growth with exact capacity three over 2,000,000 rows per
sample across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

## Batched validation handoff (2026-08-30)

This slice is included with Editor313 in ticket `0505ae168bca4fef9fadbf1c5f74d753`, source
manifest hash `ca10504651aa6d41c1fb453af6878dde52ff316a20252a7c3d20cf80ad3f47f8`. The manifest
binds both production and nested test files. Cargo, performance, review, commit, and WeCom remain
coordinator-owned and pending.
