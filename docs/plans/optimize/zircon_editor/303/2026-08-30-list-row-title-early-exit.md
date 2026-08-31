# Editor303 List Row Title Early Exit

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-357-358-359-303-304-20260830-v2`

## Scope

List-row identity now rejects control ids ending in `Title` before invoking the shared component
family resolver. The previous family-first order evaluated role, category, layout, host-role, and
control-id branches for nodes that were deterministically excluded. Non-title list rows and all
family fallback behavior remain unchanged.

## Static Evidence

- Deterministically excluded title nodes: family resolution + suffix check -> suffix check only.
- List-row role, category, layout, host-role, and control-id recognition is unchanged for eligible nodes.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR303_LIST_ROW_TITLE_EARLY_EXIT_BENCH_V1`. It compares
family-first and title-first classification over 10,000 excluded nodes with a 2,048-byte variant
payload per sample across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation, exact
timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Current ownership and static handoff (2026-08-30)

Session `root-editor303-list-row-title-20260830` owns this child directory, this plan, the list-row
identity implementation, and its focused regression test. Exact lease claim request is
`6a221c89290e41ee9fc45ce2645efa36`; current-hash baseline attribution request is
`ea49c044123b4e08bb178585f65c6cc5`. The implementation keeps the `*Title` exclusion before
`is_component_family` and leaves non-title family classification unchanged. Managed Cargo validation,
the release benchmark, independent review, coordinator commit, and WeCom notification remain pending.

## Benchmark correction (2026-08-30)

The ignored release benchmark now uses a runtime `TemplatePaneNodeData` whose family probes all miss
before the `Title` exclusion, black-boxes every classifier input, and invokes the production
`is_workbench_list_row` candidate. This removes the prior first-role short circuit and compile-time
constant-folding risk; managed Cargo validation and the 30% p95 gate remain pending.

## Current batched validation handoff (2026-08-30)

Editor303 is included in the accepted combined Runtime/Editor batch with Runtime357/358/359 and
Editor304: request `141cfc54bef342968017a4441c534e10`, ticket
`b62653294ca7402d88462819c82cfaeb`, source manifest hash
`d946a6a4cf07a559b828841f51929334afae7e1c432d43947c417d1f8055210e`. The queued command runs all
15 focused and ignored tests in one release invocation with five 30% p95 gates. No terminal Cargo
or performance result is claimed until the coordinator completes it.
