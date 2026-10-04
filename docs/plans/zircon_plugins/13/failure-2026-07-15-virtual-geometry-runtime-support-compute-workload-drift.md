---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: virtual-geometry-runtime-support-compute-workload-drift
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/tests/support/mod.rs
  - zircon_runtime/tests/virtual_geometry_support_descriptor_contract.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
tests:
  - cargo test -p zircon_runtime --test virtual_geometry_support_descriptor_contract --locked
  - cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked
  - cargo test -p zircon_runtime --lib graphics::tests::plugin_feature_compile::gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph --locked
---

# Plugins13: Virtual Geometry Runtime support fixture 缺少 compute workload

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Runtime04 project-TOML consumer 已越过原 10 个 E0599，Frameworks05 Text library gate `9af67024670242beaac743a5c7dde856` 也已通过；随后 Windows 受管 focused job `1e7cdd7825024a08b236b2edd07c67b9` 真正运行 7 个 VG integration tests，结果 `0 passed / 3 failed / 4 ignored`。三个运行用例都在 pipeline compile 处被同一 descriptor drift 阻断：`virtual_geometry` 的 `virtual-geometry-node-cluster-cull` 声明 `AsyncCompute`，但 Runtime support fixture 没有 `RenderGraphComputeWorkload`。 |
| `IN PROGRESS / 实现完成，fresh 业务门待运行` | 2026-07-15 | support fixture 已按生产合同补齐命名化 compute workload，并增加真实 Rust descriptor 回归；受管 job `b8a1305560e4404f9f0fd5b459774d74` 在进入 VG test binary 前被 Frameworks05 当前 Text consumer 编译漂移与 Plugins05 `ControlPropRef` exhaustive validation 漂移阻断，exit 101。两者已有各自编号 failure；本记录不把外部编译失败误判为 Plugins13 失败，也不在 3 个非 ignored 用例实际通过前回传 fixed。 |
| `IN PROGRESS / current producer contract synchronized` | 2026-09-20 | `failure-roll-01a084c8-plugins13-vg-support-r2` 重新核对生产 descriptor 与 GPU executor：两者当前均以 `[64, 1, 1]` dispatch 执行 node/cluster cull，原 support/contract 仍为 `[1, 1, 1]`。先将 regression 期望切到当前 producer contract，source-bound RED 精确显示 fixture `1` / expected `64`；随后只把 support 常量同步到 `64`，四方 source contract（fixture/test/producer/executor）GREEN，focused rustfmt 与 scoped diff-check GREEN。受管 Rust 原始门、独立复审、return 与 closeout 仍 pending。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Virtual Geometry debug snapshot project-TOML consumer failure 的上行 focused 验证
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：RenderGraph 编译器的 AsyncCompute workload 强校验正确；真实 Virtual Geometry 插件描述符也已经按 Plugins13 M5/T1 合同声明 pipeline label、workgroup size 和 dispatch groups。只有根级 integration support fixture 仍复制旧 descriptor 形状，因此不得归咎 Runtime04 资产序列化或降低 RenderGraph 校验。

## 失败现象与复现证据

2026-07-15 Windows 默认兼容池运行：

```powershell
cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked
```

协调器 job `1e7cdd7825024a08b236b2edd07c67b9` 在 `D:\cargo-targets\zircon-engine\pool\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025` 完成 acquire/start/finish/release，退出 101。编译成功后测试实际执行，三个失败为：

- `render_framework_uses_virtual_geometry_provider_for_missing_authored_extract`；
- `render_framework_exposes_virtual_geometry_debug_snapshot_for_effective_visible_clusters`；
- `render_framework_exposes_node_and_cluster_cull_page_request_ids_in_debug_snapshot`。

共同错误为：

```text
feature descriptor `virtual_geometry` pass `virtual-geometry-node-cluster-cull` declares `AsyncCompute` queue but no compute workload
```

对照当前生产 owner：`zircon_plugins/virtual_geometry/runtime/src/lib.rs` 已给该 pass 声明 `zircon-virtual-geometry-node-cluster-cull`、`[64, 1, 1]` workgroup 与 `[1, 1, 1]` fixed dispatch；`zircon_runtime/tests/support/mod.rs` 的复制 fixture 只设置 executor/read/write，缺少 workload。

## 最低共享层根因

Plugins13 的 AsyncCompute workload rollout 只覆盖真实插件 descriptor 和 root lib fixture，没有覆盖 Runtime integration support 中独立构造的 Virtual Geometry descriptor。RenderGraph compiler 在 2026-06 已要求所有 `AsyncCompute` pass 必须携带 workload；support fixture 仍停留在旧形状，使 Runtime04 focused test 在业务断言前失败。

