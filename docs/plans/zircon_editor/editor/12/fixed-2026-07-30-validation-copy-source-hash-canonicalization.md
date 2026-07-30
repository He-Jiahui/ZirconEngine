---
handoff_kind: fixed
status: fixed
created_at: 2026-07-30
summary_slug: validation-copy-source-hash-canonicalization
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/tests/test_cargo_reservations.py
  - tools/session_coordinator/tests/test_server.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations
  - materialized validation copy with a lowercase SHA-256 hash binds a canonical CPU reservation
  - a distinct source-copy hash remains rejected before the Cargo process starts
  - Editor12 source-copy reservation and focused catalog rerun
resolved_at: 2026-07-30
---


# Coordinator01: validation-copy source hash canonicalization mismatch

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：Editor12 immutable core plugin catalog source-bound Cargo gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：validation copy 的持久化 hash、Cargo compatibility canonicalization 和 reservation-to-copy binding 都由 Coordinator01 `CargoJobService` 所有。Editor12 只能提供带 attribution 的 source copy、精确 source manifest 和受管 reservation 请求。

## 失败现象与复现证据

Editor12 source copy `ebd72da0d6bf46109270c0250fdca37a` 已成功 materialized，包含 18,063 个输入、一个 pinned `zr_vm` source，数据库/status 返回的 `inputManifestHash` 为小写：

`a03ed40ef551cd40c0b46542829a309e66f7f01434662707c5a49936fe4f8731`

其 36 个 Editor12 owned source hashes 已与 copy 中的 `zircon_editor/build.rs` 和 `core/plugin/catalog_store.rs` 抽查一致。随后以相同 job id、该 hash、同一 source manifest 和 copy target 调用 `cargo reserve-cpu`，协调器在创建 reservation 前拒绝：

`Cargo reservation source copy is missing, foreign, stale, or incomplete`

没有 CPU reservation、Cargo job、Cargo PID 或 Cargo run。

`CargoCompatibility.canonical()` 将 `source_copy_manifest_hash` 转为大写，而 `_require_source_copy()` 直接比较数据库中保留的小写 `validation_copies.input_manifest_hash` 与 canonical payload。两个值仅大小写不同，因而任何真实小写 SHA-256 source copy 都无法绑定。现有回归使用合成的全大写 `"B" * 64` hash，未覆盖 materializer 实际格式。

## 最低共享层根因

source-copy hash 的 canonical representation 没有跨 materialization persistence 与 reservation binding 统一：compatibility 选择大写，validation copy persistence 保留 `hashlib.hexdigest()` 小写，绑定比较对大小写敏感。问题在 Coordinator01，不在 Editor12 catalog、external source pin、CPU FIFO 或 Cargo 编译。

## 架构修复验收

- source-copy manifest hash 必须在 `CargoCompatibility.canonical()`、`validation_copies.input_manifest_hash` 持久化和 `_require_source_copy()` 比较中采用同一种稳定表示，或比较前在同一边界 canonicalize。
- 添加 focused regression：由实际 materialized copy 产生小写 SHA-256，再以 canonical compatibility + 完整 selected source manifest reserve/consume，必须成功绑定同一 copy root；不允许在共享工作树回退。
- 添加不同 hash 的拒绝回归，确保修复不会放宽 copy identity 或允许 foreign/stale copy。
- 保持 source manifest 的 file-hash 重验、Session owner/status、materialized status、managed root 和 CPU FIFO 约束不变。
- 受管 reload、Python focused validation 和独立 review 接受后，Editor12 以同一 source copy identity 创建新的 CPU reservation，运行 `cargo +1.94.1 test -p zircon_editor --lib --locked core::plugin -- --test-threads=1`；此前不得宣称 Editor12 Rust GREEN、fixed return 或 commit。

## 禁止临时方案

- 不得让 Editor12 手工改写 validation-copy 数据库 hash、重写 source root、降级为无 source-copy reservation 或在共享工作树直接运行 Cargo。
- 不得只修改 payload 一侧、将 hash 比较移入调用方、跳过 source manifest 重验，或用全大写 test fixture 代替真实 materialized copy。
- 不得将这次 reservation admission 拒绝记录为 Rust 编译失败或把现有 copy 作为已运行 Cargo 的证据。

## 产出记录与时间

### 2026-07-30

