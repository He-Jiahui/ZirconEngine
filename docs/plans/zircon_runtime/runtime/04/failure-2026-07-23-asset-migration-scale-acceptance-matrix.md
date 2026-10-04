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
  - zircon_runtime/src/asset/migration/document.rs
  - zircon_runtime/src/asset/migration/document/metrics_tests.rs
  - zircon_runtime/src/asset/migration/run.rs
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

## 2026-09-09 managed execution and remaining instrumentation gaps

The ordinary public migration batch now passes on immutable input
`runtime04-migration-fixtures-3329-20260909`, manifest
`3d03d4e4fdb130c29e772a3fe2d08d29459a5e3737f0a24fa41ef932b42fe644`:
job `9aa5c29f3f6640a1808d55c69973254f`, 71 passed, 0 failed, 1 ignored.
All five non-ignored scale-acceptance cases executed. The ignored workload was
submitted separately as `runtime04-migration-scale-3329-r1`; it must finish
before any large-scale pass can be claimed.

The source audit still finds acceptance gaps. `AssetMigrationMetrics` has no
resolver filesystem-probe or whole-document Value-clone observation. The
current run owner increments both document reads and parses before
`migrate_document` attempts UTF-8 input loading, so a read failure would be
counted as a parse. Reference visits accumulated before a later document error
are discarded with the error result. These findings require production-owner
counter corrections and negative-path regression; they are not dynamic
failures observed in the running large workload.

The ignored workload currently runs DryRun for each files/references/directories
cardinality, while the four-phase case uses 100 documents and the four-root
case uses a small fixture. Passing these cases alone cannot establish the full
1/1k/100k by phase/root acceptance matrix. No zero-value instrumentation,
static source guard or declared dimension count is treated as its replacement.

The ignored workload completed as managed Windows job
`c3b4abe0d58b487b87e488b0f53665c2`: 1 passed, 0 failed, 0 ignored, exit 0,
on the same immutable `3329` input. The actual test was
`asset::tests::migration::project_commandlet::scale_acceptance::managed_scale_sweep_executes_declared_cardinalities`;
its files, references and directories loops each ran 1/1k/100k. The receipt
records 1153.843 seconds in test execution. Exact command, source digest and
terminal test summary are in `results/runtime04-migration-scale-3329-r1.json`
and its sibling log. The harness captured successful test stdout, so the
per-cardinality printed samples are not available in this log; no fabricated
sample values or peak-RSS measurements are supplied. The full phase/root
matrix and counter gaps described above remain open.

Snapshot `3340` adds four real-input metric regressions under the document
owner, before changing production counters: invalid UTF-8 must count one read
and no parse; malformed TOML must count the attempted parse; a rejected retired
reference must count its resolver visit; a later current-reference error must
retain the earlier visit and the failing one. Each uses the public migration
entry point and retains failure classification and no-write assertions. The
regressions are awaiting managed execution; no RED or GREEN is claimed yet.

The four regressions subsequently executed on immutable input
`runtime04-migration-metrics-red-3340-20260909`, manifest
`8c631afa4cec4191b00dceb04b454857011f5954f1c2ac74aa3b1cbb9ab304ae`.
Managed Windows job `9dc9e52d9a52431a9f5fbf8876eba9dc` passed prerequisite
compilation and reported 1 passed, 3 failed, 0 ignored (Cargo test exit 101).
The malformed-TOML control passed. The old counters reported one parse instead
of zero for invalid UTF-8, and zero visits instead of one or two for the
rejected-reference cases. Exact command, receipt and assertions remain in
`results/runtime04-migration-metrics-red-3340-r1.json` and its sibling log.

Snapshot `3342` repairs the private document/run boundary. `migrate_document`
updates the run's existing typed metrics at each read, parse and reference
resolution attempt, so observations survive later `Err` returns. A retired
reference is counted before invoking the resolver. The private success-only
`DocumentMigrationResult` wrapper is removed; output publication and issue
classification retain their existing path. Public getters document failed
attempts. The unchanged four tests and three modified production files are
frozen at:

- `document.rs`: `46e360aeaa07d2bbb32ca8c1f2e3cc9a13ae9c197f8b413eaec1ccc952f4d64e`.
- `run.rs`: `7c351f0e0bd39fd133ab18329b62722a52979af5762236b28ffc6219d5ec9563`.
- `report.rs`: `780f26eb104a1e9d164f37a901dda711cff62ce3e40297b725bc8b5e72b7b402`.
- `document/metrics_tests.rs`: `eb9966b49840323a1f9fa990ee816245ad30854d08de36c6891f211b09d2d138`.

Scoped rustfmt and diff checks pass; managed post-fix execution is pending.
No resolver-filesystem or full-Value-clone counter was synthesized. The full
phase/root/cardinality matrix, independent review, return and closeout remain
open. The prior large DryRun pass applies to its explicitly recorded input.

Post-fix managed evidence is now available on derived input
`runtime04-migration-metrics-3345-20260909`, manifest
`65cc43634bb13e0bc657790a58c09a6e6950ff252faf0fb70d407c1f62f7c1c8`.
The exact four document metric regressions passed 4/4 in job
`38e289ca54a64bb1be2df6d8648a2807`, and the original migration commandlet
passed 70/70 non-ignored tests with one ignored in job
`d3635796bbce417d9288d0c26d52f51c`. Results are retained as
`runtime04-migration-metrics-3345-r1.json` and
`runtime04-migration-commandlet-3345-r1.json` with sibling logs.

The lower counter repair therefore has dynamic proof for invalid-read,
invalid-parse, rejected-retired-reference and later-current-reference paths,
plus the original public migration suite. The full phase/root/cardinality
matrix and the ignored 1/1k/100k workload have not been rerun against the new
source, and resolver-filesystem/full-document-clone observations are still
absent. This failure remains open for those gates, independent review, return
and closeout.
