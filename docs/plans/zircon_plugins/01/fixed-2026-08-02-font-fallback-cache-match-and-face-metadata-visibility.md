---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-02
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

- 根因：Text01 的 font owner 拆分漏迁 sibling metadata/coverage 可见性，fallback query digest 的 composite match 同时遗漏 unit 归一，造成 E0624/E0308。
- 架构修复：face metadata 与 coverage helper 已收窄为 text::font sibling 可见，composite Some/None digest 分支统一为 unit，并由 absent/present composite identity 回归锁定；generation-owned metadata 与 fallback cache 保持唯一 owner。
- 验证：受管 Runtime Text01 broad job 8f1c073d40ce4bee8483c046e6ee6b9b/run 48f0711c4ca1468d90b7545df7c6e047 为 79 passed / 0 failed / 2 ignored；后继 Editor run ceff37fc13224768af1c365287f242e5 已编译当前 Runtime/Text 源，56 个失败均属于无关 Editor 合同且无 Text01 诊断。2026-08-02 current-source 复核确认修复与两项 identity tests 仍在，相关源码无工作树漂移。
- 回传：Text01 最低 owner 已闭合，Plugins01 可重建其受影响门；无关 Editor failure 继续由各自 owner 保持开放，不再延长本 Text01 lifecycle。
