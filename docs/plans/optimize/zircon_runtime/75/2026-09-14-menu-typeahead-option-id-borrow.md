---
title: Runtime75 Menu Typeahead Option ID Borrow
category: zircon_runtime
report_id: Runtime75-menu-typeahead-option-id-borrow-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Typeahead Option ID Borrow

## Scope

Every menu typeahead key event materializes `OptionEntry` values and then formerly cloned every
entry ID into a second `Vec<String>` solely to resolve the current index. The existing entries are
already the authoritative ordered source. This Runtime75 slice searches those IDs by borrow and
does not change typeahead matching or current-index precedence.

## Implementation

- `apply_keyboard_text` calls `current_option_entry_index` directly on the parsed entry slice.
- The helper preserves the prior `focused_index`, `selected_index`, then
  `value`/`value_text`/`group_value` precedence and returns the first matching entry index.
- Removed the per-keystroke cloned option-ID vector; entry order, matching, disabled checks,
  wrapping, typeahead buffer publication, and focus state remain unchanged.
- Added a lower current-value precedence regression and ignored
  `RUNTIME765_MENU_TYPEAHEAD_OPTION_ID_BORROW_BENCH_V1` Release marker.

## Deterministic work boundary

For `N` parsed menu options, typeahead no longer clones `N` IDs just to locate the current value.
The existing ordered entry scan still performs the same first-match lookup. This is allocation-shape
evidence only, not allocator, CPU/RSS, or product keyboard p50/p95/p99 evidence.

## Validation

- TDD source contract was RED before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_menu_typeahead_option_id_borrow_performance_contract.py`.
- The combined Runtime/Editor menu, command-palette, keyboard, TreeView, input, and capacity
  contract invocation passed `62/62` in `0.061s` through `python -B -m unittest`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
