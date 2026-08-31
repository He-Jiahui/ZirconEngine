# Runtime359 Selector Single-Match Fast Path

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-357-358-359-303-304-20260830-v2`

## Scope

UI selector matching now handles the common single-segment path directly and handles one-token
segments through a dedicated token predicate. Multi-segment combinator traversal and all selector
token semantics remain unchanged; the fast paths remove iterator setup and repeated segment-index
work for the common class/type checks.

## Static Evidence

- Single selector segment: general reverse path traversal -> direct terminal-node segment match.
- Single segment token: generic token iterator -> direct `matches_token` dispatch.
- Source contract and behavior tests cover both fast paths and preserve compound selector results.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME359_UI_SELECTOR_SINGLE_MATCH_BENCH_V1`. It
compares the previous iterator-based single-match shape with the optimized matcher over 2,000,000
matches per sample across 17 interleaved samples and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and
the one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Runtime359 benchmark correction (2026-08-30)

The ignored release benchmark now black-boxes runtime selector and node inputs while invoking the
optimized matcher directly. This removes compile-time constant folding from the comparison; managed
Cargo validation and the 30% p95 gate remain pending.

## Runtime359 focused-test correction (2026-08-30)

The source-contract regression now requires the single-segment fast path to precede, rather than
remove, the multi-segment reverse traversal initialization. The fallback remains required for
combinators; the previous negative assertion was invalid and would fail the focused test. No managed
Cargo result or performance claim is made until the coordinator reruns the batch.

The ignored benchmark baseline now uses a dedicated legacy single-segment helper that retains the
old reverse-index and token-iterator control flow. This removes the prior extra `segments` iterator
and index predicate so the p95 comparison isolates the production fast path; broader selector token
coverage remains a follow-up evidence gap.

## Current batched validation handoff (2026-08-30)

Runtime359 is included in the accepted combined Runtime/Editor batch with Runtime357/358 and
Editor303/304: request `141cfc54bef342968017a4441c534e10`, ticket
`b62653294ca7402d88462819c82cfaeb`, source manifest hash
`d946a6a4cf07a559b828841f51929334afae7e1c432d43947c417d1f8055210e`. The queued command runs all
15 focused and ignored tests in one release invocation with five 30% p95 gates. No terminal Cargo
or performance result is claimed until the coordinator completes it.
