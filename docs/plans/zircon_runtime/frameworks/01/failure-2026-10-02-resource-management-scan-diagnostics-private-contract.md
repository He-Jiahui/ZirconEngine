---
handoff_kind: failure
status: open
created_at: 2026-10-02
summary_slug: resource-management-scan-diagnostics-private-contract
origin_plan: docs/plans/astra/features/plugins/02-native-trust-admission.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/astra/features/plugins/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_resource/src/management_generation.rs
  - zircon_runtime/crates/zr_resource/src/lib.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_runtime/crates/zr_resource/src/management_generation/tests/projection.rs
  - zircon_runtime/crates/zr_resource/tests/management_scan_diagnostics.rs
  - docs/crates/zircon_runtime/core/resource.md
tests:
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zr_resource --all-targets --locked --jobs 1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zr_resource --all-targets --features test-support,profiling --locked --jobs 1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zr_resource --test management_scan_diagnostics --features test-support,profiling --locked --jobs 1 -- --test-threads=1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zr_resource --lib --features test-support,profiling --locked --jobs 1 management_generation::tests::projection:: -- --test-threads=1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/asset-facade-rust1.94.1/target -- +1.94.1 test -p zircon_runtime --locked --jobs 1 --message-format short --color never --features test-support,profiling --lib plugin::package_service::admission::policy_index::tests:: -- --test-threads=1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zircon_runtime --lib --features test-support,profiling --locked --jobs 1 asset::tests::facade:: -- --test-threads=1
  - python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zircon_runtime --lib --features test-support,profiling-tracy --locked --jobs 1
---

# Frameworks01：Resource management scan 跨 crate 诊断契约不可见

## 来源执行者

- 来源计划：`docs/plans/astra/features/plugins/02-native-trust-admission.md`
- 来源执行切片：Astra Plugins02 native trust admission 的独立 Windows `policy_index::tests::` 九测试批次；该批次同时编译真实 Runtime Asset consumer。
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：最低共享根因是 Frameworks01 拥有的 `zr_resource` 物理分拆后，公开 `ResourceManagementScan` 的查询工作量诊断仍返回 crate-private 方法、DTO 与字段；上层 Asset profiler adapter 不能在自己的 crate 调用。不能在 native trust 或 Asset caller 复制计数权威。

相关计划：[Astra Plugins02](../../../astra/features/plugins/02-native-trust-admission.md)、[Frameworks01](../01-runtime-crate-decomposition.md)。来源原项 [penpot-editor-native-trust-admission-compile](../../../astra/features/plugins/02/failure-2026-10-01-penpot-editor-native-trust-admission-compile.md) 的稳定 lifecycle 与 native/product 门保留，不由本项替代。

## 失败现象与复现证据

2026-10-02 Runtime r2 原命令通过 `tools.dev.local_cargo` 在 Windows 独立执行。完整命令如下，保留回执中的 Python 3.14 可执行文件与参数原序列：

```powershell
& 'C:\Users\HeJiahui\AppData\Local\Python\pythoncore-3.14-64\python.exe' -B -m tools.dev.local_cargo --target-dir D:\cargo-targets\zircon-local\windows\failure-roll-01a0df1a\asset-facade-rust1.94.1\target -- +1.94.1 test -p zircon_runtime --locked --jobs 1 --message-format short --color never --features test-support,profiling --lib plugin::package_service::admission::policy_index::tests:: -- --test-threads=1
```

- 执行窗口：`2026-10-02T18:34:26.761795+00:00` 至 `2026-10-02T19:00:40.873684+00:00`；Cargo exit `101`，实际执行 `0` tests，原批次期望 `9`。不是动态通过或产品验收。
- 精确诊断：`zircon_runtime\src\asset\pipeline\manager\project_asset_manager\management.rs:49:24: error[E0624]: method `profile_metrics` is private: private method`（原 `cargo.log` 第 421 行）。接收者是 `ResourceManagementScan`，不是 `ResourceManager`。
- [原回执](../../../../../.codex/tmp/failure-roll-20261002-current-runtime-local-batches-r2/policy/receipt.json)，SHA-256 `ef72e925b8a6a2b618de8c6159f850b840cc9f44c93611c835e54f7fae10a19e`；[原日志](../../../../../.codex/tmp/failure-roll-20261002-current-runtime-local-batches-r2/policy/cargo.log)，SHA-256 `6ece077322573b0cc68e67598edae3914762760a1a1787674859d23dc6ae5ca0`。
- [原 source-before](../../../../../.codex/tmp/failure-roll-20261002-current-runtime-local-batches-r2/policy/source-before.json) 为 `10,569` 输入，SHA-256 `6f6fa4daac3dc55701a85b2a3481a3c9557e611b4676b7e3cb27787abf115d76`。回执记录运行期间 `18` 个非 Resource 输入漂移，`sourceStableAtBoundaries=false`。该回执只证明实际编译诊断；不能作为任何稳定源码快照的通过证据。七路径修复范围不与这十八路径相交。
- 2026-10-02 对当前 `487` open artifacts 的完整文本与稳定 lifecycle 核对，未发现 `ResourceManagementScan`、`profile_metrics` 或本 slug 的同根因项。现有 [Asset04 management-generation 性能项](../../runtime/04/failure-2026-07-22-asset-management-generation-projection.md) 处理 compact generation/stable polling/performance；[Editor Runtime import 原项](failure-2026-09-06-editor-runtime-import-contract.md) 处理 graphics hard-cut consumer convergence。它们不是本项的替代 lifecycle。

