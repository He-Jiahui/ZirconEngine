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

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / managed_validation_pending`.

- `NativeBitmapAtlasRetrySourceSelection::discarded_stale_retry_glyph_count` is declared as `usize` at its owner and is accumulated with `saturating_add`; the frame report keeps the same cardinality type end to end.
- The retry-frame owner keeps stale-glyph selection, face invalidation, deduplicated source selection, per-frame source/byte caps, and retry reporting in the native bitmap atlas domain. No consumer-side cast, compatibility alias, or fallback bypass was added.
- Current source inspection confirms the retry counter declaration and the native bitmap retry-frame test owner both use `usize`. This records the implemented type repair and static contract review only.
- The focused Text04 and original Plugins09 upward commands remain coordinator-owned. Keep this failure `open` until fresh current-source receipts show both gates executed; do not treat the prior compile-stop as a pass.

## 2026-09-11 failure rolling repair

- Primary session `failure-roll-01a084c8-text04-native-bitmap-retry-r1` re-claimed the failure record and `retry_frame.rs`. Current source confirms `discarded_stale_retry_glyph_count` is explicitly `usize` at both selection and frame-report boundaries; edition-2021 rustfmt and scoped diff checks pass.
- Snapshot 3422 freezes the exact owner source. Corrected request `text04-native-bitmap-retry-20260911-r2` submitted `cargo +1.94.1 test -p zircon_runtime --lib native_bitmap_atlas --locked --color never -- --nocapture --test-threads=1`; the earlier request containing coordinator-owned `--jobs 1` was rejected as an input error and is not evidence.
- Admission returned `validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`; no ticket, Cargo execution, or dynamic pass exists. This lifecycle remains open and the session is `waiting_validation`; once the external owner supplies a clean revision, rerun this focused test and the declared Plugins09 upward command before return/closeout.

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-text04-native-bitmap-retry-r2` reclaimed the
  archived exact-path ownership through coordinator transfer fingerprint
  `b9dbfe6808b6e86ab1397f5a627c2d445efcbca901ad36ebb6e7d6841e9de0f4` at baseline epoch
  `611`; no source bytes were changed during attribution.
- Formal non-Cargo source-contract ticket `6a2a81518c4a49f7aae4c0006ac7c2b2` was admitted
  from request `failure-roll-01a084c8-text04-native-bitmap-retry-20260919-r1` and is
  currently `queued`. Its sealed source-manifest hash is
  `2e8c59bce8736c0b1dfb4385a631aad2d57eaaa50204f8c53cc506e50763381f`:

  | path | SHA-256 |
  | --- | --- |
  | `docs/plans/zircon_runtime/text/04/failure-2026-07-29-native-bitmap-retry-count-type.md` | `f5edffcba9585e884eb259eda726fa22cd18e7347372ce54f3337feb9f9924fc` |
  | `zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs` | `b2c92af78a5b1ddc6783055b1f25a1c48930b9ae0ee3e9230309ca2de977d921` |

- The ticket executes a Windows PowerShell/rustfmt source-contract parse proving the owner
  declaration and checked `usize` accumulation. It explicitly defers the focused Text04
  Cargo test, original Plugins09 upward command, independent C/I/M review, canonical fixed
  return and closeout. The prior Cargo admission blocker
  `validation_ticket_external_worktree_dirty:E:\\Git\\zr_vm` remains retained and is not
  converted into a test result.
- Failure remains `open`; no fixed return, commit, or notification is claimed.

### Formal source-contract ticket terminal result

- Ticket `6a2a81518c4a49f7aae4c0006ac7c2b2` completed `passed` at
  `2026-09-19T04:44:17.860313Z` (exit code 0) with immutable output
  `TEXT04_NATIVE_BITMAP_RETRY_USIZE_SOURCE_CONTRACT_PARSE_PASS`. Sealed manifest hash:
  `2e8c59bce8736c0b1dfb4385a631aad2d57eaaa50204f8c53cc506e50763381f`.
- This is only the current-source static contract result. Focused Text04 Cargo, Plugins09
  upward validation, independent C/I/M review, fixed return and closeout remain pending; the
  external `E:\Git\zr_vm` dirty-worktree admission blocker is retained.

## 2026-09-21 current-source review successor

- Successor Session `failure-roll-01a084c8-text04-native-bitmap-retry-r3` reclaimed only this
  record and `retry_frame.rs`. Snapshot `3713` binds the current source to
  `b2c92af78a5b1ddc6783055b1f25a1c48930b9ae0ee3e9230309ca2de977d921`; no source bytes changed.
- An initial local source checker expected two occurrences of the `: usize` owner spelling and
  correctly failed because the two structure fields plus the local owner declaration make three.
  The corrected checker separately requires the two fields, the one initialized local owner, the
  two `saturating_add(1)` updates, the selection-to-frame report projection, and no counter cast.
  It passed with marker `TEXT04_NATIVE_BITMAP_RETRY_USIZE_CURRENT_SOURCE_PASS`, including pinned
  Rust 1.94.1 `rustfmt --check`.
- Managed static-only ticket `b87e83f436d94d078775c34b566db97b`
  (`...-20260921-static-r1`) passed against source-manifest
  `0aa8b727f6230ea87a500cb7263d433ba208e601633fd6e41d1f1a92f648f99e`.
  Coordinator job `681c92e707ea47df9c2a13386f1644bd` / run
  `b87e83f436d94d078775c34b566db97b` exited 0 with
  `TEXT04_NATIVE_BITMAP_RETRY_USIZE_CURRENT_SOURCE_PASS`. It is explicitly
  `fullCoverage: false` / `staticParseOnly: true` and does not replace the focused Text04 Cargo or
  Plugins09 upward gates.
- Independent current-source review of snapshot `3713` found
  `Critical=0 / Important=0 / Moderate=0`. It confirmed the explicit `usize` owner through
  selection, retry frame, outer frame, and prepare report, the two saturating increments, direct
  no-cast projection, and the existing stale/matching/duplicate-source behavior coverage. The
  reviewer did not run Cargo and separately preserved both dynamic gates as pending.
- The external `E:\Git\zr_vm` worktree still has 157 foreign changes. It was not modified; no
  duplicate Cargo request, fixed return, closeout, commit, or notification is claimed.
