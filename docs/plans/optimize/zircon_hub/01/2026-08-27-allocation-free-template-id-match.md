---
title: Hub01 Allocation-Free Template ID Match
category: zircon_hub
report_id: Hub01-allocation-free-template-id-match-2026-08-27
date: 2026-08-27
session_id: root-hub01-allocation-free-template-id-match-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub01 Allocation-Free Template ID Match

## Scope

New-project action parsing resolves the requested template through
`ProjectTemplate::from_enabled_id`. The previous implementation trimmed every input and then
materialized an ASCII-lowercased `String` before matching the only enabled template ID.

The parser now borrows the trimmed input and uses `eq_ignore_ascii_case` directly. This preserves
the existing ASCII case-insensitive behavior, surrounding-whitespace acceptance, strict non-ASCII
behavior, and rejection of disabled or unknown IDs while removing the temporary string allocation.
The template catalog and its single enabled `renderable-empty` entry are unchanged.

## Performance Evidence

The isolated release model performs 65,536 lookups across exact, uppercase, padded, disabled,
unknown, and empty inputs. It runs 31 alternating sample pairs and 16 rounds per sample. The model
was compiled with `rustc -O` on Windows.

| Metric | Lowercase temporary | Borrowed ASCII comparison | Change |
|---|---:|---:|---:|
| Allocator calls per 65,536 lookups | 49,152 | 0 | -100.000% |
| Cumulative requested bytes | 688,128 | 0 | -100.000% |
| P50 for 16 rounds | 79,214,700 ns | 18,713,500 ns | -76.376% |
| P95 for 16 rounds | 114,554,800 ns | 28,289,400 ns | -75.305% |

Model source:
`.codex/state/session-coordinator/hub01-allocation-free-template-id-match-model.rs`.

## Contracts And Validation

- `tools/tests/test_hub01_allocation_free_template_id_match_performance_contract.py` locks the
  borrowed trimmed comparison, absence of lowercase/string construction, enabled-template
  mapping, and Rust behavior coverage.
- Rust behavior tests cover padded uppercase acceptance and strict non-ASCII rejection in addition
  to the existing exact enabled/disabled template assertions.
- Local source-contract result: 3 tests passed.
- Local `rustfmt +1.94.1 --edition 2021 --check` passed for the production file.
- The release model passed zero-allocation and P50/P95 reduction gates.
- Cargo compilation and the complete `create_project_request::tests` module remain pending in the
  managed asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Hub01 still owns project/store authority, process supervision, build and package lifecycle,
streaming receipt hashing, device installation, crash recovery, and same-hardware scale gates.
This slice only removes repeated template-ID normalization allocation from action parsing.
