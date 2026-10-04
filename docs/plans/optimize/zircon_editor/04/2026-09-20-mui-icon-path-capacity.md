---
title: Editor852 MUI Icon Path Capacity
category: zircon_editor
report_id: Editor852-mui-icon-path-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor852 - MUI icon path capacity

## Scope

The retained-host MUI icon parser collects every JavaScript `d: "..."` path
element before the SVG document projection. The marker count is available from
the source text, but the collector previously started at zero capacity.

## Optimization

- Reserve the number of `d: "` markers before the existing cursor scan.
- Preserve malformed-value termination, path order, opacity extraction, empty
  input behavior, and the unescaped-value fast path.
- Keep the bound conservative: marker text inside a value may over-reserve but
  can never change parsed output or correctness.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the capacity
reservation existed, then GREEN after production and an isolated lower module
were added (`4/4`). The lower module covers dense path cardinality, empty input,
and the ignored `EDITOR852_MUI_ICON_PATH_CAPACITY_BENCH_V1` Release marker. A
dense 64-path model removes geometric growth (`5 -> 0`).

## Local validation

- `tools/tests/test_editor_mui_icon_path_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The combined eleven-contract Runtime/Editor loader passes `44/44` tests in
  `0.047s`; the broad non-tooling performance/pressure loader passes
  `2402/2402` tests across `656` modules in `33.492s`, with zero load errors,
  failures, errors, or skips.
  Managed Windows Cargo/Release, allocator, and MUI icon parser product
  p50/p95/p99 evidence remain pending behind the external worktree admission
  gate.
  A subsequent single-process recheck passes the same eleven-contract slice
  `44/44` in `0.017s` and the broad batch `2402/2402` across `656` modules in
  `6.455s`; these remain local source/model receipts.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/parser.rs` | `6BDCB819B0654751953DD822B9DF30881FD0FD255A440DC57375150BD022AA7B` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/parser/capacity_tests.rs` | `253F8D7D25C5D3344F05C006FA376D49770B220ED9AEBFD0761399F8E234D42F` |
| `tools/tests/test_editor_mui_icon_path_capacity_performance_contract.py` | `47116467C6D17FB709B7C3480466959C790199FB3CE2C9EC42ECA0F36BE0B4AE` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed managed Windows
Release lane compiles the current Runtime/Editor tree, executes the lower
regression and ignored marker, and supplies allocator plus MUI icon parser
product p50/p95/p99 evidence. Tooling production remains deferred for the
later Rust migration.
