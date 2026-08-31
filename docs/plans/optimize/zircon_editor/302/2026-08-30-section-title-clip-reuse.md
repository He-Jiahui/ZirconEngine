# Editor302 Section Title Control Identity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-356-301-302-20260830-v2`

## Scope

Section-title identity now checks the three fixed workbench control ids before scanning the
component-variant tokens. The previous variant-first order scanned every token even for these fixed
ids; fallback variant matching and title identity semantics remain unchanged.

## Static Evidence

- Fixed control-id classification order: `variant scan -> control-id match` becomes `control-id match -> variant scan`.
- Variant-based section-title matching and ordinary-node rejection are unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR302_SECTION_TITLE_CONTROL_ID_FIRST_BENCH_V1`. It
compares variant-first and control-id-first classification over 10,000 titles with 32 variant tokens
per sample across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation, exact
timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Current batched validation handoff (2026-08-30)

Editor302 is included in ticket `7484a74c23ec418d92154572353fc131` for request
`runtime-editor-356-301-302-20260830-v2`, with source manifest hash
`3fc81ca19f55ebaa5de890c57acbce48ad91446e38db35111a7386ad7e37986a`. The batch runs the three
Runtime356/Editor301/Editor302 behavior and ignored performance groups in one Release invocation.
Cargo, exact p95 evidence, review, commit, push, and WeCom remain coordinator-owned and pending.

## Validation attempt (2026-08-30)

Corrected batch ticket `7484a74c23ec418d92154572353fc131` ended `failed`. The coordinator
provided no valid Cargo, performance, or commit evidence; the external validation resource was
left unchanged and no successful WeCom notification was sent.