- 状态：`open`；Coordinator01 source-copy binding handoff 已建立，来源 Editor12 保持 `resolving_failure`。
- 失败记录：source copy `ebd72da0d6bf46109270c0250fdca37a` 成功 materialized，但 `cargo reserve-cpu` 因 source-copy hash 的大小写不一致在 admission 阶段拒绝；没有 reservation、Cargo job、PID、run 或 Rust 测试结果。
- 后续：Coordinator01 修复并完成受管 Python 验证、独立 review、reload 和 failure return 后，Editor12 必须以新的 canonical reservation 重放同一 immutable copy，再进行 focused Cargo gate。

### 2026-07-30 current-source implementation evidence

- Frameworks/Plugins01 的 immutable Catalog copy `0927082468ef4419b090af071059da5e`
  以小写 `inputManifestHash`
  `a82eed6580fd7d7e0c289fda1f6f70ab6099572e8b7553c814c1720e6d0370f4`
  完成 materialize；相同 hash 经 `CargoCompatibility.canonical()` 后在 reservation admission
  被稳定复现为 `Cargo reservation source copy is missing, foreign, stale, or incomplete`，账本未创建
  reservation/job/run。
- TDD RED 将原有合成全大写 copy hash 改为 `hashlib.hexdigest()` 的真实小写形式，唯一失败堆栈落在
  `CargoJobService._require_source_copy()` 的区分大小写比较。生产修复只在该 owner 边界把持久化 hash
  规范化为大写后与 canonical compatibility 比较；Session、materialized status、source root 和逐文件
  source-manifest 重验保持不变。
- focused Python 回归 `2 passed / 0 failed`，覆盖小写 materialized hash 成功绑定以及 distinct hash
  仍返回 `cargo_cpu_reservation_source_copy_invalid`；完整
  `python -m unittest tools.session_coordinator.tests.test_cargo_reservations` 为
  `49 passed / 0 failed / 226.000s`。`git diff --check` 无错误，仅有既有 CRLF 提示。
- 当前状态仍为 `open`：独立复审、共享 coordinator 安全 reload、同一 Catalog source-copy 的真实
  reservation 重放、return record 与原子提交尚未完成。运行中的 foreign Text09 Cargo 必须自然终止，
  不得为 reload 抢占或终止其进程树。

### 2026-07-30 independent review and controlled reload

- exact owner diff 已完成独立复审，结论为 Critical `0` / Important `0` / Minor `0`。复审确认
  `_require_source_copy()` 只规范化数据库中的 materialized hash，再与已经 canonical 的 compatibility
  hash 比较；session、materialized status、source root、逐文件 source manifest 与 distinct-hash 拒绝路径
  均未放宽。
- 完整 `test_cargo_reservations` 仍为 `49 passed / 0 failed`，focused lowercase/distinct-hash 回归为
  `2 passed / 0 failed`；本阶段没有新增 Cargo reservation、job、run 或 PID。
- admission-preserving `service.rollover` 动作
  `9c6760e3a21a4ab0b65829a86065acfb` / intent
  `c8882a8d4e314dbdb7173c34ecd3ff52` 已由控制面接受。它保持 task admission 与 FIFO，不排空、不终止
  foreign Cargo；当前状态为 `executing / waitingForCargo`，将在现有全库 Runtime rustc 自然终止后切换
  successor。
- failure 继续保持 `open`。只有 successor 健康、同一 Catalog validation copy
  `0927082468ef4419b090af071059da5e` 成功创建 canonical source-bound reservation 后，才能写 return record、
  执行 coordinator 原子提交并向来源计划回传 fixed。

## 修复结果与回传

- 根因：validation_copies.input_manifest_hash persisted the materializer lowercase SHA-256 while CargoCompatibility canonicalized the same hash to uppercase, and _require_source_copy compared them case-sensitively.
- 架构修复：Canonicalize the persisted materialized hash at the Cargo source-copy admission boundary while preserving session, materialized status, source root, per-file source manifest, and distinct-hash rejection checks.
- 验证：Focused lowercase/distinct-hash regressions 2/2 GREEN; full test_cargo_reservations 49/49 GREEN; independent review C0/I0/M0; admission-preserving rollover 9c6760e3a21a4ab0b65829a86065acfb succeeded to instance 23ab68c8df8044a98ff0faa1eada842e; live immutable Catalog source-copy reservation d3179b6bf5394717a1e49dfa3eb60d46 was accepted with snapshot1349 71/71 unchanged and 65 source hashes.
- 回传：Coordinator now accepts real lowercase materialized validation-copy hashes after canonical comparison without weakening provenance checks. Catalog reservation d3179b6bf5394717a1e49dfa3eb60d46 is pending in FIFO; its focused/broad result remains owned by the open Plugins01 Catalog failure.
