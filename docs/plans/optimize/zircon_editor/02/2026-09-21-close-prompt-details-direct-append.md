---
title: Editor Close Prompt Details Direct Append
category: zircon_editor
report_id: Editor888-close-prompt-details-direct-append-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor888 Close Prompt Details Direct Append

## Finding

Every retained close-prompt refresh gathered its maximum three visible dirty
document names into a temporary `Vec<&str>` and joined that vector before
optionally appending the overflow marker. The final details string is required,
but the intermediate reference vector is not.

## Optimization

- Append the optional Active Scene label and dirty view titles directly into
  one output `String`.
- Use the source position, rather than output emptiness, to place delimiters so
  an empty authored title retains the exact legacy commas.
- Preserve source order, the three-item visibility cap, the total-count
  overflow decision, and the exact `", ..."` suffix.

## TDD and deterministic evidence

The Editor888 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions lock empty input, the project-scene prefix, empty
titles, two- and three-item output, overflow, and exact equality with the
retired collect/join implementation.

Across 4,096 renders with three visible names, the deterministic model changes
temporary reference-vector slots from `12288` to `0`; the required output
string remains. Ignored marker
`EDITOR888_CLOSE_PROMPT_DETAILS_DIRECT_APPEND_BENCH_V1` emits 101 alternating
p50/p95/p99 sample pairs and requires direct-append p95 to remain within 10%
of collect/join.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor888 and Runtime869 contracts pass `10/10`; adjacent document-toolkit
  and active-scene contracts add `18/18` passing tests.
- A wider 29-test attempt has one unrelated current-tree failure:
  `test_runtime_module_family_boundary` still expects 16 Navigation Rust files
  after a foreign untracked lower test raised the actual count to 17. Python
  tooling repair remains deferred as requested.
- Editor888 received no per-task Cargo run and was submitted with Runtime869 in
  asynchronous v15 (PID `15424`).

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text.rs` | `938EB0704E3C0FF8404E7100495E0CF0741AF1559329F09FB4FF5E71EC25AD57` |
| `zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text/direct_append_tests.rs` | `66BA322FAADA11F0AB7C5E6423A106A294BF72CA3D33C5A09DA386C0878749E5` |
| `tools/tests/test_editor888_close_prompt_details_direct_append_performance_contract.py` | `DC21EE627E7C090580205F55E45884E0EA6599417DEDA286A23D7ABA6881D584` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
close-prompt p50/p95/p99 evidence. The deterministic allocation model is not
product acceptance.
