---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: native-bitmap-retry-count-type
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib native_bitmap_atlas --locked --jobs 1 --color never -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1
---

# Text04: native bitmap retry count type

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 来源执行切片：Plugins09 compact validate-report failure closeout current-source successor
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 交接原因：Text04 owns native bitmap atlas retry selection and its stale-glyph accounting. Plugins09 does not own the dirty Text source that prevents the Runtime crate from compiling.

## 失败现象与复现证据

Managed Plugins09 job `8c1c54fc525949bba6d55ee155eef689` / run
`d047cee1d7104d3babd465f4831df617` ran
`cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1`
against the current shared source. Rust 1.94.1 reported E0689 at
`retry_frame.rs:93` and `retry_frame.rs:112`: `discarded_stale_retry_glyph_count`
was initialized from an untyped integer literal, so `saturating_add` could not resolve
the receiver type before the later `usize` sum. The process exited 101 before any
Plugins09 test executed; raw stderr is retained under the managed job/run directory.

## 最低共享层根因

The new Text04 stale-retry accounting combined an untyped zero, a literal increment,
and a `VecDeque::len()` aggregate through `saturating_add` without declaring the
counter's owner type. The contract is a collection cardinality and therefore must be
`usize` at its declaration.

## 架构修复验收

- Declare the stale retry glyph counter as `usize` at its owner; both increments must remain checked through `saturating_add`.
- Run the focused Text04 native bitmap atlas tests through the managed Rust 1.94.1 lane with exit 0 and no live PIDs.
- Re-run the original Plugins09 current-source command and require its bin and integration tests to execute and pass.
- Preserve immutable pre/post source attestation and obtain an independent exact-scope review before managed commit and failure return.

## 禁止临时方案

- Do not cast individual operands, replace checked addition with unchecked arithmetic, or duplicate the count in another type.
- Do not add aliases, compatibility shims, silent fallback, test-only bypasses, or call-site exceptions.
- Do not weaken either managed acceptance command to hide the Runtime compile failure.

## 修复结果与回传

Open state: `待验证`; the owner type is repaired, but no pass is claimed until the
focused Text04 gate and the original Plugins09 upward gate both complete on current
source.
