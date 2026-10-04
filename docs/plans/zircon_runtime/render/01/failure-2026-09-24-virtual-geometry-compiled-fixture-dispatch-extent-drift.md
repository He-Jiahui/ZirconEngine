---
handoff_kind: failure
status: open
created_at: 2026-09-24
summary_slug: virtual-geometry-compiled-fixture-dispatch-extent-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_runtime/tests/support/mod.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -TestTarget virtual_geometry_support_descriptor_contract -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -TestTarget virtual_geometry_debug_snapshot_contract -TestThreads 1 -VerboseOutput
---

# Render01：Virtual Geometry 编译测试 fixture 与生产 dispatch 范围不一致

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：Plugins13 Virtual Geometry support descriptor 的 compute-workload 修复与上行 `plugin_feature_compile` 验收。
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：Render01 拥有 Runtime 图编译测试中的 synthetic render-feature descriptor 与 compiled workload 断言；Plugins13 拥有插件生产 descriptor 及其 integration support fixture，不应越界覆盖其他 owner 的 graphics 测试改动。

## 失败现象与复现证据

[Plugins13 原始 failure 的 2026-09-20 独立审查](../../../zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md) 在业务源码的 C/I/M=0/0/0 之外，单独给上行 compiled-graph consumer 标记 Important=1：生产 `zircon_plugins/virtual_geometry/runtime/src/lib.rs` 对 `virtual-geometry-node-cluster-cull` 的 workgroup 和 dispatch groups 均为 `[64,1,1]`；Runtime integration support 已将 dispatch 从 `[1,1,1]` 同步到 `[64,1,1]`，尚待受管 Rust 门。

2026-09-24 当前源码重新核对仍有独立矛盾：`plugin_render_feature_fixtures.rs:163-167` 的 `RenderGraphComputeWorkload::fixed` 使用 workgroup `[64,1,1]`、dispatch `[1,1,1]`；`plugin_feature_compile.rs:228-240` 又要求编译结果 `RenderGraphComputeDispatchExtent::Fixed([1,1,1])`。这样即使现有精确测试绿，也只证明 synthetic fixture 自洽，不能证明生产插件的 `[64,1,1]` 编译合同。两个 graphics 文件的现行工作树均为非本会话修改，当前 SHA-256 分别为 `fe096fe7f0e9f5e537afd2b986bc7d765251fd126ed2a2bd141ae6e23c154ea4` 和 `7f1803836abc57cf2e2e1dd10b7bbd4cdb6300026ebbb26010239d194f447a9f`；生产插件源码为 `7eb819000168dcaf597d0d65314ccc06b01c613a53eada6110bd78eefa43f291`。这些只读哈希保留原始现场，不作为当前 Session 的源码 attribution。

## 最低共享层根因

插件的真实 compute workload 已按 GPU executor 能处理的范围升级，而 Render01 测试 fixture 与其 consumer assertion 仍冻结旧 dispatch。测试绕过了生产 descriptor 的当前 dispatch 约束，把第二套数据形状误当作图编译单一事实源。

## 架构修复验收

- 由两份 graphics 文件的实际 owner 核对当前差异、租约和哈希；在同一已证明归属的快照中同步 Virtual Geometry synthetic fixture 与 compiled assertion 到生产 dispatch 合同，同时保持 Hybrid GI 自己的 `[1,1,1]` 需求不变。不得直接接管或格式化其他会话在途修改。
- 精确执行 `gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph` 并确认确实运行目标测试；重新跑 Plugins13 support descriptor integration 与 Runtime04 原始 VG debug-snapshot integration，验证生产、support、编译结果为同一 dispatch 合同。旧 `0 passed / 3 failed / 4 ignored` 不是通过证据。
- 受管 Windows Cargo 保持 `--locked`，产物由协调器分配 D/E/F 合规目录。源码/命令/配置一致的动态证据、独立 C/I/M=0/0/0、跨计划回传与 closeout 完成前保留本 failure 和 Plugins13/Runtime04 上游链的 open 状态。

## 禁止临时方案

- 不回退已审查的生产插件 `[64,1,1]`，不弱化 RenderGraph 的 workload validation，不通过跳过测试、只改断言或让两个 fixture 永久分叉来取得绿色结果。
- 不把旧 source/static 检查、外部 `E:\Git\zr_vm` dirty 时拒绝入队的请求或失败的原始集成测试报成受管动态验收通过。

## 修复结果与回传

Open state: `reported / foreign graphics source attribution pending / managed Rust acceptance pending`。本 handoff 仅由 `failure-roll-01a084c8-render01-vg-compiled-fixture-r1` 持有文档；两份 graphics 源码、生产插件与 support fixture 均未编辑或领取归属。不得据此标记 Plugins13/Runtime04 已 fixed、提交或企微通知已发送。

## 2026-09-27 current-source fixture correction

Stable fixing Session `failure-roll-01a0df1a-render01-vg-fixture-r2` registered the exact
failure, synthetic fixture, and compiled assertion paths at baseline epoch `627`. The
coordinator transfer preview `da1e39e57351552ee8ca34a62fe77926f8dc6821287ce03c200325b9c6e370ff`
adopted their current bytes from archived/unattributed state; all three paths were leased
before editing. The test files' pre-existing formatting differences are retained.

The production Virtual Geometry descriptor and Runtime integration support descriptor
both declare workgroup and dispatch `[64, 1, 1]`. The Render01 synthetic fixture now gives
the cull pass dispatch `[64, 1, 1]`, and its compiled workload assertion expects that value.
The Hybrid GI dispatch expectation remains `[1, 1, 1]`. Current edited source SHA-256 values:

- `plugin_render_feature_fixtures.rs`: `f1d2bd9b944b56a7f78faf9a0b38229f19744f1a68bd868c3d2e81ce186bb2af`.
- `plugin_feature_compile.rs`: `d65907e65582dd39c029a7db0315b005e42c4b51c8643c3d2eff3f2d4094fd6e`.

Scoped `git diff --check` passed. Managed `--locked` Rust execution of the named graph test,
Virtual Geometry support descriptor test, and debug snapshot test is pending; source agreement
does not count as dynamic acceptance. Independent C/I/M review, canonical return, closeout,
and WeCom result are also pending. The failure remains `open`.

The production descriptor (`7eb819000168dcaf597d0d65314ccc06b01c613a53eada6110bd78eefa43f291`)
and Runtime integration support (`7bb354ad18b839b03784f64ca499d4e0bf9fdff21648eb7e0ecea96057646c5c`)
are still uncommitted under archived Plugins13 Session
`failure-roll-01a084c8-plugins13-vg-support-r2`. They are separate-owner inputs to the
original upward tests. Render01 has not claimed them or submitted a Cargo ticket that would
silently compile their older HEAD bytes. A source-matched managed Rust request must follow
the Plugins13 owner handoff or scoped integration with the required compile evidence;
the existing external `E:/Git/zr_vm` dirty preflight remains a separate admission blocker.
