# Runtime358 Style Sheet Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-357-358-359-303-304-20260830-v2`

## Scope

UI style resolution now computes the known imported stylesheet count and reserves the final sheet
vector capacity before appending Runtime widget, imported, and local styles. Style lookup order and
missing-import errors are unchanged; repeated vector growth and relocation are removed from the
compile path.

## Static Evidence

- Style sheet vector capacity: implicit growth -> one exact `Vec::with_capacity` reservation.
- Widget, imported, and local stylesheet ordering remains unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME358_UI_STYLE_SHEET_CAPACITY_BENCH_V1`. It compares
growing and pre-sized vectors over 2,048 sheets and 2,048 builds per sample across 17 interleaved
samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation, exact
timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Current ownership and static handoff (2026-08-30)

Session `root-runtime358-style-capacity-20260830` owns this child directory, this plan, the style
resolver implementation, and its focused regression test. Exact lease claim request is
`fbc1aacf8e7944af9d1da925ecf119e9`; current-hash baseline attribution request is
`f269d069dd7240b48eaed63fb98f87a9`. The source-level contract confirms one exact total-sheet
capacity reservation, widget/import/local ordering, and no widget-style clone. Managed Cargo
validation, the release benchmark, independent review, coordinator commit, and WeCom notification
remain pending.

## Runtime358 lookup correction (2026-08-30)

Style imports are resolved once into a borrowed list before the exact sheet-capacity reservation,
removing the previous second `BTreeMap` lookup per import while preserving unknown-import and rule
ordering behavior. The ignored benchmark now includes map resolution and imported-sheet merging;
managed Cargo validation and the 30% p95 gate remain pending.

## Runtime358 focused-test correction (2026-08-30)

The ordering regression now anchors on the `imported_styles` resolution and append loop used by the
optimized implementation. The prior assertion searched for the removed direct reference loop and
would fail before exercising production behavior; this source-contract defect is fixed. No managed
Cargo result or performance claim is made until the coordinator reruns the batch.

The release benchmark's baseline invariant was also corrected from multiplying the already-total
imported sheet count by the import count to the actual `imported_count == SHEETS` check. This keeps
the benchmark executable under debug instrumentation without changing the measured paths.

## Current batched validation handoff (2026-08-30)

Runtime358 is included in the accepted combined Runtime/Editor batch with Runtime357/359 and
Editor303/304: request `141cfc54bef342968017a4441c534e10`, ticket
`b62653294ca7402d88462819c82cfaeb`, source manifest hash
`d946a6a4cf07a559b828841f51929334afae7e1c432d43947c417d1f8055210e`. The queued command runs all
15 focused and ignored tests in one release invocation with five 30% p95 gates. No terminal Cargo
or performance result is claimed until the coordinator completes it.
