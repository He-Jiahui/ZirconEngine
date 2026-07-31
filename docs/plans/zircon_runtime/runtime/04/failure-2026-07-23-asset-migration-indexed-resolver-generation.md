---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: asset-migration-indexed-resolver-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/migration/resolver.rs
  - zircon_runtime/src/asset/migration/resolver_index.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/mod.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/resolver_index.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::migration::project_commandlet::resolver_index --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime04：asset migration indexed resolver generation缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration 性能审查 PERF-MVP-511；经批准从 single-inventory lifecycle 拆分
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：logical→physical root identity 与 migration resolver 属于 Runtime04；本 lifecycle 与已 fixed 的 scan/run/sidecar/mod/source-boundary scope 不重叠。

## 失败现象与复现证据

`MigrationResolver::project_relative_path` 对每个 reference 遍历全部 roots，并调用 filesystem-backed `persisted_source_path_for_locator` 重新探测 physical path。single-walk inventory 尚未发布 logical locator→唯一 root-relative physical identity 的查询索引。

## 最低共享层根因

resolver 没有消费 generation-owned physical identity index，仍把 filesystem probe 当作每次 reference 解析的 root authority。

## 架构修复验收

- resolver 只消费 generation-owned typed index，per-reference stat/read/probe=0，lookup 为 O(1) 或有序 O(logN)。
- inventory 发布 logical project-root、canonical physical-root 与 root-relative projection；physical alias 合并但 distinct logical roots 对同 locator 保持 ambiguous。
- direct file locator 与经 sidecar preflight 验证的 compound `.zmeta` binding 在 generation build 时一次入索引；禁止 resolver 二次 parse sidecar 或访问 filesystem。
- missing、ambiguous、link/reparse、compound `.zmeta` 与 registry conflict 的 typed error/优先级不放宽。
- 已注册 GUID 继续是权威；stale/occupied path hint 走既有 repair，不得顺手改为 RegistryConflict。Duplicate registry entries 仍在 SidecarPreflight/AssetRegistryIndex build 阶段优先失败。
- refs 1/1k/100k、roots 1/4 记录 lookup、stat/read 与结果顺序；stable generation 不重建索引。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止 last-result cache、fallback filesystem probe、第二份 root truth，或弱化 missing/ambiguous/link tests。

## 修复结果与回传

Open state: `single-inventory predecessor 的后继接线已恢复到本 failure 的受管范围：scan 发布逻辑 root/physical root/root-relative projection，sidecar preflight 仅发布已验证 compound binding，run 每 generation 构建 ephemeral MigrationResolverIndex，shared reference resolver 消费 ProjectSourceLookup，migration resolver 删除 roots/PathBuf/persisted_source_path_for_locator 与所有 per-reference FS fallback。静态检查已确认格式与空白通过，resolver 和 index 不含禁止 filesystem API；没有 Cargo green claim。Runtime04 的两次受管 focused attempt 均在目标测试执行前被 shared native-plugin loader 编译错误阻断，故当前仍 open；待该 lower shared compile owner 修复后，以新的 source snapshot/FIFO reservation 运行 declared resolver-index gate 和 origin scale/upward gate。`

## 2026-07-27 recovery status

- 原始 indexed-resolver source/test scope 已恢复并完成静态审计：`git diff --check` 与 exact Rust source 的 `rustfmt --check` 通过；`MigrationResolverIndex::build` 在 migration run 中只有一个 generation-owned 调用点；resolver/index 没有 `std::fs`、`read_to_string`、`read_dir`、`metadata`、`canonicalize` 或 `persisted_source_path_for_locator` 的每-reference fallback。索引保留 `PathBuf` 作为 generation projection 数据，不是 I/O。
- 当前 API 闭包是有意的：`migration::run`、`resolver` 与 `document` 分别消费 `MigrationResolverIndex` 与 `ProjectDocumentArtifact`。因此不得把 predecessor single-inventory 的历史 five-path manifest 单独提交为一个无法构建的 patch，也不得把本 lifecycle 的实现降级为 compatibility shim。
- 本次受管 snapshot create 在 64.5 秒后 CLI timeout，未返回 snapshot ID、reservation 或 Cargo job；这不是目标测试 RED，也不改变此前 native-plugin loader 编译阻塞的归属。状态保持 `open`，只在取得新的 source-bound snapshot 后申请声明的 focused gate。
