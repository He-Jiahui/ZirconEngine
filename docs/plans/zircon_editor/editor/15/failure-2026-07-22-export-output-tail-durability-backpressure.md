---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: export-output-tail-durability-backpressure
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/15
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/output_tail.rs
tests:
  - tools.tests.test_editor15_export_generation_inventory_contract
  - 1-byte/64KiB chunks and 1MiB/1GiB output storm
---

# Editor15：export output tail与durability backpressure

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-558 export output tail and durability backpressure
- 修复责任计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 交接原因：export output tail、terminal durability 与 Runtime11 persistence ticket 的消费边界由 Editor15 所有。

## 失败现象与复现证据

PERF-MVP-558复核确认完整日志已流式落盘、line cap为16KiB、tail限512行；本轮已让partial line按suffix继续扫描，静态合同10/10。剩余tail满后每行`Vec::remove(1)`搬移约511个String，line又clone给tail/event；finish顺序`sync_all` stdout、stderr、manifest，慢盘会占用执行/reader链路。

## 最低共享层根因

bounded tail 仍用 `Vec::remove` 线性搬移并复制多份 `String`，terminal durability 又让执行/reader 链路串行等待 stdout、stderr 与 manifest 三次同步 barrier。

## 架构修复验收

- bounded ring/shared line owner，tail逐出O(1)，UI边界才物化；terminal不得因背压丢失。
- Runtime11 artifact persistence ticket按entry+bytes+age/deadline合并flush/fsync并原子commit manifest；caller/reader不串行等待三次barrier。
- 1B/64KiB chunks、1/16KiB lines、1k/1M lines、1MiB/1GiB output记录scan bytes、String clone/move、queue bytes/age、fsync与p95/RSS。
- 保持完整log、digest、truncation marker、stdout/stderr顺序、cancel/failure与resume语义；current-source Cargo/export E2E通过前保持open。

## 禁止临时方案

不得丢terminal/error或只缩小tail掩盖搬移；不得把三次同步I/O挪到无界私有线程/队列；不得恢复whole-output内存Vec。

## 修复结果与回传

Open state: `待 Editor15 建立 O(1) tail owner 与有界 durability ticket，并回传 current-source Cargo、export E2E 和规模证据`。

## 2026-09-19 successor intake (failure-roll-01a084c8-editor15-export-tail-r2)

The current coordinator index admitted a fresh Editor15 successor and applied
ownership-transfer fingerprint
`299e8181153aaf6c8a884220904825307917ae96a822dd0a43ae69903764cc1e` for this
failure record and both production files. The exact scope was leased and
baseline-attributed before any edit. Current hashes at intake were:

- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs`
  `bc8d96d7b379324b170efc8d7d924cbcb3e7bf61af9eb3b02b07788d7fbbb0d3`
- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/output_tail.rs`
  `6ecd6fbc5ab6ce136f808d71865fb149d9b194463d0ac0c4863c40db8686dbaa`

Current-source correction: `push_bounded_output_line` already uses a
`VecDeque` O(1) eviction path, so the historical `Vec::remove(1)` description
is retained as the original evidence, not as a claim about the current bytes.
The remaining owned-path concerns are the per-line clone into the tail/event
paths and the synchronous stdout/stderr/manifest `sync_all` sequence. The
Runtime11 persistence owner must supply the merged durability ticket; Editor15
must not hide that dependency with an unbounded queue or a private fsync
thread.

### Coordinator validation receipt

- Request: `failure-roll-01a084c8-editor15-export-tail-20260919-r1`
- Ticket: `5f36d6ca87c94144a6657b53b9d731d3`
- Source-manifest hash:
  `c2ff6455ce4fb459d6d72ee3f47aa9d139dd5926b79df49c69a3fe488dc61321`
- Admission: `queued`, execution kind `pending`; no dynamic output or pass is
  claimed. The static contract is limited to the current O(1) bounded-tail
  implementation and its chunk/retention regressions. Per-line clone removal,
  Runtime11 merged durability, scale, Cargo/E2E, review, return, and closeout
 remain required.

The static contract ticket completed on managed job/run
`308e73018e734163b192b8d5795e9634` with exit code 0 and marker
`EDITOR15_OUTPUT_TAIL_O1_SOURCE_CONTRACT_PARSE_PASS`; cleanup completed. This
receipt verifies only the current bounded-tail source contract. Runtime11
merged durability, per-line clone removal, scale/Cargo/E2E gates, independent
review, canonical return and closeout remain pending.
