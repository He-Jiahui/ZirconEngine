---
title: Hub02 Allocation-Free Settings Value Parsing
category: zircon_hub
report_id: Hub02-allocation-free-settings-value-parsing-2026-08-27
date: 2026-08-27
session_id: root-hub02-allocation-free-settings-value-parsing-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub02 Allocation-Free Settings Value Parsing

## Scope

Settings draft and submission paths parse build-profile and language values through
`BuildProfile::from_ui_value` and `HubLanguage::from_ui_value`. Both parsers previously trimmed
their input and then allocated an ASCII-lowercased `String` before matching a small static alias
set.

Both parsers now borrow the trimmed input and compare it directly with `eq_ignore_ascii_case`.
The build parser preserves its `debug` and `release` values. The language parser preserves
`english`/`en` and `chinese`/`zh`/`cn`, including the original ordering. Surrounding whitespace,
ASCII case folding, unknown-value rejection, and strict non-ASCII behavior are unchanged.

## Performance Evidence

The isolated release model performs 65,536 profile lookups and 65,536 language lookups per
exercise across exact, uppercase, padded, alias, unknown, empty, and non-ASCII inputs. It runs 31
alternating sample pairs and 16 rounds per sample. The model was compiled with `rustc -O` on
Windows.

| Metric | Lowercase temporaries | Borrowed ASCII comparisons | Change |
|---|---:|---:|---:|
| Allocator calls per exercise | 94,666 | 0 | -100.000% |
| Cumulative requested bytes per exercise | 491,535 | 0 | -100.000% |
| P50 for 16 rounds | 168,869,100 ns | 29,791,900 ns | -82.358% |
| P95 for 16 rounds | 456,875,000 ns | 90,974,000 ns | -80.088% |

Model source:
`.codex/state/session-coordinator/hub02-allocation-free-settings-value-parsing-model.rs`.

## Contracts And Validation

- `tools/tests/test_hub02_allocation_free_settings_value_parsing_performance_contract.py` locks
  borrowed trimmed parsing, the complete two-profile/five-language alias set, absence of temporary
  lowercase strings, and Rust boundary assertions.
- Existing Rust settings parsing coverage now includes padded profile/language values, both
  language families, unknown profile rejection, and strict non-ASCII rejection.
- Local source-contract result: 3 tests passed.
- Local `rustfmt +1.94.1 --edition 2021 --check` passed for the production file.
- The release model passed zero-allocation and P50/P95 reduction gates.
- Cargo compilation and the focused settings parser behavior test remain pending in the next
  managed asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Hub02 still owns web-shell authority, catalog/settings lifecycle, team/cloud data, accessibility,
frontend performance budgets, and broad product qualification. This slice only removes temporary
normalization allocations from settings value parsing.