## 最低共享层根因

`zr_resource::management_generation` 是 immutable ordered pages、scan cursor 与三个 query-local counters 的唯一生产者。旧 `ResourceManagementScan::profile_metrics()` 为 `pub(crate)`，返回同样 crate-private 的 `ResourceManagementScanProfileMetrics`，字段也不可跨 crate 读取。Runtime Asset 的 `record_completed_resource_management_scan()` 属于另一 crate，即使 scan 本身公开仍触发 E0624。

诊断值是这一条 scan 的已发生工作量，不是 generation-wide diagnostics，也不是 cache/publication identity。只扩大方法可见性不能修复私有返回形状；把 recorder 放入 Resource 会逆转 foundation 依赖；让 Asset 重扫或重算会引入第二权威。scan 必须实际耗尽 `next_row()` 后才计入尾部 rejected rows；现有生产 consumer 保留该行为。

Frameworks01 M1 规定 Resource implementation/内部行为测试迁入 `zr_resource`、Runtime 只做 curated projection，并禁止 Resource 反向依赖 Runtime/Diagnostics。当前修复公开只读、私有字段的 `ResourceManagementScanDiagnostics` 和三个 getter；不能把未批准的 assembly 符号一起扩大为产品 API。

## 架构修复验收

1. 最低共享层提供唯一只读 scan-local diagnostics，捕获不前移 cursor、不取 manager 锁、不 clone rows、不分配、不改 generation identity；默认构建不要求 profiling DTO。保留 clone/repeat/exhaustion 的原不变量。
2. `zr_resource` 和 Runtime curated resource projection 只导出该已审 diagnostics 类型；Asset 保持现有 capture/frame gating 与 profiler counter 名称，仅消费同一查询工作量 getter。删除旧 `profile_metrics()` 契约，不保留 alias/shim。
3. 外部 integration test 必须验证 fresh counters、leading/trailing rejected rows、retained diagnostics、重复/耗尽读取、重新扫描及 unchanged generation Arc；原六个 projection regressions 必须实际运行。
4. 下表是待执行的最小完整 Windows 批次，全部保留 Cargo `--locked`，physical target/caches 只允许 drive-root D/E/F `cargo-targets`。精确过滤须取得实际目标数量；编译命令和零测试不能冒充动态测试。

| Gate | Expected | Command |
| --- | --- | --- |
| default Resource compile | 编译门；不声明测试执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zr_resource --all-targets --locked --jobs 1` |
| profiling Resource compile | 编译门；不声明测试执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zr_resource --all-targets --features test-support,profiling --locked --jobs 1` |
| public external integration | 1 tests，必须实际执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zr_resource --test management_scan_diagnostics --features test-support,profiling --locked --jobs 1 -- --test-threads=1` |
| Resource projection regressions | 6 tests，必须实际执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zr_resource --lib --features test-support,profiling --locked --jobs 1 management_generation::tests::projection:: -- --test-threads=1` |
| original policy batch | 9 tests，必须实际执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/asset-facade-rust1.94.1/target -- +1.94.1 test -p zircon_runtime --locked --jobs 1 --message-format short --color never --features test-support,profiling --lib plugin::package_service::admission::policy_index::tests:: -- --test-threads=1` |
| direct Asset facade consumer | 26 tests，必须实际执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 test -p zircon_runtime --lib --features test-support,profiling --locked --jobs 1 asset::tests::facade:: -- --test-threads=1` |
| Tracy consumer compile | 编译门；不声明测试执行 | `python -B -m tools.dev.local_cargo --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-01a0df1a/resource-scan-diagnostics-rust1.94.1/target -- +1.94.1 check -p zircon_runtime --lib --features test-support,profiling-tracy --locked --jobs 1` |

