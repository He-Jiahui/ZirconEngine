---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: lightmap-forward-bind-group-integration-compile
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_runtime/render/11
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
tests:
  - cargo test -p zircon_editor --locked --verbose
resolved_at: 2026-07-13
---


# Render 11：lightmap 前向 bind-group 接线编译失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1 Windows full lib testing stage
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：两个编译错误都由 EL-M3 新增 `SceneLightmapResources` 进入 mesh pipeline / Forward group 1 的半迁移造成；Editor 15 导出流水线不拥有 WGPU queue、lightmap GPU resource lifetime 或 forward receiver ABI。

## 失败现象与复现证据

Windows coordinator validator 执行 `cargo test -p zircon_editor --locked --verbose`，受管 job
`d42bb4a651604962a3cc678b4ef663bb` 在编译 `zircon_runtime` 时 exit 101，尚未进入 Editor 测试体：

- `mesh_pipeline_cache/construct.rs:82`：`SceneLightmapResources::new(device, queue)` 使用当前
  `MeshPipelineCache::new` 签名中不存在的 `queue`，报 E0425。
- `mesh_pipeline_cache/forward_shadow_receiver.rs:139`：
  `self.lightmaps.bindings().bind_group_entries()` 从语句末尾即销毁的临时
  `LightmapGpuBindings` 借用 WGPU entries，报 E0716。
- 原始 validator 日志：`.codex/tmp/editor15-m1-current-full-20260713-0243.log`。
- 两个失败文件分别在 2026-07-13 02:38:48 与 02:39:45 更新；同一时间 Render 11 产出记录明确
  EL-M3 正在把 bindings 23/24/28 接入 Forward+/Deferred，Rust compile 尚未通过。

Editor 15 自有 current-binary focused 矩阵仍为 `core::export` 13/13、Build/Export pane 6/6、wizard
session 6/6、actions 5/5、job queue 2/2、output-folder 4/4；因此不能在导出层修补这两个错误。

## 最低共享层根因

最低已证实边界是 Render 11 EL-M3 的 GPU resource 构造与 bind-group entry lifetime 没有完成同一次硬切：

1. `SceneLightmapResources` 的 fallback atlas 初始化需要 `wgpu::Queue`，但其 owner 被放进
   `MeshPipelineCache` 后，构造签名及全部生产/测试调用方没有一起迁移。
2. `LightmapGpuBindings::bind_group_entries` 返回借用其自身 Arc 字段的 entries；调用方把 bindings 当临时值，
   导致 entries 的借用长于临时 owner。

这是一个共享 WGPU 构造/生命周期合同问题，不是 Editor 测试、Render 18 volumetric/OIT 或单个调用点问题。

## 架构修复验收

- 为 `SceneLightmapResources` 选择唯一构造权威：若继续由 `MeshPipelineCache` 所有，则把 `&wgpu::Queue`
  显式加入唯一构造 API，并同变更迁移生产构造与全部测试调用方；不得增加第二个无 queue 构造或静默 fallback。
- 在 forward receiver 中先持有 `LightmapGpuBindings` 局部 owner，再扩展 entries；bindings 23/24/28
  的 layout 与 bind-group resource 类型必须继续一致。
- 先运行 Render 11 lightmap GPU ABI/资源 focused checks，再运行 `cargo test -p zircon_runtime --lib --locked --no-run`，
  最后重跑来源命令 `cargo test -p zircon_editor --locked --verbose`。
- 修复后必须确认 Forward+/Deferred 都消费同一 lightmap/probe GPU owner，且旧 baked-lighting 双权威未被恢复。

## 禁止临时方案

- 禁止在 Editor 15 增加条件编译、局部 queue mock、跳过 Runtime 编译或复用旧 test binary 声称 full gate 通过。
- 禁止增加第二套 `SceneLightmapResources` 构造器、泄漏 bindings、把 WGPU entry 改成不受 owner 约束的裸引用，
  或移除 bindings 23/24/28 来绕过编译。
- 禁止恢复 legacy baked-lighting owner、兼容 re-export、silent fallback 或 test-only bypass。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| Render 11 EL-M3 / Editor 15 M1 | lightmap forward bind-group 构造与 lifetime | `open-已精确复现并路由` | 2026-07-13 | Windows validator job `d42bb4a651604962a3cc678b4ef663bb` exit 101；E0425 `queue` 未进入构造合同，E0716 entries 借用临时 bindings；Editor15 自有 focused 36/36。 |
| Render 11 EL-M3 / Editor 15 M1 | 来源 validator 编译向上复验 | `实现已生效-完整回传仍待完成` | 2026-07-13 | Runtime05 owner 已把 `&wgpu::Queue` 纳入 `MeshPipelineCache::new` 唯一构造链，并在 forward receiver 中持有 `lightmap_bindings` 局部 owner。Windows official validator job `9c0bba0554b042c2b3c5a139a8bb10a7` 随后完成 `zircon_runtime` 与 `zircon_editor` test-profile 编译，原 E0425/E0716 未再出现；验证在进入 Editor full harness 后由 Editor14 线程耗尽（5547 threads）异常终止，故本条只接受来源编译向上门，不外推 Render11 focused/no-run 全部通过。日志 `.codex/tmp/editor15-m1-post-render11-fix-20260713-0304.log`。 |

## 修复结果与回传

- 根因：SceneLightmapResources 进入 MeshPipelineCache 时未把 queue 构造依赖与 LightmapGpuBindings 生命周期同步硬切，分别触发 E0425 和 E0716。
- 架构修复：唯一 MeshPipelineCache::new 与 SceneLightmapResources::new 显式接收 wgpu::Queue；构造时写入确定性 RGBA16F 黑色 fallback atlas；Forward 先持有 LightmapGpuBindings 局部 owner，Deferred 复用同一 GPU owner，不恢复 legacy baked-lighting owner。
- 验证：Python lightmap/HybridGI focused 18/18；Rust lightmap_binding 2/2；cargo test -p zircon_runtime --lib --locked --no-run 通过。来源 cargo test -p zircon_editor --locked --verbose 中原 E0425/E0716 消失，随后被并发 Editor typed_canvas 文件缺失与 timeline_strip 初始化遗漏阻断，属于 Editor 当前迁移而非 Render11。
- 回传：Render11 EL-M3 lightmap Forward/Deferred 构造与借用合同已修复并完成 Runtime 向上编译验证；交接返回 Editor15，Editor full gate 的剩余失败改由其 typed-canvas 迁移负责。
- 后续 ABI 校正：交接返回后发现旧 sampler 25 与 volumetric params 25 重叠；Render 11 将 sampler 硬切到 28，并新增 lightmap/volumetric binding 不重叠断言。该校正不改变本交接的 queue/lifetime 根因与修复结论。