## 架构修复验收

- 在 `zircon_runtime/tests/support/mod.rs` 的 Virtual Geometry fixture 中补齐明确、命名化的 compute workload，语义必须与真实插件当前合同一致：pipeline label `zircon-virtual-geometry-node-cluster-cull`、workgroup `[64, 1, 1]`；历史 producer 的 fixed dispatch 为 `[1, 1, 1]`，2026-09-20 current producer 与 GPU executor 已共同前移为 `[64, 1, 1]`，fixture 和回归必须跟随同一现行值，不得保留双重合同。
- 增加或扩展回归守卫，确保 root integration support fixture 与生产 Virtual Geometry descriptor 的 AsyncCompute workload 形状不再漂移；不得只让三个当前用例绕过 compile。
- 保持 `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs` 的 no-workload hard error，不新增默认 workload、兼容 fallback 或 queue 降级。
- fresh 受管重跑 `virtual_geometry_debug_snapshot_contract`，三个非 ignored 用例必须执行并通过；随后重跑现有 `plugin_feature_compile` 精确门确认生产 descriptor 不回退。
- 修复完成后按本 failure lifecycle key 回传到 Runtime04 origin child；在 focused test 通过前 Runtime04 的 project-TOML failure 不得标记 fixed。

## 禁止临时方案

- 不得把 `AsyncCompute` 改为 `Graphics` 规避 workload 合同。
- 不得在 compiler 中自动注入 `[1, 1, 1]` 默认 workload。
- 不得忽略、删除或改弱三个 Virtual Geometry debug snapshot 断言。
- 不得恢复旧 RenderGraph descriptor API 或引入兼容 alias/re-export。

## 修复结果与回传

2026-07-15 当前实现已完成：`zircon_runtime/tests/support/mod.rs` 的 node-cluster-cull pass
使用与生产插件一致的 pipeline label `zircon-virtual-geometry-node-cluster-cull`、workgroup
`[64, 1, 1]` 与 fixed dispatch `[1, 1, 1]`；
`zircon_runtime/tests/virtual_geometry_support_descriptor_contract.rs` 直接构造 support descriptor 并
断言这三个字段。旧的 Python 正则草案已删除，避免注释/死代码产生假阳性。

fresh Windows 受管 job `b8a1305560e4404f9f0fd5b459774d74` 执行原 focused 命令，exit 101，
但未进入 VG test binary；当前错误属于已有
`Frameworks05/failure-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md` 和
`Plugins05/failure-2026-07-15-control-prop-ref-validation-runtime-gate.md`。另一次 acquire 的临时目录
并发误判已交给 Coordinator01：
`failure-2026-07-15-live-ephemeral-target-misclassified-unmanaged.md`。

Open state: `实现完成 / 验收阻塞`; no pass is claimed. 待两个外部 compile owner 收敛后，fresh
重跑 Rust descriptor contract、`virtual_geometry_debug_snapshot_contract` 与现有
`plugin_feature_compile`；三门通过后才按 lifecycle key 回传 Runtime04。

## 2026-09-19 生产 descriptor 漂移核对

本轮 owner 接管时，`zircon_plugins/virtual_geometry/runtime/src/lib.rs` 已存在未提交的
foreign working-tree diff（文件 mtime `2026-09-07T20:54:45+08:00`，相对 `HEAD`
`6091905cb` 的变更包括 graph buffer schema，并把 node-cluster-cull fixed dispatch 从
`[1, 1, 1]` 改为 `[64, 1, 1]`）。同一 working tree 的 GPU executor
`zircon_plugins/virtual_geometry/runtime/src/render_pass_executors/gpu.rs` 也使用
`NODE_CULL_DISPATCH_GROUPS = 64`，但该文件不在本 lifecycle 的可证明 owner scope 内。

这与本 failure 已记录、且 support fixture/descriptor contract 当前仍验证的
`[1, 1, 1]` 生产合同冲突。为避免吸收或回退他人未提交修改，本轮不编辑该 production
diff、不把它作为已验收修复；其确切 working-tree 状态须由生产 descriptor/GPU executor
owner 先裁决（若新合同确认为 `[64, 1, 1]`，应由同一 owner 同步 support fixture、回归
断言与原始验收命令）。在该裁决和外部 Frameworks05/Plugins05 编译阻断解除前，failure
保持 `open / 验收阻塞`，不得生成 `fixed-*` 或 failure return。

## 2026-09-20 current producer contract 修复与受管门回执

