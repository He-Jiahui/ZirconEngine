# Runtime357 Single Node Validation Lookup

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-357-358-359-303-304-20260830-v2`

## Scope

UI asset node validation now uses one `BTreeMap::entry` lookup to distinguish an existing identical
subtree from a conflicting duplicate and to insert a first-seen node. The prior `get` plus `insert`
sequence performed two tree traversals for every new node; duplicate and action-reference error
semantics are unchanged.

## Static Evidence

- Node validation map traversals for new nodes: `2 -> 1`.
- Empty-id rejection, identical-subtree fast path, conflicting duplicate error, and child traversal are unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME357_UI_DOCUMENT_NODE_ENTRY_BENCH_V1`. It compares
the prior `get` plus conditional `insert` path with `entry` over 2,048 keys and 4,096 lookups per
sample across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation, exact
timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Runtime357 source/test correction (2026-08-30)

The production `Entry` import was restored so the `BTreeMap::entry` implementation is compile-valid.
The release benchmark remains ignored but now uses runtime-generated node ids and black-boxes the
lookup inputs; managed Cargo validation and the 30% p95 gate remain pending.

The benchmark intentionally measures the new-node insertion path with unique ids in a fresh map per
lookup. It does not claim duplicate/`Occupied` branch coverage or full validator cost; those semantic
cases remain covered by the focused source/behavior tests and require separate production-shaped
timing evidence.

## Current batched validation handoff (2026-08-30)

Runtime357 is included in the accepted combined Runtime/Editor batch with Runtime358/359 and
Editor303/304: request `141cfc54bef342968017a4441c534e10`, ticket
`b62653294ca7402d88462819c82cfaeb`, source manifest hash
`d946a6a4cf07a559b828841f51929334afae7e1c432d43947c417d1f8055210e`. The queued command runs all
15 focused and ignored tests in one release invocation with five 30% p95 gates. No terminal Cargo
or performance result is claimed until the coordinator completes it.
