---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: font-fallback-cache-match-and-face-metadata-visibility
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/font/database/face_access.rs
  - zircon_runtime/src/text/font/fallback.rs
  - zircon_runtime/src/text/font/fallback_cache.rs
  - zircon_runtime/src/text/font/vertical_metrics.rs
tests:
  - cargo test -p zircon_runtime --lib text::font --locked --jobs 1 --color never -- --test-threads=1
---

# Text01：fallback cache match 与 face metadata sibling 可见性编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行者：`plugins01-runtime-profile-availability-projection-r3-20260718`
- 来源执行切片：runtime profile broad source guard job `c1fe7621b2bc4aa1b68291f8fa117248` / run `835c9dcd9316494eba57e2f929f1f7df`
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：新增错误位于 Text01 正在修改的 face access 与 fallback cache owner，Plugins01 不应修改 text 内部合同。

## 失败现象与复现证据

raw stderr `.codex/state/session-coordinator/cargo-runs/c1fe7621b2bc4aa1b68291f8fa117248/835c9dcd9316494eba57e2f929f1f7df/stderr.log` 包含 7 条 Text01 编译错误：

- `fallback.rs` 仍有 5 条 E0624，无法调用 `face_covers_all`、`face_covers_codepoint`、`face_coverage_count`；它们属于既有 `font-cache-debug-and-coverage-visibility-compile-regression` lifecycle，当前 owner 已领取该记录，本文件不复制其修复责任。
- `fallback_cache.rs:411` 的 `match composite` 两个分支分别返回 `()` 与 `&mut Hasher`，产生 E0308；`None` 分支需要保持 side-effect 语义并显式归一为 unit。
- `vertical_metrics.rs:28` 无法调用 `database/face_access.rs` 中仅 `pub(super)` 的 `face_metadata`，产生 E0624；该 helper 需要与其他 font sibling 访问一起按最窄边界原子收敛。

job 最终为 `orphaned`、`exit_code=null`、live PIDs empty，且启动后 Text01 源发生 owner 修改，所以这些只作为 captured-source diagnostic，不是 Text01 或 Plugins01 acceptance。

## 最低共享层根因

Text01 的 face access owner 拆分没有把 sibling consumer 的可见性作为一个完整 contract 迁移；同时 fallback cache digest helper 的 `match` 分支遗漏 unit 归一。两者都应在 Text01 当前 generation-owned metadata/cache 切片内一次修复，避免上层消费者各自扩大 API。

## 架构修复验收

- `face_metadata` 与 coverage/vertical helper 采用 `text::font` sibling 所需的最窄可见性，不得扩大为 crate-wide/public API。
- fallback cache composite digest 的 `Some/None` 分支必须都返回 unit，并以 focused test 锁定 `None` 与 `Some` 的稳定、不同 digest 输入语义。
- 保持 generation-owned face metadata 与 fallback candidate cache，不恢复重复 parse、全量候选重建或 clone-on-hit。
- 运行 fresh immutable Text01 focused/broad gate、独立复审、managed commit/fixed return；上游 Plugins01/Layout15/Editor 再按新 SHA 重建门禁。

## 禁止临时方案

- 不得删除 cache/metadata 字段、跳过 digest 分支、用 `pub`/`pub(crate)` 粗暴放宽，或让 Plugins01 修改 Text 源。
- 不得把 orphaned/no-exit run 当作失败测试计数或 GREEN。

## 修复结果与回传

Open state: `implementation_complete / review_green / managed_validation_pending`。fallback composite digest 的 `Some/None` 分支已归一为 unit；metadata/coverage/vertical/glyph-id helper 按 consumer 所需分别限制在 `crate::text::font` 或 `crate::text`，没有扩大成 crate public。generation-owned metadata、fallback cache 与 SDF glyph projection 保持不回退；独立终审 0 Critical / 0 Important / 0 Minor、Ready，仍待 fresh managed focused/broad gate 与 fixed return。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
| --- | --- | --- | --- |
| 2026-07-19 03:46 +08:00 | `open / Text01 待修复` | 在既有 coverage visibility failure 之外，新增记录 `fallback_cache.rs:411` E0308 与 `vertical_metrics.rs:28` `face_metadata` E0624；未修改 Text 业务源码。 | job orphaned、exit null、PIDs empty且 source-raced，仅作诊断；待 Text01 原子修复、immutable focused/broad、复审与 fixed return。 |
| 2026-07-19 08:20 +08:00 | `implementation_complete / review_green / managed_validation_pending` | cache digest unit、sibling visibility、generation-owned glyph map、SDF no-reparse 与 FontDb/TTC shared-byte cells 已原子收敛；positive/negative manifest cache 行为测试随 generation 清空。 | 48 个 leased Rust 文件 rustfmt + scoped diff、9/9 结构断言通过；独立终审 0/0/0 Ready。exact Cargo 尚未轮到，故 failure 继续 open。 |
| 2026-07-19 15:54 +08:00 | `implementation_complete / review_green / managed_validation_pending` | 新增 folder-backed `fallback_cache/tests.rs`，分别锁定 `None`/`Some(composite)` query identity 的重复稳定性与 presence bit 差异；生产 digest 实现未改。 | Rust 1.94.1 rustfmt、snapshot 662 exact 3/3 零漂移；独立增量复审 C0/I0/M0 `READY`。按 milestone-first 规则未在外部 Editor10→Runtime12→Layout15 边界闭合前启动 Cargo，测试尚不得记 GREEN。 |
| 2026-07-28 01:45 +08:00 | `implementation_complete / managed_broad_runtime_passed / editor_upward_running` | Managed Runtime job `8f1c073d40ce4bee8483c046e6ee6b9b` / run `48f0711c4ca1468d90b7545df7c6e047` completed the declared `text::font` broad return. | Exit 0: `79 passed / 0 failed / 2 ignored / 8922 filtered`, covering fallback cache query identity and generation invalidation alongside face metadata visibility consumers. Editor upward result remains required before fixed return. |
| 2026-07-28 02:42 +08:00 | `Text01_runtime_return_passed / external_editor_return_failed` | Editor job `4eefa547982a4bd896813d9fad698f21` / run `ceff37fc13224768af1c365287f242e5` reached and compiled the current Runtime/Text source, then exited 101. | 56 failures are owned by unrelated editor contracts and tests; no Text01 source file appears in diagnostics. Preserve this record as open until the external editor return is green. |
