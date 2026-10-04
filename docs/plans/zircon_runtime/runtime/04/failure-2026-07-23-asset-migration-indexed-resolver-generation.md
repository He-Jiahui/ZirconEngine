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
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/scan.rs
  - zircon_runtime/src/asset/migration/sidecar.rs
  - zircon_runtime/src/asset/migration/resolver.rs
  - zircon_runtime/src/asset/migration/resolver_index.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/mod.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/resolver_index.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
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

Resolving state: `single-inventory predecessor 与 indexed-resolver 后继已在 current source 完成接线。scan generation 同时发布 canonical physical identity、logical/root-relative projection 与被拒绝的 link/reparse 路径；sidecar counterpart 与 mint 决策只消费该 generation，不再用 exists/read fallback 把外部 link 当作 authority。physical identity 与 rejected-path 查询都基于排序索引执行 O(logN) lookup。run 每 generation 只构建一个 ephemeral MigrationResolverIndex，resolver/index 的 per-reference filesystem API 计数为 0。测试覆盖 100k distinct references、1/4 roots、forward/reverse 结果顺序、compound/direct/ambiguous/missing/duplicate 优先级及 current/retired linked-sidecar 拒绝。独立二次复审 Critical/Important/Minor=0/0/0；rustfmt、diff-check 与源码 guard 通过。Snapshot 1493 的 Rust manifest hash=a587940bc1d0f00836e5b2977329773d2b4ce1ed8371e8442b87e9905fef0d55；Windows Rust 1.94.1 package-check ticket 5cc91e28c6f542e4b749f4c856054e4a 与 focused ticket 4cfb001e3f134b35bd1988efbdec0400 已受理。两者尚无 terminal receipt，故不宣称 Cargo GREEN、fixed 或 upward acceptance。`

## 2026-07-27 recovery status

- 原始 indexed-resolver source/test scope 已恢复并完成静态审计：`git diff --check` 与 exact Rust source 的 `rustfmt --check` 通过；`MigrationResolverIndex::build` 在 migration run 中只有一个 generation-owned 调用点；resolver/index 没有 `std::fs`、`read_to_string`、`read_dir`、`metadata`、`canonicalize` 或 `persisted_source_path_for_locator` 的每-reference fallback。索引保留 `PathBuf` 作为 generation projection 数据，不是 I/O。
- 当前 API 闭包是有意的：`migration::run`、`resolver` 与 `document` 分别消费 `MigrationResolverIndex` 与 `ProjectDocumentArtifact`。因此不得把 predecessor single-inventory 的历史 five-path manifest 单独提交为一个无法构建的 patch，也不得把本 lifecycle 的实现降级为 compatibility shim。
- 本次受管 snapshot create 在 64.5 秒后 CLI timeout，未返回 snapshot ID、reservation 或 Cargo job；这不是目标测试 RED，也不改变此前 native-plugin loader 编译阻塞的归属。状态保持 `open`，只在取得新的 source-bound snapshot 后申请声明的 focused gate。

## 2026-08-29 architecture and algorithm re-review

- 重新逐文件审查 `run.rs`、`scan.rs`、`sidecar.rs`、`resolver.rs`、
  `resolver_index.rs` 与 100K focused matrix。正常执行路径只在 recovery 后需要刷新时
  重建一次 inventory；随后每个 generation 只构建一个 `MigrationResolverIndex`，resolver
  本身不持有 filesystem capability。forward/reverse 查询继续由 `HashMap` 提供平均
  `O(1)`，排序索引只用于 inventory physical/rejected-path 查询，复杂度为 `O(logN)`。
- 对照 `dev/UnrealEngine` 的 `FAssetRegistryState::CachedAssets`、
  `UAssetManager::AssetPathMap` 与显式 `ScanPathsSynchronous` 边界，Zircon 当前的
  discovery -> immutable generation index -> pure query 分层一致。GUID 仍是 authority，
  path hint 仅用于 repair；physical alias 合并与 distinct logical-root ambiguity 没有被
  cache 或 fallback 弱化。
- `resolver_projections` 的 `RelPath::parse(...).ok()` 在已 canonicalize 的目录 root 与
  `strip_prefix` 正常文件投影下不会形成 steady-state missing 分支；asset root 被配置为
  ordinary file 属于独立的 project input validation 边界，不以 resolver cache 修补。
  `prepare_roots` 现已在唯一 inventory admission 处以 `InvalidInput` 拒绝非目录 root，
  focused inventory 与 commandlet 回归同时固定无写入副作用和
  `AssetMigrationError::Scan` 投影。本轮没有发现可复现的 resolver 算法缺陷，因此不做
  无 profile 依据的生产算法改写；两份 Rust source 已通过 pinned Rust 1.94.1 formatter
  与 scoped diff gate，Cargo focused 仍待受管 lane。
- 2026-08-29 current-source package request
  `d284c06bcf06402085f70582b41c16c3` 在 Cargo admission 前以
  `command_post_timeout` 退出，未产生 compiler/test/performance receipt。状态继续保持
  `open`；不重复提交、不轮询，并继续执行不依赖该 lane 的源码与架构收敛。

## 2026-09-09 production-counter acceptance repair

Managed Windows job `9aa5c29f3f6640a1808d55c69973254f` on immutable input
`runtime04-migration-fixtures-3329-20260909`, manifest
`3d03d4e4fdb130c29e772a3fe2d08d29459a5e3737f0a24fa41ef932b42fe644`,
passed the public migration batch: 71 passed, 0 failed, 1 ignored. All eleven
`project_commandlet::resolver_index` tests executed, including the 100k-entry,
1/4-root query matrix, compound bindings, missing/ambiguous cases and duplicate
registry precedence. The ignored case belongs to the separate filesystem
scale-acceptance lifecycle.

Review of the passing scale test found that its printed lookup count was
computed from `reference_count * 2`; it did not assert the production counter
for each workload. The fixture now samples `MigrationResolverIndex::lookup_count`
before and after the actual forward/reverse queries, asserts the observed
delta against the required count, and prints that observed value. The same
index remains alive across all three cardinalities for each root count; all
existing order and full-result assertions remain. Production resolver/index
code is unchanged. The fixture preimage hash is
`c70c363dd02952dfcdfafbf582db7e474baf49ab8a1228c94998233859fba2ce`.

The strengthened test requires a new managed run. The earlier 71-test receipt
does not validate these new assertions, provide uncaptured timing rows, or
replace the required filesystem/whole-commandlet performance evidence.
This lifecycle remains open through complete validation, independent review
and coordinator closeout.

The strengthened resolver batch completed as managed Windows job
`131f0f22cde34897be53e25343f1b853`, exit 0: 11 passed, 0 failed, 0 ignored.
Input `runtime04-migration-metrics-red-3340-20260909` has manifest
`8c631afa4cec4191b00dceb04b454857011f5954f1c2ac74aa3b1cbb9ab304ae`;
the unrelated document-counter RED cases were outside this resolver filter.
The current resolver test hash remains
`934b3f5fad3f52cc9d7cbe5b6a27c244e17461cff32953595f974ac69e99f61c`
from snapshot `3336`, identical to the tested bytes.

The actual 1/1k/100k forward/reverse workloads for 1/4 roots now pass their
production `lookup_count` assertions, as do all retained ambiguity, compound,
registry-precedence and path-result checks. Exact command and receipt are in
`results/runtime04-resolver-counters-3336-r1.json` and its sibling log.
The successful harness captures printed output, so this supplies executed
counter assertions rather than per-cardinality timing samples. Filesystem
observations, whole-commandlet acceptance, review and closeout remain open.
