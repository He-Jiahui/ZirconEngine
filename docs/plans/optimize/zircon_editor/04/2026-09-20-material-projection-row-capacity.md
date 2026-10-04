---
title: Editor856 material projection row capacity
category: zircon_editor
report_id: Editor856-material-projection-row-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor856 - material projection row capacity

## Scope

`MaterialEditorProjection` builds property and texture-slot rows from a shader
schema followed by material-only overrides. Both input collections expose a
safe upper bound, but the row vectors previously started at zero capacity and
grew geometrically during every projection.

## Optimization

- Reserve `shader_schema_len + material_override_len` for property rows.
- Reserve `shader_texture_slot_len + material_texture_slot_len` for texture
  rows, using saturating addition for hostile authored sizes.
- Preserve shader-declared order, append only unknown material overrides, and
  keep duplicate suppression, row payloads, and empty-input behavior unchanged.
- Keep diagnostic-row construction and the existing Editor624 membership index
  optimization outside this slice.

## TDD and deterministic evidence

The new Python source/model contract was intentionally run RED before the
capacity declarations and lower owner existed (`2` structural failures and one
missing lower file), then GREEN at `4/4`. The lower Rust owner checks that
schema order and unknown-override append order remain stable and carries the
ignored `EDITOR856_MATERIAL_PROJECTION_ROW_CAPACITY_BENCH_V1` marker. A dense
8,192-row model changes the zero-capacity vector's modeled geometric growth
events from `12` to `0`.

## Local validation

- `tools/tests/test_editor_material_projection_row_capacity_performance_contract.py`:
  `4/4`.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners.
- The batched non-tooling performance/pressure loader now covers `659` files
  and passes `2424/2424` tests in `52.778s`, with zero load errors, failures,
  errors, or skips; this includes the new Editor856 contract.
- The current expanded non-tooling source-contract loader (filenames containing
  `performance` or `contract`, excluding `tooling`, `export`, and
  `coordinator`) covers `962` files and passes `4072/4072` tests in `139.499s`,
  with zero load errors, failures, errors, or skips. The earlier `961`-file /
  `4068`-test receipt is pre-Runtime856, and the `957`-file / `4042`-test
  receipt is pre-Editor856 historical context.
- No managed Windows Cargo/Release validation command was started by this
  session. Allocator and Material Editor product p50/p95/p99 evidence remain
  pending behind the shared external worktree gate; individual contract tests
  may still exercise their own isolated subprocess fixtures.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/material_editor/projection.rs` | `934616A0E4FD165D7B92F86894F4537816555B75E540E127F9D05B3BCD0D7628` |
| `zircon_editor/src/ui/material_editor/projection/row_capacity_tests.rs` | `574BC2AA5DBCF1A02AA77EFCAB1B3E8CB2D577BCC389589B9C83EBD88B32BAA2` |
| `tools/tests/test_editor_material_projection_row_capacity_performance_contract.py` | `695EF159793F2F758FC5726B2DC54C6ED41E3F6617C2D33F40D39DFBE557AFE3` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
executes the lower regression and ignored marker, and supplies allocator plus
Material Editor p50/p95/p99 measurements. Tooling production remains deferred
for the later Rust migration; no coordinator status is polled here.
