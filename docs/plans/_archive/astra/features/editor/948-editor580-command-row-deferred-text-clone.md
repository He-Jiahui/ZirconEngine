---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/580/2026-08-31-command-row-deferred-text-clone.md
related_records:
  - docs/plans/astra/features/editor/947-editor580-persistent-bucket-dense-sort.md
  - docs/plans/astra/features/runtime/891-runtime580-cookie-context-preflight.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/rows/entry.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/rows/entry.rs
---

# Editor948 Editor580 Command-Row Deferred Text Clone

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Command-palette row paint | Borrow label/detail text through clipping and empty-text rejection, taking ownership only when a visible command is emitted; visible contents, style, geometry, order, and clipping remain unchanged. | Behavior contract verifies offscreen borrowed text emits no commands. |
| 性能门禁 | 65,536 offscreen label/detail projections avoid allocating 1,728-byte text values. | ignored marker `EDITOR580_COMMAND_ROW_DEFERRED_TEXT_CLONE_BENCH_V1` requires optimized P95 ≤50% of legacy; managed Editor Release receipt remains pending. |

- `entry.rs` contains the Editor580 behavior contract and marker.
- No tooling changes; the combined Runtime580/Editor580 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor580 was included in the related `optimization_batch_gz` submission:
Runtime development PTY `81972`, Editor development PTY `21257`, Runtime02
Release PTY `20043`, and Editor Release PTY `50976`. All four wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gy` replacement wave uses Runtime development PTY
`72314`, Editor development PTY `76797`, Runtime02 Release PTY `83368`, and
Editor Release PTY `99044`; these wrappers remain intentionally unpolled.
