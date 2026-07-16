---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: virtual-geometry-runtime-support-compute-workload-drift
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_plugins/13
related_code:
  - zircon_runtime/tests/support/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
tests:
  - cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked
  - cargo test -p zircon_runtime --lib graphics::tests::plugin_feature_compile::gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph --locked
---

# Plugins13: Virtual Geometry Runtime support fixture 缺少 compute workload

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Runtime04 project-TOML consumer 已越过原 10 个 E0599，Frameworks05 Text library gate `9af67024670242beaac743a5c7dde856` 也已通过；随后 Windows 受管 focused job `1e7cdd7825024a08b236b2edd07c67b9` 真正运行 7 个 VG integration tests，结果 `0 passed / 3 failed / 4 ignored`。三个运行用例都在 pipeline compile 处被同一 descriptor drift 阻断：`virtual_geometry` 的 `virtual-geometry-node-cluster-cull` 声明 `AsyncCompute`，但 Runtime support fixture 没有 `RenderGraphComputeWorkload`。 |
| `IN PROGRESS / 实现完成，fresh 业务门待运行` | 2026-07-15 | support fixture 已按生产合同补齐命名化 compute workload，并增加真实 Rust descriptor 回归；受管 job `b8a1305560e4404f9f0fd5b459774d74` 在进入 VG test binary 前被 Frameworks05 当前 Text consumer 编译漂移与 Plugins05 `ControlPropRef` exhaustive validation 漂移阻断，exit 101。两者已有各自编号 failure；本记录不把外部编译失败误判为 Plugins13 失败，也不在 3 个非 ignored 用例实际通过前回传 fixed。 |

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

- 在 `zircon_runtime/tests/support/mod.rs` 的 Virtual Geometry fixture 中补齐明确、命名化的 compute workload，语义必须与真实插件当前合同一致：pipeline label `zircon-virtual-geometry-node-cluster-cull`、workgroup `[64, 1, 1]`、fixed dispatch `[1, 1, 1]`。
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
