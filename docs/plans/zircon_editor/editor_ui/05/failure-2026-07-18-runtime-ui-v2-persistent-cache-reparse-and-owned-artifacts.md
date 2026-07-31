---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-v2-persistent-cache-reparse-and-owned-artifacts
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
tests:
  - persistent write zero-source-reparse counter
  - compiled artifact clone-byte and peak-RSS test
  - schema corruption and concurrent writer recovery test
---

# Runtime UI v2 persistent cache重解析源码并深复制artifact

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：v2 file cache/compiler审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：persistent compiled artifact格式、staged build与asset fingerprint由EditorUI05统一定稿。

## 失败现象与复现证据

PERF-MVP-272/308：store path深clone root/compiled/documents，并重开重parse全部source只为恢复aliases；当前compile cache key还序列化根与全部注册imports，hit深clonecompiled tree，persistent asset eviction递归读目录并反序列化候选记录。root style合并另clone documents/tokens/styles，UI caller承担序列化准备。

## 最低共享层根因

首次parse没有保留canonical alias/import metadata，persistent DTO与内存compiled graph分离为两套owned payload，导致落盘必须重新读取源码并复制完整AST/arena。

## 架构修复验收

- 首次parse同时产出可序列化source/alias/import index；每source每generation parse≤1。
- persistent writer借用或消费immutable compiled artifact，额外full clone与source parse=0，并移出UI caller。
- schema/compiler/fingerprint显式版本化；损坏、旧版本、并发writer安全回退且不污染current entry。
- 1/100/1k source记录read/parse/clone/serialized bytes、caller/background CPU和peak RSS。

## 禁止临时方案

- 不得只关闭persistent cache回避重复工作；冷启动compiled artifact仍是MVP目标。
- 不得用更多Arc包住已经重复物化的AST而保留双authority。

## 修复结果与回传

Open state: `等待EditorUI05回传single-parse persistent artifact、后台落盘和版本/损坏验收`。