- stable owner `failure-roll-01a084c8-plugins13-vg-support-r2` 已恢复并重新领取 failure、
  support fixture、descriptor regression 与生产 descriptor 四路径；编辑前哈希与既有 attribution
  完全一致。生产 descriptor 与 GPU executor 当前都使用 `[64, 1, 1]` dispatch，故 2026-09-19
  的合同分歧已经裁决为 support fixture 漂移，而不是回退生产端。
- TDD RED 先把 regression 的现行期望更新为 `[64, 1, 1]`，确定性 source-bound 对比得到
  `fixture_dispatch=[1,1,1] expected_dispatch=[64,1,1]`；随后仅把 support 常量同步到 `64`。
  同一检查复核 fixture、regression、producer 与 executor 四方均为 `[64,1,1]`，focused
  `rustfmt --check --config skip_children=true` 和 scoped `git diff --check` 均通过。未修改生产
  descriptor 或 GPU executor。
- coordinator 新票据策略拒绝非 Cargo 命令，因此 source check 没有伪装成 validation ticket。
  第一份 Cargo admission request `184cc9522c094b58b0b7d8f84a181e73` 在入队前因调用方携带
  coordinator-owned `--jobs` 被拒绝；按其 repair condition 移除该参数后，request
  `460846703e49485b801f6ee450dc9b1c` 又在入队前被
  `validation_ticket_external_worktree_dirty` 拒绝，唯一 blocker 为 `E:\Git\zr_vm` 的外部
  owner 未提交改动。两次请求均未创建 ticket、job 或 Cargo process，不构成 GREEN/RED。
- 当前状态为 `open / waiting_validation`。外部 owner 将 `zr_vm` 收敛为可固定 clean revision 后，
  以当前四路径哈希重提同一双 integration-target Cargo 批次，再执行生产
  `plugin_feature_compile` 精确门；其后仍需独立 C0/I0/M0 review、canonical return 与 closeout。

## 2026-09-20 independent review receipt and upward-contract handoff

- 对本 lifecycle 实际编辑的 `zircon_runtime/tests/support/mod.rs` 与
  `zircon_runtime/tests/virtual_geometry_support_descriptor_contract.rs` 做只读独立审查，
  结论为 `Critical=0, Important=0, Moderate=0`。两文件各仅有一行 dispatch 值同步，
  pipeline label、`[64,1,1]` workgroup、`QueueLane::AsyncCompute` 与 compiler hard-error
  语义均未被削弱；focused rustfmt、scoped diff-check，以及 fixture/test/producer/GPU
  四方 source comparison 均通过。审查没有申请 Cargo ticket，也不宣称 Rust 动态门通过。
- 审查发现一个必须向上交接的 `Important=1` 缺口（不属于本 Session 的可编辑范围）：
  `zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs:163-167` 和
  `zircon_runtime/src/graphics/tests/plugin_feature_compile.rs:228-240` 仍构造/断言
  `[1,1,1]`。这会让本记录要求的 `plugin_feature_compile` 门继续接受过时的 synthetic
  descriptor，同时生产 descriptor/GPU executor 已执行 `[64,1,1]`，形成第二份矛盾合同。
  两个文件均保留 foreign 修改；`plugin_feature_compile.rs` 的现行 coordinator 归属是
  已归档 `render01-compiled-pipeline-metadata-closeout-20260717`，另一个文件没有可证明的
  当前 lease/attribution。本 lifecycle 不吸收、回退或直接编辑它们，改由其 owner 通过关联
  failure/handoff 同步后再重跑 upward gate。
- Review snapshot Git object IDs（仅作证据，不代表当前 owner 转移）为：
  `support/mod.rs=b43f385adf5cd90f153d5be014d7c838700b2e55`,
  `virtual_geometry_support_descriptor_contract.rs=831842d3b02c6961fbbb911c27c3d0638b7b63c6`,
  `producer lib.rs=9172eccb3e49eef6203d49ad58418091f06278e2`,
  `gpu.rs=4c6c9306326db11cecc187196fc0d41a0de78c46`,
  `plugin_render_feature_fixtures.rs=7b0c04b412703acbf8d543a569a93649021485c7`,
  `plugin_feature_compile.rs=7e3c9eedb327ac041ebe656926e64b76dbc2a2f3`.
- Status remains `open / waiting_validation`: external `E:\Git\zr_vm` still blocks managed Cargo;
  the upward stale-fixture handoff, Cargo gates, canonical return, fixed artifact, and closeout
  remain pending. The no-dual-contract acceptance rule is preserved.
