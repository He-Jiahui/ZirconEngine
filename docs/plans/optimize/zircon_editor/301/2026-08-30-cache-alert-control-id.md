# Editor301 Cache Chip Control Id

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-356-301-302-20260830-v2`

## Scope

Workbench chip classification repeatedly created the same `control_id.as_str()` view across its
early-return and keyword branches. The view is now cached once and reused, with chip identity and
status exclusion behavior unchanged.

## Static Evidence

- `node.control_id.as_str()` evaluations in chip classification: `5 -> 1`.
- Status exclusion, fixed control ids, chip variant, and chip/pill role behavior are unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR301_CACHED_CONTROL_ID_BENCH_V1`. It compares
repeated control-id view acquisition with one cached view over 1,000,000 classifications per sample
and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.

## Current batched validation handoff (2026-08-30)

Editor301 is included in ticket `7484a74c23ec418d92154572353fc131` for request
`runtime-editor-356-301-302-20260830-v2`, with source manifest hash
`3fc81ca19f55ebaa5de890c57acbce48ad91446e38db35111a7386ad7e37986a`. The batch runs the three
Runtime356/Editor301/Editor302 behavior and ignored performance groups in one Release invocation.
Cargo, exact p95 evidence, review, commit, push, and WeCom remain coordinator-owned and pending.

## Validation attempt (2026-08-30)

Corrected batch ticket `7484a74c23ec418d92154572353fc131` ended `failed`. The coordinator
provided no valid Cargo, performance, or commit evidence; the external validation resource was
left unchanged and no successful WeCom notification was sent.
