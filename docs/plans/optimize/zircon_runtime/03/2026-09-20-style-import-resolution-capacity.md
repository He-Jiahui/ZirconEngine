---
title: Runtime851 Style Import Resolution Capacity
category: zircon_runtime
report_id: Runtime851-style-import-resolution-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime851 - style import resolution capacity

## Scope

The UI style resolver resolves each document style import into a borrowed list
before flattening imported and local stylesheets. The import count is known
from the document, but the temporary `imported_styles` vector previously used
iterator `collect` from zero capacity.

## Optimization

- Reserve `document.imports.styles.len()` before resolving imports.
- Preserve one style-map lookup per import, unknown-import error precedence,
  borrowed lifetimes, and widget/import/local stylesheet order.
- Keep the existing exact final `sheets` reservation and Runtime358 behavior
  unchanged; this slice removes only the intermediate import-list growth.
- Refresh the adjacent Runtime358 source-order guard so it recognizes the
  capacity-reserving declaration while retaining the legacy marker fallback.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the bounded
loop and reservation existed, then GREEN after production and an isolated
folder-backed lower module were added (`4/4`). The lower module covers the
single-lookup/error-order shape and the ignored
`RUNTIME851_STYLE_IMPORT_RESOLUTION_CAPACITY_BENCH_V1` Release marker. A
dense 64-import model removes geometric growth (`5 -> 0`).

## Local validation

- `tools/tests/test_runtime_style_resolver_import_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The refreshed eleven-contract Runtime/Editor source/model loader with
  Runtime851 included passes `44/44` focused tests in `0.047s`; the broad
  non-tooling performance/pressure loader passes `2402/2402` tests across `656`
  modules in `33.492s`, with zero load errors, failures, errors, or skips.
  Managed Windows Cargo/Release,
  allocator, and UI style-resolution product p50/p95/p99 evidence remain
  pending behind the external worktree admission gate.
  A subsequent single-process recheck passes the same eleven-contract slice
  `44/44` in `0.017s` and the broad batch `2402/2402` across `656` modules in
  `6.455s`; these remain local source/model receipts.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver.rs` | `50B0F4F819CE8FB21C6A9E06456D985A6E2B4373897FA64EFE484ACC145F1E35` |
| `zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver/capacity_tests.rs` | `19A5A551E778109AC625D67A1F5AA2CFD1C3D6E990215F369ADFB645C31B3B25` |
| `zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver/import_capacity_tests.rs` | `2A95999ACE4ED23EB378C6374652382F9BC27F563C3CD0696558553A4EA7D18E` |
| `tools/tests/test_runtime_style_resolver_import_capacity_performance_contract.py` | `0FBED1D9407044202A6544781FAEC423F57F66BC52D92FCBF6BCC555E5F9F646` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed managed Windows
Release lane compiles the current Runtime/Editor tree, executes the lower
regression and ignored marker, and supplies allocator plus UI style-resolution
product p50/p95/p99 evidence. Tooling production remains deferred for the
later Rust migration.