5. 下层、原九测试批次及二十六个直接 Asset facade tests 均需匹配新鲜源码/配置/命令回执。其他 owner 的编译阻断只建立独立关联并保留本项 open。Astra Plugins02 原 native DLL/admit-before-load/side-effect/hot-reload 及 Editor 产品门保持各自验收，不由 Resource smoke pass 关闭。
6. 用户已于 2026-10-02 授权“独立 Windows 验证、精确范围提交与去重通知，仍不 push”；[迁移授权](../../../../../.codex/tmp/failure-roll-20261002-workflow-migration-authorization.json) SHA-256 `09dc8a7e9c73b144bd88b6cdb5f2a67704e2ceb6806acf7d107ec0cab7ca48db`。遵循 [现行验证政策](../../../milestone-validation-policy.md) 与 [协调器退役](../../../../cli-and-tooling/coordinator-retirement.md)，不恢复旧服务或重放旧票据。独立本地回执不授予 Jenkins/milestone 验收。

## 禁止临时方案

- 不新增 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypass 或调用点特例。
- 不将 Resource recorder 反向接入 Runtime/Diagnostics；不以 Asset 全表重扫或 generation counters 替代 scan-local metrics。
- 不扩大所有内部 symbols/assembly 的公开范围，不把诊断数字当作 publication/cache identity。
- 不弱化计数、cursor、generation、trailing reject 或 direct consumer 测试；不以源码、静态审查、语法检查、旧快照通过或零测试回执充当动态验收。
- 不合并/关闭 Asset04 的原 performance lifecycle，不跳过原 policy/native/product 验收，不操作 foreign bytes/index/locks，不 push。

## 修复结果与回传

Open state: `源码修复已安装，独立 Windows 验证及正式回传待完成`；不声明通过或关闭。

- Root 于 `2026-10-02T20:10:39.495801+00:00` 按已审七路径精确安装。[安装回执](../../../../../.codex/tmp/failure-roll-20261002-resource-public-boundary-install/installed.json) SHA-256 `dd83c5e0dd0b3d3d82038495763b11994b612c099303406d30bc70fe2c6516a9`；[v2 manifest](../../../../../.codex/tmp/failure-roll-20261002-resource-profile-public-boundary-audit/manifest-v2.json) SHA-256 `e1fa3214247b67b3f68a050ccd2fde238beb81ea3a0ce89e71ce6415b9720963`；独立静态 v2 review SHA-256 `fec909e61b62765ae97ee19fbf01d31d5eaec83fd0098302bbdf24586368492f`，C/I/M `0/0/0`。这些是源码/审查证据，动态门仍待执行。
- Root 只采用 public diagnostics/test migration deltas，原有 foreign source/comment bytes 和归属在 preimages 中保留，不把整文件归因给本项。
- 精确原文与 postimage 血缘如下；`absent` 表示新 integration test 原先不存在，不能虚构 preimage。

| Path | Preserved preimage SHA-256 | Installed postimage SHA-256 |
| --- | --- | --- |
| `zircon_runtime/crates/zr_resource/src/management_generation.rs` | `3c4e4e9e63a62bf7c08547e41d3949ecfed02b35ab14b592ec75596ef807fd35` | `8dcb43932f0e6908eae79cdff673066b68337d166b7eeacdd448a9445a113fd3` |
| `zircon_runtime/crates/zr_resource/src/lib.rs` | `abe1823f1e5740ff62d3b8f0d64effce77a945bc3a9aeb9b43566a89fe4dfb18` | `0fa6dcc9d89ae25863d20c9ae900b04aaba4f5fd753cc515565bd8f86b105619` |
| `zircon_runtime/src/core/resource/mod.rs` | `1bfef5def9d021a7b1267a764fd98c83bef360f2d66f6f87a14fc670144c4a77` | `7d508d809746d6df135dc7cfabdd9aca7b2fff48e64a12a084f218839d0e27fd` |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs` | `9e59888ef51d260cc5560206d1d563b5d051ac1493109365785c55e812b0dd28` | `d0a9e6dd1e6b1881d2a08c33b484f727fdef46d4d98743b2961b4b9231c4cc26` |
| `zircon_runtime/crates/zr_resource/src/management_generation/tests/projection.rs` | `52ecf78e084d293919da61fbd95b9bd9c055637824d6f4a3a5ce450ad2031cd2` | `338bd32a03fe70f22bfa36ef4e899ae82458675c9d5c11fe69e0236f3a06d5f8` |
| `docs/crates/zircon_runtime/core/resource.md` | `150ae8c7c824658ca7216ec7002adb6c51f655de933af8de21dbb57b5d4f5466` | `476dec2006b1c45fd5507e4f94dd3a072191d5383e58e6383a603f76ba36fd31` |
| `zircon_runtime/crates/zr_resource/tests/management_scan_diagnostics.rs` | `absent` | `3d4ee4f41ae79895364e722edaeec8afa28e5a39b24e3a62c58e4c66c90a8d84` |

- 后续只复用源码、命令、配置和环境均匹配的实际通过回执。完成相关动态门及独立 C/I/M=0 审查后，按同一 lifecycle 回传唯一 canonical `fixed-*`；本记录当前仍为 `open`，没有真实 commit/通知回执。
