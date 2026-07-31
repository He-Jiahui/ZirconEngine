---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: asset-migration-scale-acceptance-matrix
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/migration/report.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/scale_acceptance.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::scale_acceptance --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::scale_acceptance::managed_scale_sweep_executes_declared_cardinalities --locked --jobs 1 -- --ignored --exact --nocapture --test-threads=1
---

# Runtime04：asset migration规模验收矩阵缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration 性能审查 PERF-MVP-511；经批准从 single-inventory lifecycle 拆分
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：Runtime04 必须提供同一 migration production contract 的规模/计数验收 owner；本 lifecycle 不与 production repair scope 重叠。

## 失败现象与复现证据

focused source-boundary suite 已证明单一递归 owner 与小型分类 fixture，但尚无 1/1k/100k files/dirs/refs、1/4 roots、dry-run/apply/unchanged/1% change 的统一计数证据，不能从小 fixture 推断 PERF-MVP-511 的规模合同。

## 最低共享层根因

migration 缺少唯一、production-backed 的规模 instrumentation 与 acceptance matrix owner；小型 source-boundary contract 无法证明大规模复杂度与稳定 generation 行为。

## 架构修复验收

- 记录 entry visits、directory reads/sorts、resolver filesystem probes、document reads/parses、完整 Value clone 与输出 bytes/issues/order。
- 计数必须由 production run 聚合到 typed `AssetMigrationMetrics`/report；不得通过全局 atomic、测试重扫或源码字符串猜测动态复杂度。
- entry visits≤1/run 或 generation、directory read/sort≤1、per-ref fs=0、document parse≤1、full Value clone=0。
- dry-run/apply/unchanged/1% change 的 deterministic bytes、issue/order、rollback/recovery 与 idempotence 不变。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 测试必须使用 production instrumentation 或公开诊断，不得复制 scanner/resolver/parser 真相、伪造计数或缩小计划阈值。

## 修复结果与回传

Open state: `Runtime04 scale instrumentation r2 已恢复 production 接线：run 从唯一 inventory 聚合 entry/directory counters，document migration 回传 reference visits，report 记录 authoring document reads/parses 与 pending output bytes。scale_acceptance 已由 project_commandlet test root 实际挂载：常规合同覆盖 dry-run/apply/unchanged/1% change 与 1/4 roots；受管 ignored scale lane 实际生成 files/refs/directories 的 1/1k/100k workload 并读取 production metrics，不以 216 维度声明替代执行。受管 source leases 覆盖 report/document/run/scan/sidecar/test root/scale test，rustfmt 与 scoped diff check 已通过。尚无 current-source managed scale terminal：最近两次 Runtime04 focused GREEN 在执行目标前被 shared native-plugin loader 编译错误阻断（native_plugin_load_report/tests.rs 缺少 NativePluginLoadProjection，discover/authority.rs 的 NativePluginLoadReport literal 缺少私有 projection）。该阻断不属于 Runtime04 owned paths；native-plugin owner 修复后，必须以新的 FIFO reservation 运行本记录的 exact scale command。旧 snapshot 985 的 source_boundary 6/6 green 仅属于已接受的 single-inventory fixed return，不可充当 scale matrix green。因此本 failure 保持 open`。
